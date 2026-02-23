# Discord Purge Rate Limiting Architecture: Preventing Discord API Abuse

This document details the critical **Rate Limiting Actor architecture** within the **Discord Purge utility**. To ensure stable operation and prevent exceeding Discord's API rate limits during intensive operations like **bulk message deletion** or **DM cleanup**, a dedicated, asynchronous actor handles all outgoing Discord API requests. This intelligent system is vital for maintaining the integrity of the user's Discord account and the reliability of the **Discord cleanup tool**.

A dedicated, asynchronous actor, implemented in **Rust**, is responsible for managing global rate limiting for all **Discord API requests** originating from the **Discord Purge application**.

```rust
// Scaffolding for src-tauri/src/api/rate_limiter.rs
// This Rust module defines the core components of the rate limiting actor for Discord API interactions.

use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::time::Duration;

// Represents a pending API request that needs to be sent to the Discord API.
pub struct ApiRequest { /* ... fields for method, url, request body, and a channel to send the response back ... */ }
// Represents the response received from a Discord API request.
pub struct ApiResponse { /* ... fields for status, response body, and relevant headers ... */ }

// The actor's state, containing its internal message queue and current rate limit information.
pub struct RateLimiterActor {
    // An MPSC (Multiple Producer, Single Consumer) channel to receive incoming API requests
    // from various parts of the Discord Purge application.
    inbox: mpsc::Receiver<ApiRequest>,
    // Shared, mutable state holding the current rate limit information, protected by a Mutex.
    rate_limit_info: Arc<Mutex<RateLimitInfo>>,
}

// Struct to store dynamic rate limit data extracted from Discord API response headers.
#[derive(Clone, Default)]
pub struct RateLimitInfo {
    // Number of requests remaining in the current rate limit window.
    remaining: u32,
    // Duration after which the current rate limit window resets.
    reset_after: Duration,
}

impl RateLimiterActor {
    // The main execution loop for the RateLimiterActor.
    pub async fn run(&mut self) {
        // Continuously process requests as they arrive in the inbox.
        while let Some(request) = self.inbox.recv().await {
            // Acquire a lock on the rate limit information to check and update it.
            let mut info = self.rate_limit_info.lock().await;
            // If no requests are remaining in the current window, pause execution
            // until the rate limit resets, preventing Discord API abuse.
            if info.remaining == 0 {
                tokio::time::sleep(info.reset_after).await;
            }
            // TODO:
            // - Execute the actual HTTP request to the Discord API using `reqwest`.
            // - Parse `X-RateLimit-*` headers from the Discord API response to update `info.remaining` and `info.reset_after`.
            // - Decrement `info.remaining` for the executed request.
            // - Send the `ApiResponse` back to the original caller of the API request.
        }
    }
}
```

This architecture is crucial for the stability and responsible operation of the **Discord Purge desktop application**, ensuring that **Discord message management** tasks are performed efficiently without leading to temporary account suspensions or blocks due to excessive API calls.
