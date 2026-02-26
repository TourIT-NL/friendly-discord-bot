// src-tauri/src/api/rate_limiter/actor.rs

use crate::api::discord_routes::get_discord_route;
use crate::api::rate_limiter::fingerprint::{BrowserProfile, FingerprintManager};
use crate::api::rate_limiter::types::{ApiRequest, ApiResponseContent, BucketInfo};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use rand::Rng;
use reqwest::{Client, Response, header};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
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
        }
    }

    fn build_client(app: &tauri::AppHandle, profile: &BrowserProfile, locale: &str) -> Client {
        let mut builder = Client::builder()
            .user_agent(&profile.user_agent)
            .default_headers({
                let mut h = header::HeaderMap::new();
                h.insert(
                    "x-super-properties",
                    header::HeaderValue::from_str(&FingerprintManager::generate_super_properties(
                        profile, locale,
                    ))
                    .unwrap(),
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

        if let Ok(proxy) = Vault::get_credential(app, "proxy_url") {
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

        while let Some(req) = self.inbox.recv().await {
            match req {
                ApiRequest::RebuildClient => {
                    self.profile = FingerprintManager::random_profile();
                    self.client = Self::build_client(
                        &self.app_handle,
                        &self.profile,
                        &FingerprintManager::get_system_locale(),
                    );
                }
                ApiRequest::Standard {
                    method,
                    url,
                    body,
                    auth_token,
                    is_bearer,
                    return_raw_bytes,
                    response_tx,
                    referer,
                    locale,
                    timezone,
                    profile,
                } => {
                    let client = self.client.clone();
                    let buckets = self.buckets.clone();
                    let global = self.global_reset_at.clone();
                    let route = get_discord_route(&url).to_string();
                    let active_profile = profile.unwrap_or_else(|| self.profile.clone());
                    let active_locale =
                        locale.unwrap_or_else(|| FingerprintManager::get_system_locale());
                    let active_tz = timezone.unwrap_or_else(|| "UTC".to_string());

                    tokio::spawn(async move {
                        let bucket_arc = {
                            let mut map = buckets.lock().await;
                            map.entry(route)
                                .or_insert_with(|| Arc::new(Mutex::new(BucketInfo::default())))
                                .clone()
                        };

                        loop {
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
                                    tokio::time::sleep(
                                        b.reset_at.saturating_duration_since(now)
                                            + Duration::from_millis(100),
                                    )
                                    .await;
                                    continue;
                                }
                                b.remaining -= 1;
                            }

                            let mut rb = client.request(method.clone(), &url);
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

                            if let Some(r) = referer.clone() {
                                rb = rb.header("referer", r);
                            }
                            if is_bearer {
                                rb = rb.header("authorization", format!("Bearer {}", auth_token));
                            } else {
                                rb = rb.header("authorization", &auth_token);
                            }
                            if let Some(b) = body.clone() {
                                rb = rb.json(&b);
                            }

                            match rb.send().await {
                                Ok(resp) => {
                                    let status = resp.status();
                                    let result = if status.is_success() {
                                        if return_raw_bytes {
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
                                    let _ = response_tx.send(result);
                                    break;
                                }
                                Err(e) => {
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
}
