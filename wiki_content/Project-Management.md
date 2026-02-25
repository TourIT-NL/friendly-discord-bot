# 🎯 Project Management: Vision & Execution

This document outlines the scope, phasing, and MVP definition for **Discord Purge**. We focus on shipping early, shipping often, and never compromising on our core tenets of security and performance.

---

## 🏗️ Project Phases

### Phase 1: Core Foundation (CURRENT)

- **Infrastructure**: Fully automated Rust/Tauri/TS build pipeline.
- **Authentication**: Rock-solid OAuth2 PKCE handshake.
- **Message Cleanup**: Stable bulk deletion engine with rate-limit protection.
- **Deployment**: Digitally signed installers for all desktop OSs.

### Phase 2: Advanced Sanitization (Q2 2026)

- **Keyword Filtering**: Support for regex-based targeting.
- **Attachment Management**: Identifying and deleting large files vs. text.
- **Relationship Management**: Bulk friend removal and blocklist purging.
- **GDPR Tooling**: Integrated instructions for official data requests.

### Phase 3: Automation & Scale (Q4 2026)

- **Scheduled Tasks**: Periodic background cleanup.
- **Multi-Account Hub**: Managing several digital identities from one UI.
- **Audit Visualizer**: Graphical reports of your digital footprint reduction.

---

## 🛠️ Project Scope

### In Scope

- **Desktop Focus**: Native applications for Windows (x64/ARM), macOS (Universal), and Linux.
- **Direct API Access**: Local-only client that communicates directly with Discord.
- **Security Automation**: SBOMs, cargo-deny, and CodeQL integration.

### Out of Scope (For Now)

- **Web-Based Interface**: We refuse to host your Discord tokens on a central server.
- **Automated Bot accounts**: This is a tool for _user_ accounts, not server moderation bots.
- **Cloud Sync**: All settings and logs remain local to protect your privacy.

---

## 🏆 MVP Definition

Our **Minimum Viable Product** is our "Masterpiece Prototype":

> A standalone desktop application that allows a user to log in via OAuth2, select multiple DM/Server channels from a GUI, and delete their entire message history while safely respecting Discord's global rate limits.

---

## ⚖️ Legal & Risk Management

We operate with total transparency regarding Discord's terms:

1.  **Risk Mitigation**: Our adaptive rate-limiter is the most conservative in the industry.
2.  **Disclaimer Engine**: Users are clearly informed of the risks before executing destructive actions.
3.  **Local-Only**: By keeping data on the user's machine, we eliminate 99% of common data breach vectors.

_Last updated: February 25, 2026_
