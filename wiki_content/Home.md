# 🏠 Discord Purge Official Wiki: The Ultimate Privacy Hub

Welcome to the official **Discord Purge Wiki**, your comprehensive guide to the definitive **desktop application** for advanced **Discord message deletion**, massive **server departures**, and total **digital footprint management**.

Built with the blazing speed of **Rust** 🦀 and the modern security of **Tauri** ✨, this utility is engineered for users who demand the highest standards of privacy, performance, and transparency. This Wiki serves as the living blueprint for our project, detailing everything from high-level vision to deep architectural specifics.

---

## 🧭 Navigation Guide

### 🚀 Getting Started

- **[User Stories](./User-Stories)**: Understand the core features and how they solve real-world privacy problems.
- **[Project Management](./Project-Management)**: Review our scope, current MVP status, and development phasing.
- **[CI/CD Pipeline](./CI-CD-Pipeline)**: Explore how we automate builds, tests, and releases to maintain elite quality.
- **[FAQ & Glossary](./FAQ-Glossary)**: Clear answers to common questions and technical terms.

### 🏗️ Architecture & Security

- **[OAuth2 Flow](./Architecture-OAuth2-Flow)**: Deep dive into our secure, token-free authentication protocol.
- **[Rate Limiting](./Architecture-Rate-Limiting)**: Learn how our intelligent actor system prevents account flags.
- **[Account & GDPR](./Architecture-Account-GDPR)**: See how we handle sensitive profile data and official Discord data requests.
- **[Security Deep Dive](./Security-Privacy-Deep-Dive)**: The technical measures that protect your digital sovereignty.
- **[Error Handling](./Architecture-Error-Handling)**: Understand our robust protocol for cross-platform stability.

### 🛠️ Technical Insights

- **[Logging Strategy](./Logging-Strategy)**: How we maintain transparency with zero data leakage.
- **[Project Structure](./Project-Structure)**: A map of the codebase for developers and contributors.
- **[Testing Methodology](./Testing-Methodology)**: Our rigorous standards for unit, integration, and E2E testing.
- **[Contributor Onboarding](./Contributor-Onboarding)**: Guide for setting up the dev environment and our quality standards.

---

## 🎯 Vision & Core Tenets

Our mission is to restore **Digital Sovereignty** to every Discord user. In an era where data is commodified, we provide the tools to take it back. We operate under three unbreakable laws:

1.  **🛡️ Unyielding Security**: Your data stays local. We leverage the **OS Keychain** (Windows Credential Manager, macOS Keychain, Linux Secret Service) to ensure tokens are encrypted at rest. We never see your password, and we never touch your messages on our servers—because we don't have any.
2.  **⚡ Peak Performance**: Managing thousands of messages or hundreds of servers shouldn't be a chore. Our backend is powered by **Tokio**, an asynchronous runtime for Rust, allowing us to handle massive API queues without blocking the UI.
3.  **💖 Total User Empowerment**: Privacy is a right, not a technical skill. We provide a beautiful, Material You (M3) inspired interface that makes complex operations as simple as a few clicks.

---

## 🧠 Technical Philosophy

We don't just write code; we engineer solutions for the long term.

- **Modular Architecture**: Every feature (Message Purge, Server Leaving, Identity Management) is a separate module. This allows us to scale functionality without creating a "spaghetti" codebase.
- **Rust for the Core**: We chose Rust for its memory safety and speed. By eliminating common bugs like null pointers and data races at compile time, we ensure the application is rock-solid.
- **Tauri for the Shell**: Tauri provides a lightweight bridge between our Rust backend and TypeScript frontend. It uses the system's native webview, keeping the app size tiny (under 10MB) compared to Electron (80MB+).
- **State Management**: We use **Zustand** on the frontend for lightweight, high-performance state synchronization between the UI and the backend events.

---

## 🏆 Current MVP Definition

The **Minimum Viable Product (MVP)** for Discord Purge is:

> A fully distributable and installable **desktop application** that enables a user to securely log in via Discord OAuth2 and seamlessly utilize the **Bulk Message Deletion** feature from end-to-end, all presented through a professional, M3-compliant user interface.

_Current Status_: **Phase 1 Complete**. All core features are functional, and the CI/CD pipeline is fully automated.

---

## 🆘 Getting Help & Contributing

- **Bugs**: Report issues on our [GitHub Issue Tracker](https://github.com/TourIT-NL/friendly-discord-bot/issues).
- **Discussions**: Join the conversation in our [Discussions tab](https://github.com/TourIT-NL/friendly-discord-bot/discussions).
- **Community**: Connect with us on our [Official Discord Server](https://discord.gg/kRFhXPTm).
- **Developers**: Check out our [Contributing Guide](../CONTRIBUTING.md) to start building.

_Last updated: February 25, 2026_
