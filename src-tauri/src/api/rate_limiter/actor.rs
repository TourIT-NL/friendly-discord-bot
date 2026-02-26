// src-tauri/src/api/rate_limiter/actor.rs

use crate::api::discord_routes::get_discord_route;
use crate::api::rate_limiter::fingerprint::FingerprintManager;
use crate::api::rate_limiter::types::{ApiRequest, ApiResponseContent, BucketInfo};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use rand::Rng;
use reqwest::{Client, Proxy, Response, header};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::{Mutex, mpsc};

/// The central authority for all Discord API interactions.
/// Runs as an asynchronous Actor task, managing global and per-route rate limits,
/// dynamic fingerprinting (User-Agents), and proxy routing.
pub struct RateLimiterActor {
    /// Inbox for incoming API requests and control signals.
    pub(crate) inbox: mpsc::Receiver<ApiRequest>,
    /// The shared HTTP client, potentially configured with a proxy.
    pub(crate) client: Client,
    /// A map of active rate limit buckets, keyed by Discord route.
    pub(crate) buckets: Arc<Mutex<HashMap<String, Arc<Mutex<BucketInfo>>>>>,
    /// Global throttle timestamp to handle 429 GLOBAL responses.
    pub(crate) global_reset_at: Arc<Mutex<Instant>>,
    pub(crate) app_handle: tauri::AppHandle,
    /// Circuit breaker: number of consecutive global failures.
    pub(crate) failure_count: Arc<std::sync::atomic::AtomicU32>,
}

