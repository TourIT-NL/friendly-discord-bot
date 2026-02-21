# Discord Purge: Privacy Enforcement Unit

<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="Discord Purge Logo" width="128" height="128">
  <p><em>High-performance Privacy Enforcement for the Modern Discord User.</em></p>

  [![Release](https://img.shields.io/github/v/release/evuldeeds-design/Discord-Purge?style=for-the-badge&color=7289da)](https://github.com/evuldeeds-design/Discord-Purge/releases)
  [![License](https://img.shields.io/github/license/evuldeeds-design/Discord-Purge?style=for-the-badge&color=grey)](LICENSE)
  [![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=for-the-badge)](https://github.com/evuldeeds-design/Discord-Purge/releases)
  [![Build Status](https://img.shields.io/github/actions/workflow/status/evuldeeds-design/Discord-Purge/main.yml?branch=main&style=for-the-badge)](https://github.com/evuldeeds-design/Discord-Purge/actions)
</div>

---

Discord Purge is a high-performance, high-security desktop utility designed for deep Discord privacy management. Built with **Rust** and **Tauri**, it provides ultimate control over your social footprint with unyielding security and peak performance.

## ✨ Key Protocols

*   🛡️ **Official Gate (OAuth2)**: Secure, standard authorization for managing public guilds and profile data.
*   🔓 **Bypass Mode (User Token)**: High-level access for private buffers including DMs, group chats, and bulk relationship severance.
*   🤝 **Local Handshake (RPC)**: Zero-config rapid link using your active Discord desktop process.
*   📱 **QR Signature**: Secure mobile-bridge login via Discord's remote auth gateway.

## 🚀 Core Features

*   **Bulk Message Deletion**: High-speed, rate-limit aware purging of messages across multiple channels and servers simultaneously.
*   **Connection Severance**: Rapidly leave multiple servers at once while maintaining a whitelist of essential nodes.
*   **Identity Purge**: Bulk relationship severance (friends/blocks) to clear your social footprint.
*   **Engine Tools**:
    *   **Audit Log Burial**: Cyclic node renames to flood and mask server audit history.
    *   **Webhook Ghosting**: Detection and removal of identity-linked integrations.
    *   **Stealth Wipes**: Automated profile masking (status, DMs, presence).

## 🔒 Security Architecture

*   **OS Vault Integration**: Sensitive tokens and application secrets are stored exclusively in the host OS keychain (Windows Credential Manager / macOS Keychain). No plain-text secrets reside on disk.
*   **Rate Limit Engine**: A granular, multi-threaded Rust dispatcher ensures your account remains safe with exponential backoff and speculative bucket tracking.
*   **Transparency**: A real-time **System Protocol Log** provides a deep technical trace of every handshake and API interaction.

## 📦 Installation

Download the latest production build for your platform from the [Releases](https://github.com/evuldeeds-design/Discord-Purge/releases) page.

### Windows
1.  Download `.msi` or `.exe`.
2.  Install and launch `Discord Purge`.

### macOS
1.  Download `.dmg`.
2.  Drag `Discord Purge` to your Applications folder.

### Linux
1.  Download `.AppImage` or `.deb`.
2.  `chmod +x` the AppImage and execute.

## 🛠 Developer Setup

### Prerequisites
*   Node.js (v20+)
*   Rust (latest stable)
*   Build tools for your OS (see [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

### Setup
```bash
# Clone the repository
git clone https://github.com/evuldeeds-design/Discord-Purge.git

# Navigate to the project directory
cd discord-privacy-util

# Install dependencies
npm install

# Launch in Development Mode
npm run tauri dev
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.

---
<div align="center">
  <em>Created for the Privacy Enforcement Unit.</em>
</div>
