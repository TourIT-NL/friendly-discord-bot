// src-tauri/src/api/rate_limiter/actor.rs

use crate::api::discord_routes::get_discord_route;
use crate::api::rate_limiter::types::{ApiRequest, BucketInfo};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use rand::Rng;
use reqwest::{Client, Response, header};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::{Mutex, mpsc};

pub struct RateLimiterActor {
    pub(crate) inbox: mpsc::Receiver<ApiRequest>,
    pub(crate) client: Client,
    pub(crate) buckets: Arc<Mutex<HashMap<String, Arc<Mutex<BucketInfo>>>>>,
    pub(crate) global_reset_at: Arc<Mutex<Instant>>,
    pub(crate) app_handle: tauri::AppHandle,
}

impl RateLimiterActor {
    pub fn new(inbox: mpsc::Receiver<ApiRequest>, app_handle: tauri::AppHandle) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Norton/144.0.0.0")
            .default_headers({
                let mut headers = header::HeaderMap::new();
                headers.insert("x-super-properties", header::HeaderValue::from_static("eyJvcyI6IldpbmRvd3MiLCJicm93c2VyIjoiQ2hyb21lIiwiZGV2aWNlIjoiIiwic3lzdGVtX2xvY2FsZSI6ImVuLVVTIiwiaGFzX2NsaWVudF9tb2RzIjpmYWxzZSwiYnJvd3Nlcl91c2VyX2FnZW50IjoiTW96aWxsYS81LjAgKFdpbmRvd3MgTlQgMTAuMDsgV2luNjQ7IHg2NCkgQXBwbGVXZWJLaXQvNTM3LjM2IChLSFRNTCwgbGlrZSBHZWNrbykgQ2hyb21lLzE0NC4wLjAuMCBTYWZhcmkvNTM3LjM2IE5vcnRvbi8xNDQuMC4wLjAiLCJicm93c2VyX3ZlcnNpb24iOiIxNDQuMC4wLjAiLCJvc192ZXJzaW9uIjoiMTAiLCJyZWZlcnJlciI6IiIsInJlZmVycmluZ19kb21haW4iOiIiLCJyZWZlcnJlcl9jdXJyZW50IjoiaHR0cHM6Ly9kaXNjb3JkLmNvbS8iLCJyZWZlcnJpbmdfZG9tYWluX2N1cnJlbnQiOiJkaXNjb3JkLmNvbSIsInJlbGVhc2VfY2hhbm5lbCI6InN0YWJsZSIsImNsaWVudF9idWlsZF9udW1iZXIiOjUwMDMzNCwiY2xpZW50X2V2ZW50X3NvdXJjZSI6bnVsbCwiY2xpZW50X2xhdW5jaF9pZCI6ImE3ZTBhNDczLTY5ZTAtNDE4Yi05M2ZlLWVlNjRkNTk2NGRhMSIsImxhdW5jaF9zaWduYXR1cmUiOiJhZDMxNjA4Mi1jNTRiLTQ1M2EtOGE3ZS0wMzk5NzgwODU0YmEiLCJjbGllbnRfYXBwX3N0YXRlIjoiZm9jdXNlZCIsImNsaWVudF9oZWFydGJlYXRfc2Vzc2lvbl9pZCI6IjNkMjJiZGZkLTlkN2MtNDM3MS1iNDllLWRhN2U3YzhlYzI2ZSJ9"));
                headers.insert("origin", header::HeaderValue::from_static("https://discord.com"));
                headers
            })
            .build()
            .expect("Failed to build reqwest client");

