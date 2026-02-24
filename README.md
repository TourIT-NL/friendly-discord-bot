# ✨ Friendly Discord Bot - Your Ultimate Discord Privacy & Data Cleanup Utility ✨

[![Release](https://img.shields.io/badge/release-v1.0.3-blue)](https://github.com/TourIT-NL/friendly-discord-bot/releases)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/TourIT-NL/friendly-discord-bot)
[![Discord Server](https://img.shields.io/discord/YOUR_DISCORD_SERVER_ID?label=Discord&logo=discord&color=7289DA)](https://discord.gg/YOUR_INVITE_LINK) <!-- Placeholder: Replace with actual Discord invite -->
[![GitHub Stars](https://img.shields.io/github/stars/TourIT-NL/friendly-discord-bot?style=social)](https://github.com/TourIT-NL/friendly-discord-bot/stargazers)

**Friendly Discord Bot** is the secure, privacy-focused, and incredibly user-friendly desktop application designed for comprehensive Discord data management. Effortlessly **delete Discord messages**, purge DMs, clean up server content, manage relationships, and wipe profile data with **intelligent rate-limit handling** and **local-only processing**. Take back control of your digital footprint and enhance your Discord privacy today!

---

## 📖 Table of Contents

- [🌟 Overview](#-overview)
- [🚀 Why Choose Friendly Discord Bot?](#-why-choose-friendly-discord-bot)
- [💎 Key Features](#-key-features)
- [⚡ Quick Start: How It Works](#-quick-start-how-it-works)
- [🔒 Security and Privacy - Our Core Promise](#-security-and-privacy---our-core-promise)
- [⬇️ Installation for End Users](#️-installation-for-end-users)
- [🗺️ Roadmap & Future Vision](#️-roadmap--future-vision)
- [🤝 Community & Support](#-community--support)
- [🛠️ Developer & Project Information](#️-developer--project-information)

---

## 🌟 Overview

**Friendly Discord Bot** is an open-source, cross-platform desktop application built with the robust power of **Rust** 🦀, the modern UI capabilities of **Tauri** ✨, and the dynamic frontend of **TypeScript** 🚀. It empowers Discord users with unparalleled control to safely and efficiently **remove their Discord chat history and personal data**. Engineered for both **usability and unyielding security**, it intelligently minimizes account risk through advanced rate-limit aware dispatching and conservative defaults.

This powerful utility is your definitive solution to:

- **Delete all Discord messages in bulk** from Direct Messages, group chats, and server channels.
- **Remove Discord history safely** and securely, without ever exposing your sensitive tokens to third-party servers.
- **Master your Discord privacy** by purging old, unwanted data and easily departing from multiple servers.

This repository serves as the central hub for our project, providing source code, ready-to-use release artifacts, comprehensive documentation, and a vibrant community space for users, contributors, and security researchers.

---

## 🚀 Why Choose Friendly Discord Bot?

At the heart of **Friendly Discord Bot** lie three core tenets: **Unyielding Security**, **Peak Performance**, and **Total User Empowerment**.

### The Discord Data Dilemma 🧐

Discord, while fantastic for communication, lacks a centralized, user-friendly tool to comprehensively manage and remove personal history across DMs, servers, and profile fields. Manual deletion is a tedious, error-prone, and often impossible endeavor at scale, leaving many users feeling helpless about their digital footprint.

### Our Elegant Solution ✅

**Friendly Discord Bot** provides an intuitive, **GUI-driven workflow** that puts you in command. With powerful preview filters, secure defaults, and multiple login options (including the highly recommended **OAuth2**), even non-technical users can effortlessly **reclaim their privacy**. We ensure your data never leaves your machine, processing everything locally without the need to expose raw tokens or upload private information to external servers. This makes us the #1 **Discord privacy utility** for comprehensive **Discord data removal**.

### Our Unique Positioning 🏆

This project is the **only open-source desktop application** specifically designed to be **user-friendly** while uncompromisingly focusing on **privacy-first deletion** of Discord data. We are committed to transparency, security, and performance.

---

## 💎 Key Features

Empower yourself with precise, secure control over your Discord data. Each feature is crafted to be powerful yet intuitive.

- **🗑️ Bulk Message Deletion**: Effortlessly **delete Discord messages** in mass from Direct Messages, group chats, and selected server channels. _(MVP: End-to-end Bulk Message Deletion)_
  - **Advanced Filtering**: **Purge DMs and group chats** with intelligent filters by specific date ranges, by user, by channel, and even by keyword.
  - **Targeted Cleanup**: Select precisely which conversations or channels you wish to clear.
- **👋 Bulk Server Departure**: Easily **leave multiple Discord servers at once**, with the crucial option to **delete all your messages** in those servers _before_ departing. _(MVP: End-to-end Bulk Server Departure)_
  - **Whitelist Functionality**: Maintain a whitelist of servers you wish to remain in.
- **👤 Relationship Management**: **Clean up your Discord account** by purging unwanted entries from your friends list, blocked list, and pending requests.
- **🧹 Profile Data Wipe**: Take control of your public persona by clearing sensitive or outdated information from your Discord profile fields and custom statuses.
- **🛡️ Intelligent Rate-Limiting**: Features a sophisticated, **Discord API rate-limit aware dispatcher** with exponential backoff and randomized jitter to safeguard your account from suspensions.
- **🔐 Secure Authentication**: Supports **multiple login modes** for flexibility and security:
  - **OAuth2 (Recommended)**: The most secure method, allowing you to log in via Discord's official consent screen without ever exposing your password or token. _(MVP: Secure Discord OAuth2 Login)_
  - **Token Mode**: Available for advanced users who understand the associated risks.
  - **RPC & QR Login**: Convenient alternatives for specific use cases.
- **🏠 Local-Only Processing**: Guarantees **no cloud storage** of your sensitive tokens or message content. All deletion operations are executed directly on your local machine, ensuring maximum privacy.
- **🔑 OS Keychain Storage**: Your Discord credentials are encrypted and stored securely within your operating system's native keychain on Windows, macOS, and Linux.
- **👀 Preview and Dry Run**: **Safely preview changes** before any destructive actions are executed. See exactly what will be deleted, giving you full confidence and control.
- **📊 Comprehensive Reporting**: Provides real-time UI progress updates, detailed status information, and local logs for transparent auditing of all operations.
- **🌐 Open Source & Auditable**: Our entire codebase is publicly available under the MIT license, fostering transparency, community contributions, and independent security reviews.

---

## ⚡ Quick Start: How It Works

Begin your journey to a cleaner, more private Discord presence with **Friendly Discord Bot** today!

1.  **⬇️ Download**: Grab the latest release for your operating system (Windows, macOS, Linux) directly from our [**Releases page**](https://github.com/TourIT-NL/friendly-discord-bot/releases).
2.  **📦 Install**:
    - **Windows/macOS**: Run the provided installer or mount the DMG and drag to Applications.
    - **Linux**: Execute the AppImage or install via your preferred distribution package.
3.  **🚀 Run**: Launch the application. You'll be greeted with an intuitive interface.
4.  **🔒 Login Securely**: Choose **OAuth2 (recommended)** to securely connect your Discord account. This guides you through Discord's official consent process, ensuring your token remains safe.
5.  **🔍 Scan & Filter**: The app will intelligently scan your accessible DMs, servers, and relationships. Utilize powerful filters (date, user, channel, keyword) to precisely define your cleanup scope, like "delete Discord messages from before 2023."
6.  **👀 Preview (Dry Run)**: **Crucial step!** Always run a preview first. The app will show you exactly what will be deleted without making any permanent changes. This is your safety net.
7.  **🔥 Execute**: Once you're confident with the preview, confirm and initiate the purge. Our smart dispatcher will handle all API interactions, respecting Discord's rate limits automatically.

---

## 🔒 Security and Privacy - Our Core Promise

Our design is rooted in **Unyielding Security** and a commitment to your privacy. We believe you should have total control over your digital footprint.

### Core Design Principles

- **🛡️ Local Processing Only**: **Your data stays local.** All operations are performed exclusively on your machine. We **never** upload message content, Discord tokens, or any other sensitive data to our servers or any third-party cloud service.
- **🔐 OS Keychain Integration**: Your valuable Discord credentials (obtained via OAuth2) are encrypted and stored using your operating system's native, highly secure keychain mechanism.
- **🐢 Conservative Defaults**: To protect your Discord account, our rate limits and retry logic are set conservatively by default, significantly reducing the risk of account flags or temporary suspensions.
- **🔎 Open Source Transparency**: Full transparency is paramount. Our entire codebase is open to public scrutiny, enabling security researchers and the community to audit our practices and build trust.
- **🚫 No Telemetry by Default**: Your usage data is private. Telemetry is strictly opt-in, meaning it's disabled unless you explicitly grant consent.

### Advanced Security Safeguards

- **🚨 Adaptive Rate Limit Detection**: Our system intelligently monitors Discord API rate limit headers and dynamically adjusts its pacing to ensure smooth, uninterrupted operation without hitting API limits.
- **🎲 Intelligent Retry Logic**: Implements robust exponential backoff with randomized jitter, a proven strategy to manage transient API errors and avoid synchronized retries that could trigger further rate limits.
- **🔑 Token Safety Guidance**: While a token mode exists for expert users, we strongly advocate for **OAuth2**. This method eliminates the need for you to handle or store raw tokens, significantly reducing security risks.
- **📜 Local & Clearable Logs**: All operational logs detailing activity are stored exclusively on your local machine. You have full control to review and clear them at any time.
- **Minimal Permissions**: We adhere to the principle of least privilege, ensuring the application requests only the precise Discord API permissions absolutely necessary for the operations you choose to perform.

---

## ⬇️ Installation for End Users

Get the **Friendly Discord Bot** running on your system quickly and easily.

### 🪟 Windows

1.  Download the latest `friendly-discord-bot-setup.exe` from the [**Releases page**](https://github.com/TourIT-NL/friendly-discord-bot/releases).
2.  Run the installer and follow the intuitive on-screen prompts.
3.  Launch the application directly from your Start Menu.

### 🍎 macOS

1.  Download the latest `friendly-discord-bot.dmg` from the [**Releases page**](https://github.com/TourIT-NL/friendly-discord-bot/releases).
2.  Mount the DMG file and simply drag the "Friendly Discord Bot" application icon into your Applications folder.
3.  Open the application from your Applications folder.

### 🐧 Linux

1.  Download the AppImage or a suitable distribution package for your system from the [**Releases page**](https://github.com/TourIT-NL/friendly-discord-bot/releases).
2.  **For AppImage**:
    - Make the downloaded file executable: `chmod +x FriendlyDiscordBot.AppImage`
    - Run it directly: `./FriendlyDiscordBot.AppImage`
    - _(Alternatively, install via a provided distribution package if available for your distro.)_

---

## 🗺️ Roadmap & Future Vision

We are committed to continuously improving **Friendly Discord Bot** to provide even greater control and privacy features. Here's a glimpse into our exciting future:

- **Keyword Filtering Enhancements**: More granular control over message deletion with advanced keyword matching and exclusion rules.
- **Scheduled Cleanup Tasks**: Automate recurring cleanup operations for ongoing Discord data management.
- **GDPR Data Package Integration**: Tools to help users interpret and manage data from Discord's GDPR data packages. _(Guidance for official GDPR Data Request/Profile Deletion is already in place as per [Project Blueprint](https://github.com/TourIT-NL/friendly-discord-bot/wiki/Architecture-Account-GDPR#34-account--gdpr-management-flow))_
- **Customizable Theming**: Personalize your application's look and feel.
- **Plugin/Extension System**: Empower the community to build and share their own tools and integrations.

Stay tuned for updates and contribute your ideas in our [GitHub Discussions](https://github.com/TourIT-NL/friendly-discord-bot/discussions)!

---

## 🤝 Community & Support

We thrive on community involvement and are here to help you every step of the way!

### Official Channels

- **💬 GitHub Discussions**: Your primary hub for support, asking questions, sharing ideas, and requesting new features.
- **📢 Official Discord Server**: Join our real-time community for quick help, discussions, and direct interaction with the development team. [Join now!](https://discord.gg/YOUR_INVITE_LINK) <!-- Placeholder: Replace with actual Discord invite -->
- **📚 Subreddit (r/FriendlyDiscordBot)**: Follow us for announcements, user stories, and community content.
- **▶️ YouTube Channel**: Watch our tutorials, demos, and feature showcases.

### How to Get Help

1.  **Consult the FAQ & Wiki**: Many common questions are answered in our [Frequently Asked Questions](#frequently-asked-questions-faq) and comprehensive [Wiki](https://github.com/TourIT-NL/friendly-discord-bot/wiki).
2.  **Open an Issue**: For bugs or feature requests, please open a detailed issue on GitHub, including logs and reproduction steps.
3.  **Ask in Discord**: For quick questions or real-time assistance, our Discord community is always ready to help!

---

## 🛠️ Developer & Project Information

This section is dedicated to those who wish to delve deeper into the project, contribute, or understand its inner workings.

### Detailed Usage Guides

Explore specific, step-by-step workflows for various cleanup operations. These guides provide more granular control and understanding:

- [**Bulk Message Deletion (US-002)**](https://github.com/TourIT-NL/friendly-discord-bot/wiki/User-Stories#22-bulk-message-deletion-us-002)
- [**Bulk Server Departure (US-003)**](https://github.com/TourIT-NL/friendly-discord-bot/wiki/User-Stories#23-bulk-server-departure-us-003)
- [**Account & GDPR Management Flow**](https://github.com/TourIT-NL/friendly-discord-bot/wiki/Architecture-Account-GDPR#34-account--gdpr-management-flow)

### Advanced Usage & Integration

Unlock the full potential of Friendly Discord Bot:

- **Token Mode**: For advanced users who fully understand the associated security risks and require direct token access.
- **RPC Mode**: Facilitates integration with local client applications for custom workflows.
- **Scripting**: Leverage our CLI helper for powerful, automated cleanup tasks for power users.
- **Custom Rate Profiles**: Fine-tune API request rates for unique scenarios or specific Discord account histories.
- **Headless Mode**: Implement automated workflows (an advanced, opt-in feature).

### Screenshots and Demo

Help us visualize the power of **Friendly Discord Bot**!

- **Screenshots**: Place your high-quality screenshots in the `/assets/screenshots` directory and link them here.
- **Demo GIF**: Include a short, compelling GIF that showcases a purge preview and the real-time progress bar.
- **Demo Video**: Link a 60-second demo video from YouTube or your website, highlighting key features and ease of use.
- _(**Pro-Tip for SEO**: Ensure all images have descriptive `alt` text for accessibility and better search engine indexing!)_

### Comparison with Alternatives

Transparency and informed choice are vital. See how **Friendly Discord Bot** stands out:

**Why Friendly Discord Bot is the definitive choice for secure Discord data deletion:**

| Attribute            | Friendly Discord Bot | Manual deletion | Browser scripts & bots |
| :------------------- | :------------------: | :-------------: | :--------------------: |
| GUI                  |      **✅ Yes**      |      ❌ No      |       🤔 Varies        |
| Bulk DM deletion     |      **✅ Yes**      |      ❌ No      |       ⚠️ Limited       |
| Rate-limit aware     |      **✅ Yes**      |       N/A       |      ❌ Often no       |
| Secure local storage |      **✅ Yes**      |       N/A       |      ❌ Often no       |
| Open source          |      **✅ Yes**      |       N/A       |       🤔 Varies        |
| Preview & dry run    |      **✅ Yes**      |      ❌ No      |       🤔 Varies        |
| Cross platform       |      **✅ Yes**      |       N/A       |       🤔 Varies        |

**Feature-Level Comparison:**

| Feature               | Friendly Discord Bot | Other GUI tools | CLI scripts |
| :-------------------- | :------------------: | :-------------: | :---------: |
| Preview before delete |      **✅ Yes**      |    🤔 Varies    |    ❌ No    |
| OAuth2 recommended    |      **✅ Yes**      |     ❌ Rare     |   ❌ Rare   |
| OS keychain storage   |      **✅ Yes**      |     ❌ Rare     |    ❌ No    |
| Rate limit handling   |      **✅ Yes**      |    🤔 Varies    |   ⚠️ Poor   |
| Community support     |      **✅ Yes**      |    🤔 Varies    |  🤔 Varies  |

**Detailed Capability Matrix:**

| Capability             | Friendly Discord Bot | Browser extensions | Manual | Third-party cloud services |
| :--------------------- | :------------------: | :----------------: | :----: | :------------------------: |
| Delete DMs in bulk     |          ✅          |         ❌         |   ❌   |             ❌             |
| Delete server messages |          ✅          |         ❌         |   ❌   |             ❌             |
| Leave servers in bulk  |          ✅          |         ❌         |   ❌   |             ❌             |
| Wipe profile fields    |          ✅          |         ❌         |   ❌   |             ❌             |
| Local processing only  |          ✅          |         ❌         |   ✅   |             ❌             |
| Rate limit aware       |          ✅          |         ❌         |  N/A   |             ❌             |
| Signed releases        |          ✅          |       Varies       |  N/A   |           Varies           |

### Developer Build and Contribution

We welcome and encourage contributions from the community! Please refer to our `CONTRIBUTING.md` for detailed guidelines, code style, and the pull request process.

**Prerequisites:**

- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable) 🦀
- [Node.js LTS](https://nodejs.org/en/) 🟢
- [Yarn](https://yarnpkg.com/) or npm 📦
- [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) for your specific platform 🖥️

**Build Steps:**

```bash
# Clone the repository
git clone https://github.com/TourIT-NL/friendly-discord-bot.git
cd friendly-discord-bot/discord-privacy-util # Navigate to the project root

# Install frontend dependencies
yarn install # or npm install

# Build frontend (Vite)
yarn build # or npm run build

# Build backend (Rust)
cargo build --release --workspace # Build the entire workspace in release mode

# Package the desktop application
yarn tauri build # or npm run tauri build
```

_(Note: Use `cargo build --release --workspace` for a complete release build across the Rust workspace.)_

### Testing and Quality Assurance

Ensuring the stability and security of **Friendly Discord Bot** is paramount.

- **Unit Tests**: Robust tests for core logic, dispatcher, and utility functions.
- **Integration Tests**: Comprehensive tests for Discord API interactions, often using mocked endpoints to simulate real-world scenarios.
- **End-to-End Tests**: UI flows are rigorously tested using headless automation to simulate user interactions.
- **Security Tests**: Regular static analysis, dependency scanning (via `cargo audit` and `npm audit`), and adherence to secure coding practices.
- **Manual QA**: Extensive manual testing is performed across Windows, macOS, and Linux using dedicated test accounts.

### Troubleshooting

Encountering an issue? Here's how to get back on track:

- For common issues, fixes, and detailed logging information, please consult our comprehensive [**Troubleshooting Guide**](SUPPORT.md) or explore our [**Project Wiki**](https://github.com/TourIT-NL/friendly-discord-bot/wiki).
- **Common Issues**:
  - **Login fails**: Verify OAuth2 redirect configuration and ensure your system time is accurate.
  - **Rate limit errors**: Utilize the conservative rate profile and ensure exponential backoff is enabled.
  - **Missing messages in preview**: Confirm account access to the conversation and correct app permissions.
  - **App crashes**: Check local logs (`%APPDATA%/FriendlyDiscordBot/logs` on Windows, `~/Library/Logs/FriendlyDiscordBot` on macOS, `~/.config/FriendlyDiscordBot/logs` on Linux) and open a detailed issue on GitHub.

### Frequently Asked Questions (FAQ)

- **Is this safe for my account?** While designed with security in mind (OAuth2, rate-limiting), no tool can guarantee zero risk due to platform ToS. Use at your own discretion, and follow our safety checklist.
- **Does the app store my messages?** Absolutely not. Messages are processed locally and never uploaded or stored remotely.
- **Will Discord ban me?** We cannot provide a guarantee. Our app minimizes risk through careful API interaction, but we recommend discretion.
- **Can I recover deleted messages?** No. Once deleted via the app, messages cannot be recovered. Always back up what you wish to keep!
- **Is the app open source?** Yes, it's MIT licensed. Contributions are highly encouraged!
- **How do I report a security issue?** Please follow the responsible disclosure process in our `SECURITY.md` file.

### Legal and Risk Disclaimer

**Important**: The use of this software may potentially violate Discord's terms of service or community guidelines. The project maintainers are not responsible for any account actions (e.g., temporary suspension or permanent ban) taken by Discord. Users assume full responsibility for their actions and must ensure compliance with all relevant local laws and Discord's platform terms.

**No Warranty**: This software is provided "as-is" without any warranty of any kind, express or implied.

### Responsible Disclosure

We take security seriously. If you discover a vulnerability, please adhere to the process outlined in our `SECURITY.md` file. Provide a clear and concise reproduction path, and kindly refrain from public disclosure until a fix is available and deployed. The maintainers will acknowledge receipt promptly and provide a timeline for remediation.
