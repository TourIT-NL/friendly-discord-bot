# 👮 Account & GDPR Management Flow

Privacy is more than just deleting messages; it's about navigating complex legal and platform-specific data policies. This document explains how **Discord Purge** facilitates deep data removal while respecting Discord's technical constraints.

---

## 🌍 Multi-Server Strategy

Our Rust backend is built for parallel efficiency. The `bulk_delete_messages` command accepts a `Vec<String>` of channel IDs, allowing it to:

1.  Map channels to their respective guilds or DMs.
2.  Queue requests per-rate-limit-bucket.
3.  Execute deletions across multiple servers in a single logical operation from the user's perspective.

---

## 🗑️ Profile Deletion Protocol

Direct programmatic account deletion is restricted by Discord for safety. We solve this by guiding the user through the verified official process:

1.  **Backend Integration**: The `open_discord_url_for_action(action_type)` command uses `tauri::api::shell::open` to launch the browser directly to the relevant settings page.
2.  **Safety First**: The Frontend triggers a high-severity warning modal.
3.  **Handoff**: Once the user confirms, the app opens the browser and **automatically logs the user out** of the application to prevent any session conflicts.

---

## 📊 GDPR Data Request Assistance

We empower users to utilize their legal rights under the General Data Protection Regulation (GDPR):

- **Instructional GUI**: The app provides a dedicated "Data Request" portal.
- **Step-by-Step**: It provides clear instructions on downloading the official Discord Data Package (Settings > Data & Privacy).
- **Next Level Cleanup**: We provide templates for contacting Discord Support to request bulk message deletion using the channel IDs derived from their data package.

---

## 🛡️ Identity Persistence

User identities are managed via a local secure buffer.

- **OAuth2 IDs**: Saved using OS-native encryption.
- **Switching**: Fast account switching allows users to manage multiple digital identities without re-authenticating every time.

_Last updated: February 25, 2026_