        Self {
            inbox,
            client,
            buckets: Arc::new(Mutex::new(HashMap::new())),
            global_reset_at: Arc::new(Mutex::new(Instant::now())),
            app_handle,
        }
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
        let client_clone = self.client.clone();
        let app_handle_clone = self.app_handle.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(45)).await;
                let op_manager = app_handle_clone.state::<OperationManager>();
                
                // Only send heartbeat if an operation is currently running or app is in active use
                if op_manager.state.is_running.load(Ordering::SeqCst) {
                    let (token, is_bearer) = match crate::core::vault::Vault::get_active_token(&app_handle_clone) {
                        Ok(t) => t,
                        _ => continue,
                    };

                    let mut req = client_clone.post("https://discord.com/api/v9/users/@me/meaningfully-online");
                    if is_bearer {
                        req = req.header(header::AUTHORIZATION, format!("Bearer {}", token));
                    } else {
                        req = req.header(header::AUTHORIZATION, token);
                    }

                    let _ = req.send().await;
                }
            }
        });

        while let Some(request) = self.inbox.recv().await {
            // Block telemetry calls entirely at the dispatcher level
            if request.url.contains("/beaker") || request.url.contains("/metrics") {
                let _ = request.response_tx.send(Err(AppError {
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
            let route = Self::get_route(&request.url);

            tokio::spawn(async move {
                let mut retry_count = 0;
                const MAX_RETRIES: u32 = 3;

                loop {
                    // Apply human-like jitter before every request
                    let jitter = rand::thread_rng().gen_range(50..250);
                    tokio::time::sleep(Duration::from_millis(jitter)).await;

                    let now = Instant::now();

                    // 1. Global Wait
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
                    let mut req_builder = client.request(request.method.clone(), &request.url);
                    
                    // Contextual Referer Injection
                    if request.url.contains("/messages") {
                        req_builder = req_builder.header("referer", "https://discord.com/channels/@me");
                    } else if request.url.contains("/settings") || request.url.contains("/harvest") {
                        req_builder = req_builder.header("referer", "https://discord.com/settings/privacy");
                    } else if request.url.contains("/billing") {
                        req_builder = req_builder.header("referer", "https://discord.com/settings/billing");
                    }

                    if request.is_bearer {
                        req_builder = req_builder.header(
                            header::AUTHORIZATION,
                            format!("Bearer {}", request.auth_token),
                        );
                    } else {
                        req_builder =
                            req_builder.header(header::AUTHORIZATION, &request.auth_token);
                    }

                    if let Some(body) = request.body.clone() {
                        req_builder = req_builder.json(&body);
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
                                Logger::warn(
                                    &app_handle,
                                    &format!("[LIM] 429 received for {}", route),
                                    None,
                                );
                                continue; // Retry after rate limit
                            }

                            if !status.is_success()
                                && status.is_server_error()
                                && retry_count < MAX_RETRIES
                            {
                                retry_count += 1;
                                tokio::time::sleep(Duration::from_secs(retry_count as u64)).await;
                                continue; // Retry after server error
                            }

                            // Deserialize response body within the actor
                            let result = if status.is_success() {
                                if status == reqwest::StatusCode::NO_CONTENT {
                                    Ok(serde_json::json!({}))
                                } else {
                                    response
                                        .json::<serde_json::Value>()
                                        .await
                                        .map_err(AppError::from)
                                }
                            } else {
                                // For non-success responses, try to read body as JSON for error details
                                // Otherwise, create a generic AppError
                                response
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(AppError::from)
                                    .and_then(|json_body| {
                                        Err(AppError {
                                            user_message: format!("API error: {}", json_body),
                                            error_code: format!("api_http_{}", status.as_u16()),
                                            technical_details: Some(json_body.to_string()),
                                        })
                                    })
                                    .map_err(|_| AppError {
                                        user_message: format!("API error with status {}", status),
                                        error_code: format!("api_http_{}", status.as_u16()),
                                        technical_details: Some(format!(
                                            "Response status: {}",
                                            status
                                        )),
                                    })
                            };

                            let _ = request.response_tx.send(result);
                            break;
                        }
                        Err(e) => {
                            if retry_count < MAX_RETRIES {
                                retry_count += 1;
                                tokio::time::sleep(Duration::from_secs(retry_count as u64)).await;
                                continue;
                            }
                            let _ = request.response_tx.send(Err(AppError::from(e)));
                            break;
                        }
                    }
                }
            });
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
