# 🚨 Architecture: Intelligent Rate Limiting Actor

Performance and reliability depend on how well we interact with Discord's API. This document details the asynchronous **Actor system** that manages global rate limits.

---

## 🏗️ The Actor Design Pattern

In **Discord Purge**, all API requests are routed through a single, centralized "Rate Limiter Actor" in the Rust backend. This prevents multiple threads from accidentally slamming the API and causing account flags.

### Workflow:

1.  **Request**: Any module (Message Deleter, Server Leaver) sends an `ApiRequest` message to the Actor's inbox.
2.  **Queueing**: The Actor holds an MPSC (Multi-Producer, Single-Consumer) queue.
3.  **Throttling**: The Actor checks the `remaining` requests for the current bucket.
4.  **Suspension**: If the limit is reached, the Actor **asynchronously sleeps** (`tokio::time::sleep`) until the `reset_after` duration expires.
5.  **Execution**: The Actor executes the request using `reqwest`.
6.  **Header Parsing**: The Actor parses `X-RateLimit-Remaining` and `X-RateLimit-Reset-After` from the response to update its internal state.
7.  **Response**: The result is sent back to the original caller via a one-shot channel.

---

## 🎲 Advanced Safeguards

- **Randomized Jitter**: We add a small random delay (50ms - 200ms) to every request. This prevents "robotic" patterns that could trigger anti-automation systems.
- **Exponential Backoff**: If we hit a `429 Too Many Requests` error, we don't just wait; we exponentially increase the wait time for subsequent retries.
- **Global vs. Local**: The Actor intelligently distinguishes between Global (IP-wide) and Route-specific (per channel) rate limits.

---

## ⚡ Performance Impact

By using non-blocking asynchronous Rust, we ensure that while the background Actor is waiting for a reset, the UI remains perfectly responsive and fluid.

_Last updated: February 25, 2026_
