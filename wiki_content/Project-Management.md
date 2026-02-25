# 🎯 Project Management: Vision & Roadmap

This document outlines the scope, phasing, and MVP definition for **Discord Purge**. We focus on shipping early, shipping often, and never compromising on our core tenets.

---

## 🛠️ Project Scope

### In Scope (v1.0)

- **Authentication**: Secure OAuth2 with PKCE + Session Persistence.
- **Cleanup**: Bulk message deletion (DMs, Groups, Servers).
- **Management**: Bulk server leave with pre-purge option.
- **Privacy**: Relationship removal (Friends, Blocked).
- **Platform**: Desktop installers for Windows, macOS, and Linux.

### Out of Scope (v1.0)

- Mobile applications (Android/iOS) - _Currently in Roadmap_.
- Cloud synchronization of settings.
- Automated bots for server moderation.

---

## 🗺️ Roadmap (Future Vision)

### Phase 2: Advanced Cleanup

- **Keyword Filtering**: Regex-based deletion.
- **Attachment Purge**: Isolate and delete only files/images.
- **Thread Support**: Extend cleanup to Discord threads and forum posts.

### Phase 3: Automation & Scale

- **Scheduled Tasks**: Daily/Weekly auto-purge.
- **Multi-Account**: Manage several digital identities simultaneously.
- **GDPR Tooling**: Auto-download and parse Discord data packages.

---

## 🏆 MVP Definition

The **Minimum Viable Product** is defined as:

> A standalone desktop application that allows a user to log in via OAuth2, select multiple DM/Server channels, and delete their message history while respecting rate limits.

_Last updated: February 25, 2026_
