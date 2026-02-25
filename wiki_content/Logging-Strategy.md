# 📜 Logging Strategy: Transparency Without Exposure

Effective logging is the difference between a tool that "just works" and a professional application. This document details our structured logging implementation, designed for developer insight while strictly maintaining user privacy.

---

## 🛠️ The Tech Stack: Tracing 🦀

We use the **`tracing`** ecosystem in Rust, which is the industry standard for high-performance, structured instrumentation. Unlike old-school `log` crates, `tracing` allows us to record events with structured metadata.

### Core Components:

- **`tracing`**: The primary API for recording spans and events.
- **`tracing-subscriber`**: For filtering events (e.g., show only ERRORS in production) and formatting the output.
- **`tracing-appender`**: For high-speed, non-blocking file writing and automatic rotation.

---

## 🏗️ JSON Structured Logs

To make debugging across platforms easier, our logs are structured as **JSON** objects. This allows a developer to parse them with tools like `jq`.

**Example Entry:**

```json
{
  "timestamp": "2026-02-25T10:00:00Z",
  "level": "INFO",
  "message": "Purge cycle initiated",
  "channel_id": "123456789",
  "target_count": 500
}
```

---

## 📁 Secure File Management (Rotation)

To protect the user's disk space and ensure we don't leak logs indefinitely, we use a **Rotating File Appender**:

- **Location**:
  - **Windows**: `%APPDATA%/FriendlyDiscordBot/logs/`
  - **macOS**: `~/Library/Logs/FriendlyDiscordBot/`
  - **Linux**: `~/.config/FriendlyDiscordBot/logs/`
- **Capacity**:
  - **Retention**: Maximum 3 log files.
  - **Size Threshold**: 5MB per file.
  - **Logic**: When the active log exceeds 5MB, the oldest file is purged, a secondary log is rotated, and a new file is initialized.

---

## 🚨 Log Levels & Usage

1.  **TRACE**: High-frequency network events. Only enabled in "Developer Mode".
2.  **DEBUG**: Logic flow and state changes. Used for identifying edge cases during testing.
3.  **INFO**: Major user-initiated events (Login success, Cleanup completed).
4.  **WARN**: Recoverable anomalies (Rate limit hit, API retry, Network flicker).
5.  **ERROR**: Critical failures that halt an operation (Invalid token, OS Vault access denied).

---

## 🛡️ Privacy First: Our "No Message" Policy

**No message content is ever recorded in the logs.**

We strictly log **metadata only**:

- **YES**: Channel IDs, Guild IDs, message counts, error codes, and performance metrics.
- **NO**: Actual text content of DMs, usernames of other people, or attached file content.

This ensures that even if you share your log file with a developer for troubleshooting, your private conversations remain 100% private.

_Last updated: February 25, 2026_
