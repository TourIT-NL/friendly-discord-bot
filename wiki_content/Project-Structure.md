# 📂 Project Structure: Mapping the Masterpiece

A clean architecture is essential for long-term maintainability. This document provides a directory map for developers and contributors.

---

## 🌲 Tree Overview

```text
discord-privacy-util/
├── .github/                # CI/CD Workflows & Config
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # M3-compliant UI Components
│   ├── hooks/              # Custom React Hooks (Auth, API)
│   ├── store/              # Zustand State Management
│   └── types/              # TypeScript Interfaces
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── api/            # Discord API Client & Actor
│   │   ├── auth/           # OAuth2 & Token Logic
│   │   ├── core/           # Error Handling & Utilities
│   │   └── main.rs         # Tauri Command Entrypoints
│   ├── capabilities/       # Tauri Security Permissions
│   └── Cargo.toml          # Rust Dependencies
├── wiki_content/           # Documentation (Auto-Syncs to Wiki)
├── package.json            # Frontend Dependencies & Scripts
├── README.md               # SEO Masterpiece Documentation
└── LICENSE                 # MIT License
```

---

## 🏗️ Architectural Split

### 1. The Frontend (Vite + React)

- **Responsibility**: UI rendering, user interaction, and event listening.
- **Communication**: Uses `@tauri-apps/api` to call Rust functions via IPC (Inter-Process Communication).

### 2. The Backend (Tauri + Rust)

- **Responsibility**: Secure network requests, encryption, OS Keychain access, and global rate limiting.
- **Communication**: Emits events to the frontend (e.g., `deletion_progress`) and returns `Result` objects for commands.

### 3. Documentation (Wiki)

- **Responsibility**: Maintaining the project blueprint.
- **Workflow**: Managed as code within `wiki_content/`, ensuring all technical docs are versioned.

_Last updated: February 25, 2026_
