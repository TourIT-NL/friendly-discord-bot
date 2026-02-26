# 🚨 Architecture: Intelligent Rate Limiting Actor

Performance and reliability depend on how well we interact with Discord's API. This document details the asynchronous **Actor system** that manages global rate limits, ensuring your account remains in good standing while processing massive cleanup tasks.

---

## 🏗️ The Actor Design Pattern

In **Discord Purge**, all API requests are routed through a single, centralized "Rate Limiter Actor" in the Rust backend. This prevents multiple threads from accidentally slamming the API and causing account flags or temporary blocks.

### The Actor Logic Flow:

1.  **Request**: Any module (Message Deleter, Server Leaver) sends an `ApiRequest` message to the Actor's inbox.
2.  **Queueing**: The Actor holds an **MPSC** (Multi-Producer, Single-Consumer) queue, ensuring requests are handled in order.
3.  **Bucket Analysis**: The Actor maintains a map of "Rate Limit Buckets". Discord groups requests (e.g., all deletions in channel A belong to one bucket).
4.  **Throttling**: Before sending, the Actor checks if the bucket has `remaining` requests.
5.  **Suspension**: If the limit is reached, the Actor **asynchronously sleeps** (`tokio::time::sleep`) until the `reset_after` duration expires. This is non-blocking; other UI tasks continue to run.
6.  **Header Parsing**: Every response is scanned for:
    - `X-RateLimit-Limit`: Total requests allowed in the window.
    - `X-RateLimit-Remaining`: Requests left.
    - `X-RateLimit-Reset-After`: Seconds until reset.
7.  **Adaptive Learning**: The Actor dynamically updates its internal timing based on these headers.

---

## 🎭 Fingerprinting & Stealth

To blend into regular Discord traffic and prevent automated detection, the Rate Limiter Actor employs several advanced techniques:

### Dynamic User-Agent Pool

The Actor does not use a single User-Agent. It rotates between a pool of modern browser strings (Chrome, Firefox, Safari on Windows/macOS/Linux) to avoid fingerprinting.

### X-Super-Properties Generation

Internal Discord headers (`x-super-properties`) are dynamically generated to match the selected User-Agent and platform, ensuring consistency that passes automated integrity checks.

### Secure Proxy Tunneling

Users can configure a global SOCKS5 or Tor proxy. When active, the Actor's HTTP client is rebuilt to route all traffic through the proxy, masking the user's real IP address from Discord's telemetry endpoints.

---

## 🎲 Jitter & Backoff Safeguards

To distinguish our traffic from "dumb" bots and to handle unexpected network conditions, we implement:

- **Randomized Jitter**: We add a small random delay (50ms - 250ms) to every request. This breaks the predictable "one request per second" pattern that anti-abuse systems look for.
- **Exponential Backoff**: If we receive a `429 Too Many Requests` error, we don't just wait for the reset; we multiply our wait time for that specific route.
- **Graceful Recovery**: If multiple `429`s occur, the Actor will automatically pause all outgoing traffic for a "Cool Down" period.

---

## ⚡ Performance Impact

By using **non-blocking asynchronous Rust** via the `Tokio` runtime, we achieve:

- **Perfect Fluidity**: The UI progress bars update smoothly even while the backend is "sleeping" due to a rate limit.
- **Resource Efficiency**: The app consumes minimal CPU and RAM even when managing a queue of 5,000+ deletion requests.

---

## 🛠️ Configuration (deny.toml)

We also utilize `cargo-deny` to ensure our network dependencies (`reqwest`, `tokio`) are up-to-date and free of known vulnerabilities, maintaining the "Unyielding Security" tenet.

_Last updated: February 25, 2026_
