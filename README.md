# Discord Purge — Advanced Discord Message Deletion & Privacy Management Tool

> **Permanently delete Discord messages**, **bulk erase Discord history**, **mass clean DMs**, and **manage your Discord privacy** with this powerful, local-first desktop utility.

Please buy me a coffee! ☕ https://buymeacoffee.com/discordpurge

<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="Discord Purge Logo" width="128" height="128">
  <p><em>A high-performance Discord privacy management solution for users seeking full control over their message history and digital footprint.</em></p>

<!-- Project Badges -->

[![Release](https://img.shields.io/github/v/release/TourIT-NL/friendly-discord-bot?style=for-the-badge&color=7289da)](https://github.com/TourIT-NL/friendly-discord-bot/releases)
[![License](https://img.shields.io/github/license/TourIT-NL/friendly-discord-bot?style=for-the-badge&color=grey)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=for-the-badge)](https://github.com/TourIT-NL/friendly-discord-bot/releases)

<!-- CI/CD Workflow Badges -->

[![Build Status](https://img.shields.io/github/actions/workflow/status/TourIT-NL/friendly-discord-bot/main.yml?branch=main&style=for-the-badge)](https://github.com/TourIT-NL/friendly-discord-bot/actions)
[![Lint and Test](https://github.com/TourIT-NL/friendly-discord-bot/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/TourIT-NL/friendly-discord-bot/actions/workflows/main.yml)

<!-- Security Workflows Badges -->

[![CodeQL](https://github.com/TourIT-NL/friendly-discord-bot/actions/workflows/codeql-analysis.yml/badge.svg)](https://github.com/TourIT-NL/friendly-discord-bot/actions/workflows/codeql-analysis.yml)

<!-- Quality Workflows Badges -->

[![Spell Check](https://github.com/TourIT-NL/friendly-discord-bot/actions/workflows/spell-check.yml/badge.svg)](https://github.com/TourIT-NL/friendly-discord-bot/actions/workflows/spell-check.yml)

</div>

---

## 🔥 What Is Discord Purge?

**Discord Purge** is a robust, cross-platform **desktop application** engineered with **Rust** and **Tauri** to provide users with unparalleled control over their Discord presence. It functions as a comprehensive **Discord cleanup tool**, enabling **bulk message deletion**, efficient **DM cleanup**, and thorough **privacy management** for your Discord account. Designed for performance and security, it's the ultimate solution for anyone looking to **erase Discord chat history**, **delete all Discord messages**, or simply manage their digital footprint.

### Core Principles:

1.  **Unyielding Security**: Your credentials and data are protected using OS-level secure storage (Keychain, Credential Manager, Secret Service). No plaintext tokens or remote servers handling your data.
2.  **Peak Performance**: Powered by **Rust**, the application delivers fast, rate-aware operations for large-scale **Discord message deletion**, ensuring efficiency without triggering API limits.
3.  **Total User Empowerment**: Clear, intuitive interfaces combined with powerful, reliable tools for **Discord privacy cleanup** and **account hygiene**.

---

## Why Choose Discord Purge?

- **Local-First Processing**: All operations are executed directly on your machine, guaranteeing your data never leaves your control.
- **Secure Authentication**: Utilizes Discord OAuth2 for secure login, never requiring your password.
- **Comprehensive Cleanup**: Beyond messages, manage servers, friends, and block lists effectively.
- **Designed for Scale**: Efficiently handles years of Discord history and thousands of messages.

---

## 🚀 Core Features

- **Bulk Discord Message Deletion**: Effortlessly **delete Discord messages** from multiple channels, private DMs, and group chats. Features rate-limit aware processing and parallelized Rust dispatcher for speed and safety.
- **Mass Server & Connection Cleanup**: Quickly **leave multiple Discord servers** while maintaining whitelists, and efficiently manage bulk friend removal or block list resets.
- **Advanced Discord Privacy Tools**: Specialized utilities for identity cleanup, integration and webhook inspection, and profile masking to enhance your **Discord privacy**.

---

## 📦 Installation

Download the latest stable release for your operating system (Windows, macOS, Linux) directly from the [Discord Purge GitHub Releases page](https://github.com/TourIT-NL/friendly-discord-bot/releases).

- **Windows**: Download and run the `.msi` or `.exe` installer.
- **macOS**: Download the `.dmg` file, then drag the app to your Applications folder.
- **Linux**: Download the `.AppImage` or `.deb` package. For AppImage, make it executable (`chmod +x YourApp.AppImage`) and run it.

---

## 🧑‍💻 Usage

1.  Launch the **Discord Purge desktop application**.
2.  Securely authenticate using your Discord account via the OAuth2 flow.
3.  Select the desired **Discord cleanup tool** (e.g., message deletion, server leaving).
4.  Configure the scope of the operation (e.g., specific channels, date ranges).
5.  Review the proposed actions carefully and execute the operation. Monitor progress through the transparent logging.

---

## 🛠 Developer Setup

### Prerequisites

- Node.js v20+
- Rust (latest stable toolchain)
- Operating System build tools (refer to [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) for your specific OS).

### Setup

```bash
# Clone the Discord Purge repository
git clone https://github.com/TourIT-NL/friendly-discord-bot.git

# Navigate into the project directory
cd friendly-discord-bot

# Install frontend (npm) dependencies
npm install

# Run the application in development mode
npm run tauri dev
```

---

## ⚙️ GitHub Workflows

This project utilizes a comprehensive suite of GitHub Actions to ensure code quality, security, and efficient development:

- **CI/CD Pipeline** (`.github/workflows/main.yml`): Executes continuous integration checks including linting, testing (frontend and backend), and building multi-platform application binaries. Releases are automatically drafted and published upon new tags.
- **CodeQL Analysis** (`.github/workflows/codeql-analysis.yml`): Performs advanced static code analysis to proactively identify potential security vulnerabilities in both Rust and TypeScript/JavaScript code.
- **Spell Check** (`.github/workflows/spell-check.yml`): Maintains professionalism and accuracy by checking for spelling errors across the entire codebase and documentation.
- **Stale Issues & PRs** (`.github/workflows/stale.yml`): Automatically identifies and closes inactive issues and pull requests after a period of inactivity, keeping the repository tidy and focused.
- **Dependabot** (`.github/dependabot.yml`): Automates the process of keeping dependencies up-to-date for both Rust (`Cargo`) and npm, reducing security risks and maintenance overhead.
- **Release Drafter** (`.github/workflows/release-drafter.yml`): Streamlines the release process by automatically drafting comprehensive release notes based on merged pull requests.
- **Pull Request Labeler** (`.github/workflows/labeler.yml`): Applies relevant labels to pull requests based on the types of files changed, aiding in quicker review and categorization.
- **First Interaction Welcome** (`.github/workflows/first-interaction.yml`): Fosters a welcoming community by automatically greeting new contributors on their first issue or pull request, providing helpful resources.
- **Automated Backport** (`.github/workflows/backport.yml`): Facilitates efficient maintenance of multiple release lines by automating the backporting of fixes or features to older, supported branches.

---

## 📚 Documentation & Support

- **Project Wiki**: For in-depth technical documentation, detailed architectural diagrams, comprehensive user stories, and more, please visit the [Discord Purge Wiki](https://github.com/TourIT-NL/friendly-discord-bot/wiki).
- **Support**: For general questions, bug reports, or feature requests, please refer to our [SUPPORT.md](SUPPORT.md) file for guidelines on how to get help.
- **Contributing**: We welcome contributions from the community! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on how to contribute.
- **Security**: If you discover a security vulnerability, please refer to [SECURITY.md](SECURITY.md) for instructions on responsible disclosure.

---

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for full details.

---

<div align="center">
  <em>Empowering Discord users with advanced tools for message deletion, privacy management, and digital footprint control.</em>
</div>
