# 👮 Account & GDPR Management Flow

Privacy is more than just deleting messages; it's about navigating complex legal and platform-specific data policies. This document explains how **Discord Purge** facilitates deep data removal while respecting Discord's technical constraints and empowering the user's right to be forgotten.

---

## 🌍 Multi-Server Strategy

Our Rust backend is built for parallel efficiency. The `bulk_delete_messages` command accepts a `Vec<String>` of channel IDs, allowing it to:

1.  **Map channels** to their respective guilds or DMs using an internal registry.
2.  **Queue requests** per-rate-limit-bucket. Discord uses different buckets for global requests vs. specific channel requests.
3.  **Execute deletions** across multiple servers in a single logical operation from the user's perspective.
4.  **Progress Tracking**: Emits specialized events (`deletion_progress`) for every successful deletion, ensuring the UI stays in sync with reality.

---

## 🗑️ Profile Deletion Protocol

Direct programmatic account deletion is restricted by Discord for safety. We solve this by guiding the user through the verified official process:

### The Workflow:

1.  **Backend Integration**: The `open_discord_url_for_action(action_type)` command uses `tauri::api::shell::open` to launch the browser directly to the relevant settings page.
2.  **Safety First**: The Frontend triggers a high-severity warning modal.
3.  **Handoff**: Once the user confirms, the app opens the browser and **automatically logs the user out** of the application to prevent any session conflicts or data mismatches.

### Technical Implementation:

```rust
#[tauri::command]
pub async fn open_discord_url_for_action(action_type: String) -> Result<(), AppError> {
    let url = match action_type.as_str() {
        "account_deletion" => "https://discord.com/settings/account",
        "privacy_settings" => "https://discord.com/settings/privacy",
        _ => return Err(AppError::new("Invalid action type", "invalid_action")),
    };
    tauri::api::shell::open(&tauri::Config::default().shell.scope, url, None)
        .map_err(|_| AppError::new("Failed to open browser", "shell_error"))
}
```

---

## 📊 GDPR Data Request Assistance

We empower users to utilize their legal rights under the General Data Protection Regulation (GDPR):

- **Instructional GUI**: The app provides a dedicated "Data Request" portal within the "Privacy" mode.
- **Step-by-Step**: It provides clear instructions on downloading the official Discord Data Package (Settings > Data & Privacy).
- **Data Parsing (Roadmap)**: Future versions will include a local parser that reads your `messages.json` from the Discord data package and automatically identifies every channel ID where you have sent messages, making "Total Wipe" 100% automated.
- **Support Templates**: We provide templates for contacting Discord Support to request bulk message deletion using the channel IDs derived from their data package.

---

## 🛡️ Identity Persistence

User identities are managed via a local secure buffer.

- **OAuth2 IDs**: Saved using **OS-native encryption** (Keyring). We store the `access_token` and `refresh_token` but never the password.
- **Switching**: Fast account switching allows users to manage multiple digital identities without re-authenticating every time.
- **Wiping**: When a user logs out, we don't just clear the memory; we remove the secrets from the system keychain.

_Last updated: February 25, 2026_
