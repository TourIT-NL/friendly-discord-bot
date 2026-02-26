# Future Development & Optimization Roadmap

This document outlines potential mistakes, architectural improvements, and additional functions for the Discord Privacy and Cleanup Utility, based on a comprehensive analysis of the current codebase and Discord API protocols.

## 1. Authentication Protocols

### ⚠️ Potential Mistakes & Risks

- **Fixed OAuth2 Callback Port (58123):** Currently, the application binds to a fixed port. If another application or instance is using this port, authentication will fail.
  - _Improvement:_ Implement a port range fallback or dynamic port detection (though Discord requires exact Redirect URI matching, multiple URIs can be registered in the Developer Portal).
- **Static `x-super-properties`:** The rate limiter uses a hardcoded base64 string for Discord's internal tracking header. This can become a "fingerprint" for the app if Discord updates their client tracking logic.
  - _Improvement:_ Generate these properties dynamically based on the current environment or a periodically updated configuration.
- **Hardcoded User-Agent:** Using a static User-Agent (currently `Chrome/144.0.0.0`) for all users and identities makes the application's traffic easily identifiable.
  - _Improvement:_ Implement a pool of realistic User-Agents or allow the user to specify one.

### 🚀 Additional Functions

- **OAuth2 Token Rotation:** Implement automatic `refresh_token` handling to maintain long-term access without re-authentication for OAuth2 identities.
- **Multi-Factor Authentication (MFA) Support:** For direct token logins, provide a UI flow to handle 2FA/MFA if the token requires a fresh session validation.
- **Session Management:** Add a function to list and revoke other active Discord sessions (using the `/science` or settings endpoints where available).

## 2. API Interaction & Rate Limiting

### ⚠️ Potential Mistakes & Risks

- **Linear Message Purging ($O(N)$):** The current `bulk_delete_messages` scrolls through every message in a channel to find yours. This is extremely slow for high-volume channels.
  - _Improvement:_ Utilize Discord's Search API (`/channels/{id}/messages/search?author_id=...`) to directly target only your messages. This is significantly faster and reduces API overhead.
- **Lack of Proxy Support:** Privacy-focused users may want to route their traffic through Tor or a SOCKS5 proxy.
  - _Improvement:_ Add proxy configuration to the `reqwest` client in the `RateLimiterActor`.
- **Synchronous Operation Processing:** While the backend is async, the operations themselves are mostly sequential per channel.
  - _Improvement:_ Allow a configurable "Concurrency Level" to process multiple channels/nodes simultaneously, balanced by the global rate limiter.

### 🚀 Additional Functions

- **HTTP/3 (QUIC) Support:** Enable HTTP/3 in the client to better mimic modern browser behavior and potentially improve performance.
- **Advanced Jitter Logic:** Move from simple random ranges to more sophisticated "human-like" delays (e.g., longer pauses after a certain number of actions, mimicking "fatigue").

## 3. Cleanup & Privacy Operations

### ⚠️ Potential Mistakes & Risks

- **Server Ownership Blindness:** `bulk_leave_guilds` does not check if the user is the owner. Leaving a server you own is not possible through the "leave" endpoint (it requires deletion or transfer).
  - _Improvement:_ Detect ownership and provide options to "Delete Server" or "Transfer & Leave".
- **Reaction Purge Efficiency:** Reactions are currently removed one by one.
  - _Improvement:_ If the user has "Manage Messages" permission in a channel, use the "Delete All Reactions" endpoint for a specific message to speed up the process.

### 🚀 Additional Functions

- **Hypesquad Management:** Add functionality to change or remove Hypesquad house membership.
- **Relationship Cleanup:**
  - Bulk cancel outgoing friend requests.
  - Bulk ignore/clear incoming friend requests.
  - Bulk unfriend/block users based on "Last Interacted" or "Common Servers" (requires caching).
- **Privacy Settings Bulk Update:**
  - Toggle "Allow Direct Messages from Server Members" for all servers.
  - Update friend request permissions (Everyone, Friends of Friends, etc.).
- **Authorized Apps Audit:** List and revoke OAuth2 applications authorized on the account.
- **Profile "Ghosting":** Bulk clear bio, custom status, and remove avatar/banner in one click.

## 4. Security & Vault Architecture

### ⚠️ Potential Mistakes & Risks

- **Master Password Absence:** The vault is currently encrypted with a key stored in the OS keyring or a fallback file. This means anyone with access to the OS account can access the tokens.
  - _Improvement:_ Implement an optional Master Password that derives a key (using Argon2/PBKDF2) to encrypt the vault key, ensuring tokens are safe even if the physical files are accessed.
- **Plaintext Fallback Key:** If the keyring is unavailable, the fallback key is stored in a file. If this file is unencrypted, the entire "Vault" security is bypassed.
  - _Improvement:_ Encrypt the fallback file with a hardware-bound ID or a user password.

### 🚀 Additional Functions

- **Memory Hardening:** Ensure tokens are zeroed out in memory (`Zeroize` trait) after use to prevent leakage in memory dumps.
- **Identity Isolation:** Ensure that rate limits and headers are strictly separated per identity to prevent "cross-account contamination" if one account is flagged.

## 5. User Experience & Advanced Features

### 🚀 Additional Functions

- **GDPR Data Package Integration:**
  - Allow users to upload their Discord `data.json` package.
  - The app can then automatically find every channel, DM, and server the user has ever been in, even those that aren't currently visible in the UI.
- **Pre-Cleanup Backup:** Offer an option to export messages/history to HTML or JSON before they are permanently deleted.
- **"Nuclear Option":** A one-click button to trigger all cleanup operations (Messages, Servers, Relationships, Profile) for a complete digital footprint wipe.
- **Operation Scheduling:** Allow users to schedule periodic cleanups (e.g., "Delete messages older than 30 days every Sunday").

## 6. Project Architecture Improvements

- **Plugin System for Operations:** Refactor operations into a trait-based system where new cleanup modules (e.g., "Audit Log Cleaner", "Webhook Manager") can be added without modifying the core `op_manager`.
- **Protobuf for Internal Messaging:** Consider using Protobuf for the communication between the UI and the Rate Limiter Actor to ensure type safety and performance for high-frequency progress updates.
- **Enhanced Logging:** Implement a "Redaction Layer" in the logger to ensure no part of a token or sensitive email is ever accidentally written to the `app.log`.
