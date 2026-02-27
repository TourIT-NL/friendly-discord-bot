// src-tauri/src/api/rate_limiter/actor.rs

use crate::api::discord_routes::get_discord_route;
use crate::api::rate_limiter::fingerprint::{BrowserProfile, FingerprintManager};
use crate::api::rate_limiter::types::{ApiRequest, ApiResponseContent, BucketInfo};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use rand::Rng;
use reqwest::{Client, Response, header};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::{Mutex, mpsc};

pub struct RateLimiterActor {
    pub inbox: mpsc::Receiver<ApiRequest>,
    pub client: Client,
    pub profile: BrowserProfile,
    pub buckets: Arc<Mutex<HashMap<String, Arc<Mutex<BucketInfo>>>>>,
    pub global_reset_at: Arc<Mutex<Instant>>,
    pub app_handle: tauri::AppHandle,
    pub global_429_count: Arc<AtomicU32>,
}

impl RateLimiterActor {
    pub fn new(inbox: mpsc::Receiver<ApiRequest>, app_handle: tauri::AppHandle) -> Self {
        let profile = FingerprintManager::random_profile();
        let client = Self::build_client(
            &app_handle,
            &profile,
            &FingerprintManager::get_system_locale(),
        );
        Self {
            inbox,
            client,
            profile,
            buckets: Arc::new(Mutex::new(HashMap::new())),
            global_reset_at: Arc::new(Mutex::new(Instant::now())),
            app_handle,
            global_429_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn build_client(
        app_handle: &tauri::AppHandle,
        profile: &BrowserProfile,
        locale: &str,
    ) -> Client {
        let super_props = FingerprintManager::generate_super_properties(profile, locale);

        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(&profile.user_agent)
            .default_headers({
                let mut h = header::HeaderMap::new();
                h.insert(
                    "x-super-properties",
                    header::HeaderValue::from_str(&super_props).unwrap(),
                );
                h.insert(
                    "origin",
                    header::HeaderValue::from_static("https://discord.com"),
                );
                for (name, val) in FingerprintManager::generate_client_hints(profile) {
                    h.insert(name, header::HeaderValue::from_str(&val).unwrap());
                }
                h
            });

        if let Ok(proxy) = Vault::get_credential(app_handle, "proxy_url") {
            if !proxy.is_empty() {
                if let Ok(p) = reqwest::Proxy::all(&proxy) {
                    builder = builder.proxy(p);
                }
            }
        }

        builder.build().unwrap()
    }

    pub async fn run(&mut self) {
        Logger::info(
            &self.app_handle,
            "[LIM] Engine Dispatcher active and resilient",
            None,
        );

        while let Some(request) = self.inbox.recv().await {
            match request {
                ApiRequest::RebuildClient => {
                    self.profile = FingerprintManager::random_profile();
                    self.client = Self::build_client(
                        &self.app_handle,
                        &self.profile,
                        &FingerprintManager::get_system_locale(),
                    );
                    Logger::info(
                        &self.app_handle,
                        "[LIM] Client rebuilt with new profile",
                        None,
                    );
                }
                ApiRequest::Standard(req) => {
                    let client = self.client.clone();
                    let buckets = self.buckets.clone();
                    let global = self.global_reset_at.clone();
                    let app_handle = self.app_handle.clone();
                    let route = get_discord_route(&req.url).to_string();
                    let global_429_count = self.global_429_count.clone();

                    let active_profile =
                        req.profile.clone().unwrap_or_else(|| self.profile.clone());
                    let active_locale = req
                        .locale
                        .clone()
                        .unwrap_or_else(|| FingerprintManager::get_system_locale());
                    let active_tz = req.timezone.clone().unwrap_or_else(|| "UTC".to_string());

                    tokio::spawn(async move {
                        let bucket_arc = {
                            let mut map = buckets.lock().await;
                            map.entry(route.clone())
                                .or_insert_with(|| Arc::new(Mutex::new(BucketInfo::default())))
                                .clone()
                        };

                        loop {
                            let jitter = rand::thread_rng().gen_range(50..250);
                            tokio::time::sleep(Duration::from_millis(jitter)).await;

                            let now = Instant::now();
                            {
                                let g = global.lock().await;
                                if now < *g {
                                    tokio::time::sleep(*g - now).await;
                                    continue;
                                }
                            }

                            {
                                let mut b = bucket_arc.lock().await;
                                if now >= b.reset_at {
                                    b.remaining = b.limit;
                                }
                                if b.remaining == 0 {
                                    let wait = b.reset_at.saturating_duration_since(now)
                                        + Duration::from_millis(100);
                                    drop(b);
                                    tokio::time::sleep(wait).await;
                                    continue;
                                }
                                b.remaining -= 1;
                                b.last_request_at = now;
                            }

                            let mut rb = client.request(req.method.clone(), &req.url);

                            rb = rb.header("user-agent", &active_profile.user_agent);
                            rb = rb.header(
                                "x-super-properties",
                                FingerprintManager::generate_super_properties(
                                    &active_profile,
                                    &active_locale,
                                ),
                            );
                            rb = rb.header(
                                "accept-language",
                                FingerprintManager::generate_accept_language(&active_locale),
                            );
                            rb = rb.header(
                                "cookie",
                                FingerprintManager::generate_synthetic_cookies(&active_locale),
                            );
                            rb = rb.header("x-discord-locale", &active_locale);
                            rb = rb.header("x-discord-timezone", &active_tz);

                            for (name, val) in
                                FingerprintManager::generate_client_hints(&active_profile)
                            {
                                rb = rb.header(name, val);
                            }

                            if let Some(r) = req.referer.clone() {
                                rb = rb.header("referer", r);
                            } else if req.url.contains("/messages") {
                                rb = rb.header("referer", "https://discord.com/channels/@me");
                            }

                            if req.is_bearer {
                                rb = rb
                                    .header("authorization", format!("Bearer {}", req.auth_token));
                            } else {
                                rb = rb.header("authorization", &req.auth_token);
                            }
                            if let Some(b) = req.body.clone() {
                                rb = rb.json(&b);
                            }

                            // Elaborate OperationManager integration
                            let op_manager = app_handle.state::<OperationManager>();
                            if op_manager.state.is_running.load(Ordering::SeqCst) {
                                Logger::trace(
                                    &app_handle,
                                    &format!("[LIM] Request linked to active operation: {}", route),
                                    None,
                                );
                            }

                            match rb.send().await {
                                Ok(resp) => {
                                    let status = resp.status();
                                    Self::handle_rate_limits(
                                        &app_handle,
                                        &bucket_arc,
                                        &global,
                                        &resp,
                                        &global_429_count,
                                    )
                                    .await;

                                    if status.as_u16() == 429 {
                                        continue;
                                    }

                                    global_429_count.store(0, Ordering::SeqCst);

                                    let result = if status.is_success() {
                                        if req.return_raw_bytes {
                                            resp.bytes()
                                                .await
                                                .map(ApiResponseContent::Bytes)
                                                .map_err(AppError::from)
                                        } else if status == reqwest::StatusCode::NO_CONTENT {
                                            Ok(ApiResponseContent::Json(serde_json::json!({})))
                                        } else {
                                            resp.json::<serde_json::Value>()
                                                .await
                                                .map(ApiResponseContent::Json)
                                                .map_err(AppError::from)
                                        }
                                    } else {
                                        let json = resp
                                            .json::<serde_json::Value>()
                                            .await
                                            .unwrap_or_default();
                                        Err(AppError::from_discord_json(&json))
                                    };
                                    let _ = req.response_tx.send(result);
                                    break;
                                }
                                Err(e) => {
                                    let _ = req.response_tx.send(Err(AppError::from(e)));
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    async fn handle_rate_limits(
        app: &tauri::AppHandle,
        bucket_arc: &Arc<Mutex<BucketInfo>>,
        global_throttle: &Arc<Mutex<Instant>>,
        response: &Response,
        global_429_count: &AtomicU32,
    ) {
        let headers = response.headers();
        let mut bucket = bucket_arc.lock().await;
        let now = Instant::now();

        if let Some(bid) = headers
            .get("x-ratelimit-bucket")
            .and_then(|v| v.to_str().ok())
        {
            bucket.bucket_id = Some(bid.to_string());
        }

        if let Some(limit) = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            bucket.limit = limit;
        }
        if let Some(remaining) = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            bucket.remaining = remaining;
        }
        if let Some(reset_after) = headers
            .get("x-ratelimit-reset-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok())
        {
            bucket.reset_at = now + Duration::from_secs_f64(reset_after);
        }

        if response.status().as_u16() == 429 {
            bucket.consecutive_429s += 1;
            let retry_after = headers
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0);

            if global_429_count.fetch_add(1, Ordering::SeqCst) > 10 {
                let mut g = global_throttle.lock().await;
                *g = now + Duration::from_secs(60);
                Logger::error(
                    app,
                    "[LIM] Circuit breaker active! Locking engine for 60s.",
                    None,
                );
                global_429_count.store(0, Ordering::SeqCst);
            }

            let backoff = Duration::from_secs_f64(retry_after)
                + Duration::from_secs(2u64.pow(bucket.consecutive_429s.min(6)));
            bucket.reset_at = now + backoff;
            bucket.remaining = 0;

            if headers.contains_key("x-ratelimit-global") {
                let mut g = global_throttle.lock().await;
                *g = now + Duration::from_secs_f64(retry_after);
                Logger::error(
                    app,
                    &format!("[LIM] GLOBAL 429. Throttle for {:?}", retry_after),
                    None,
                );
            } else {
                Logger::warn(
                    app,
                    &format!("[LIM] Bucket 429. Backoff for {:?}", backoff),
                    None,
                );
            }
        } else {
            bucket.consecutive_429s = 0;
        }
    }
}