impl RateLimiterActor {
    /// Initializes a new RateLimiterActor with a fresh fingerprint.
    pub fn new(inbox: mpsc::Receiver<ApiRequest>, app_handle: tauri::AppHandle) -> Self {
        let client = Self::build_client(&app_handle);

        Self {
            inbox,
            client,
            buckets: Arc::new(Mutex::new(HashMap::new())),
            global_reset_at: Arc::new(Mutex::new(Instant::now())),
            app_handle,
            failure_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    fn build_client(app_handle: &tauri::AppHandle) -> Client {
        let user_agent = FingerprintManager::random_user_agent();
        let super_props = FingerprintManager::generate_super_properties(user_agent);

        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(user_agent)
            .default_headers({
                let mut headers = header::HeaderMap::new();
                headers.insert(
                    "x-super-properties",
                    header::HeaderValue::from_str(&super_props).unwrap(),
                );
                headers.insert(
                    "origin",
                    header::HeaderValue::from_static("https://discord.com"),
                );
                headers
            });

        if let Ok(proxy_url) = Vault::get_credential(app_handle, "proxy_url")
            && !proxy_url.is_empty()
            && let Ok(proxy) = Proxy::all(&proxy_url)
        {
            builder = builder.proxy(proxy);
            Logger::info(
                app_handle,
                &format!("[LIM] Traffic routed through proxy: {}", proxy_url),
                None,
            );
        }

        builder.build().expect("Failed to build reqwest client")
    }

    fn get_route(url: &str) -> String {
        get_discord_route(url).to_string()
    }

    pub async fn run(&mut self) {
        Logger::info(
            &self.app_handle,
            "[LIM] Engine Dispatcher active and resilient",
            None,
        );

        // Start background heartbeat task
        let app_handle_clone = self.app_handle.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(45)).await;
                let op_manager = app_handle_clone.state::<OperationManager>();

                if op_manager.state.is_running.load(Ordering::SeqCst) {
                    let (token, is_bearer) =
                        match crate::core::vault::Vault::get_active_token(&app_handle_clone) {
                            Ok(t) => t,
                            _ => continue,
                        };

                    let api_handle =
                        app_handle_clone.state::<crate::api::rate_limiter::ApiHandle>();
                    let _ = api_handle
                        .send_request_json(
                            reqwest::Method::POST,
                            "https://discord.com/api/v9/users/@me/meaningfully-online",
                            None,
                            &token,
                            is_bearer,
                        )
                        .await;
                }
            }
        });

        while let Some(request) = self.inbox.recv().await {
            match request {
                ApiRequest::RebuildClient => {
                    self.client = Self::build_client(&self.app_handle);
                    Logger::info(
                        &self.app_handle,
                        "[LIM] Client rebuilt with fresh fingerprint/proxy",
                        None,
                    );
                    continue;
                }
                ApiRequest::Standard {
                    method,
                    url,
                    body,
                    auth_token,
                    is_bearer,
                    return_raw_bytes,
                    response_tx,
                } => {
                    if url.contains("/beaker") || url.contains("/metrics") {
                        let _ = response_tx.send(Err(AppError {
                            user_message: "Telemetry blocked by Privacy Guard".into(),
                            error_code: "telemetry_blocked".into(),
                            ..Default::default()
                        }));
                        continue;
                    }

                    let client = self.client.clone();
                    let buckets_map = self.buckets.clone();
                    let global_throttle = self.global_reset_at.clone();
                    let app_handle = self.app_handle.clone();
                    let route = Self::get_route(&url);
                    let failure_count = self.failure_count.clone();

                    tokio::spawn(async move {
                        let mut retry_count = 0;
                        const MAX_RETRIES: u32 = 3;

                        loop {
                            let jitter = rand::thread_rng().gen_range(50..250);
                            tokio::time::sleep(Duration::from_millis(jitter)).await;

                            let now = Instant::now();

                            // 1. Global Wait & Circuit Breaker Check
                            {
                                let global = global_throttle.lock().await;
                                if now < *global {
                                    let wait = *global - now;
                                    tokio::time::sleep(wait).await;
                                    continue;
                                }
                            }

                            // 2. Bucket Synchronization
                            let bucket_arc = {
                                let mut map = buckets_map.lock().await;
                                map.entry(route.clone())
                                    .or_insert_with(|| Arc::new(Mutex::new(BucketInfo::default())))
                                    .clone()
                            };

                            {
                                let mut bucket = bucket_arc.lock().await;
                                if now >= bucket.reset_at {
                                    bucket.remaining = bucket.limit;
                                }

                                if bucket.remaining == 0 {
                                    let wait = bucket.reset_at.saturating_duration_since(now);
                                    if !wait.is_zero() {
                                        drop(bucket);
                                        tokio::time::sleep(wait + Duration::from_millis(100)).await;
                                        continue;
                                    }
                                }
                                bucket.remaining = bucket.remaining.saturating_sub(1);
                            }

                            // 3. Execution
                            let mut req_builder = client.request(method.clone(), &url);

                            if url.contains("/messages") {
                                req_builder = req_builder
                                    .header("referer", "https://discord.com/channels/@me");
                            } else if url.contains("/settings") || url.contains("/harvest") {
                                req_builder = req_builder
                                    .header("referer", "https://discord.com/settings/privacy");
                            }

                            if is_bearer {
                                req_builder = req_builder.header(
                                    header::AUTHORIZATION,
                                    format!("Bearer {}", auth_token),
                                );
                            } else {
                                req_builder =
                                    req_builder.header(header::AUTHORIZATION, &auth_token);
                            }

                            if let Some(b) = body.clone() {
                                req_builder = req_builder.json(&b);
                            }

                            match req_builder.send().await {
                                Ok(response) => {
                                    let status = response.status();
                                    let is_429 = status.as_u16() == 429;

                                    Self::process_headers(
                                        &app_handle,
                                        &route,
                                        &response,
                                        &bucket_arc,
                                        &global_throttle,
                                        is_429,
                                    )
                                    .await;

                                    if is_429 {
                                        let count = failure_count.fetch_add(1, Ordering::SeqCst);
                                        if count > 5 {
                                            // Trigger circuit breaker: 1 minute global cooldown
                                            let mut g = global_throttle.lock().await;
                                            *g = Instant::now() + Duration::from_secs(60);
                                            Logger::error(
                                                &app_handle,
                                                "[LIM] CIRCUIT BREAKER TRIGGERED. Global cooldown active.",
                                                None,
                                            );
                                            failure_count.store(0, Ordering::SeqCst);
                                        }

                                        let wait = {
                                            let bucket = bucket_arc.lock().await;
                                            bucket
                                                .reset_at
                                                .saturating_duration_since(Instant::now())
                                        };
                                        tokio::time::sleep(wait + Duration::from_millis(500)).await;
                                        continue;
                                    }

                                    failure_count.store(0, Ordering::SeqCst);

                                    if !status.is_success()
                                        && status.is_server_error()
                                        && retry_count < MAX_RETRIES
                                    {
                                        retry_count += 1;
                                        tokio::time::sleep(Duration::from_secs(retry_count as u64))
                                            .await;
                                        continue;
                                    }

                                    let result = if status.is_success() {
                                        if return_raw_bytes {
                                            response
                                                .bytes()
                                                .await
                                                .map(ApiResponseContent::Bytes)
                                                .map_err(AppError::from)
                                        } else if status == reqwest::StatusCode::NO_CONTENT {
                                            Ok(ApiResponseContent::Json(serde_json::json!({})))
                                        } else {
                                            response
                                                .json::<serde_json::Value>()
                                                .await
                                                .map(ApiResponseContent::Json)
                                                .map_err(AppError::from)
                                        }
                                    } else {
                                        let err_json = response
                                            .json::<serde_json::Value>()
                                            .await
                                            .unwrap_or(serde_json::json!({}));
                                        Err(AppError {
                                            user_message: format!("API error: {}", err_json),
                                            error_code: format!("api_http_{}", status.as_u16()),
                                            technical_details: Some(err_json.to_string()),
                                        })
                                    };

                                    let _ = response_tx.send(result);
                                    break;
                                }
                                Err(e) => {
                                    if retry_count < MAX_RETRIES {
                                        retry_count += 1;
                                        tokio::time::sleep(Duration::from_secs(retry_count as u64))
                                            .await;
                                        continue;
                                    }
                                    let _ = response_tx.send(Err(AppError::from(e)));
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    async fn process_headers(
        app: &tauri::AppHandle,
        route: &str,
        response: &Response,
        bucket_arc: &Arc<Mutex<BucketInfo>>,
        global_throttle: &Arc<Mutex<Instant>>,
        is_429: bool,
    ) {
        let headers = response.headers();
        let now = Instant::now();
        let mut bucket = bucket_arc.lock().await;

        if is_429 {
            bucket.consecutive_429s += 1;
        } else {
            bucket.consecutive_429s = 0;
        }

        if let Some(rem) = headers
            .get("X-RateLimit-Remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            bucket.remaining = rem;
        }
        if let Some(reset) = headers
            .get("X-RateLimit-Reset-After")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<f32>().ok())
        {
            bucket.reset_at = now + Duration::from_secs_f32(reset);
        }
        if let Some(lim) = headers
            .get("X-RateLimit-Limit")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            bucket.limit = lim;
        }

        if is_429 {
            let retry_after = headers
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.0);
            let mut wait = Duration::from_secs_f32(retry_after);
            if bucket.consecutive_429s > 1 {
                wait += Duration::from_secs(3u64.pow(bucket.consecutive_429s.min(5)));
            }

            if headers
                .get("X-RateLimit-Global")
                .and_then(|h| h.to_str().ok())
                == Some("true")
            {
                let mut g = global_throttle.lock().await;
                *g = now + wait;
                Logger::error(
                    app,
                    &format!("[LIM] GLOBAL RATE LIMIT. Locking for {:?}", wait),
                    None,
                );
            } else {
                bucket.remaining = 0;
                bucket.reset_at = now + wait;
                Logger::warn(
                    app,
                    &format!("[LIM] Route '{}' limited for {:?}", route, wait),
                    None,
                );
            }
        }
    }
}
