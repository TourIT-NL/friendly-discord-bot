# 📜 Logging Strategy: Transparency Without Exposure

Effective logging is the difference between a tool that "just works" and a professional application. This document details our structured logging implementation.

---

## 🛠️ The Tech Stack

We use the **`tracing`** ecosystem in Rust, which is the gold standard for high-performance, structured instrumentation.

### Libraries:

- `tracing`: Core API for event logging.
- `tracing-subscriber`: For filtering and formatting logs.
- `tracing-appender`: For high-speed file writing and rotation.

---

## 🏗️ Structured Logs

Unlike standard text logs, our logs are structured as **JSON** objects. This allows for:

- **Contextual Data**: Attaching `channel_id` or `guild_id` to an event without string concatenation.
- **Filtering**: Easily showing only `ERROR` events while ignoring `DEBUG` messages.

---

## 📁 File Management (Rotation)

To protect the user's disk space, we use a **Rotating File Appender**:

- **Path**: `User/AppData/Local/DiscordPurge/logs/`.
- **Max Files**: 3 log files.
- **Max Size**: 5MB per file.
- **Logic**: When the current log exceeds 5MB, the oldest is deleted, and a new one starts.

---

## 🚨 Log Levels

1.  **TRACE**: High-frequency network events (Headers, heartbeats).
2.  **DEBUG**: Developer-focused info (Logic flow, variable states).
3.  **INFO**: Major events (Login success, Cleanup started, App boot).
4.  **WARN**: Recoverable errors (Rate limit hit, Retrying API call).
5.  **ERROR**: Critical failures (Missing credentials, IO failure).

---

## 🛡️ Privacy Policy

**No message content is ever logged.** We only log event metadata (counts, success/fail status, and identifiers) to ensure that the user's private conversations remain private, even in the log files.

_Last updated: February 25, 2026_
