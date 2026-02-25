# 📂 Project Structure: Mapping the Masterpiece

A clean, modular architecture is the key to scaling **Discord Purge** from a simple script to a production-grade application. This document provides a high-level map of our codebase for developers and maintainers.

---

## 🌲 Repository Tree

```text
discord-privacy-util/
├── .github/                # CI/CD & Repository Management
│   ├── workflows/          # Automation Engines (Main, Quality, Docs)
│   ├── labeler.yml         # PR Categorization Rules
│   └── release-drafter.yml # Changelog Automation
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # M3-compliant UI Components (Atomic Design)
│   ├── hooks/              # Custom Logic (Auth, Operations, Selection)
│   ├── store/              # Zustand State Management (Lightweight)
│   └── types/              # Unified TypeScript Interfaces
├── src-tauri/              # Backend (Rust Core)
│   ├── src/
│   │   ├── api/            # API Clients, Models, and Rate Limiter Actor
│   │   ├── auth/           # OAuth2 PKCE, QR Login, and Token Vault
│   │   ├── core/           # Standardized Error Handling & App Config
│   │   └── main.rs         # The Tauri Entrypoint (IPC Hub)
│   ├── capabilities/       # Security Policy (Whitelist/Denylist for Rust)
│   ├── deny.toml           # Security Audit Configuration
│   └── Cargo.toml          # Backend Dependencies
├── wiki_content/           # The Documentation Hub (Synced to Wiki Tab)
├── package.json            # Frontend Scripts & Quality Hooks (Husky)
├── README.md               # The SEO Masterpiece
└── LICENSE                 # MIT License
```

---

## 🏗️ The Architectural Split

We utilize a **Hybrid-Process Architecture** to balance UI fluidity with system-level security.

### 1. The Frontend (The Renderer)

- **Tech**: Vite + React + Tailwind CSS.
- **Role**: Handles all user interactions and visualizes progress.
- **IPC**: Communicates with the backend using `@tauri-apps/api`. It sends commands (e.g., `start_purge`) and listens for events (e.g., `deletion_progress`).

### 2. The Backend (The Core)

- **Tech**: Rust + Tokio + Tauri.
- **Role**: Manages the "Heavy Lifting"—networking, encryption, and the OS keychain.
- **Safety**: Rust's type system prevents data races between the Rate Limiter and the Deletion Engine.

### 3. The Documentation (The Wiki)

- **Role**: Living blueprint.
- **Workflow**: Documentation is treated as code. Edits are made in `wiki_content/` and synced via GitHub Actions, ensuring our "Technical Masterpiece" remains well-documented.

---

## 🛠️ Key Coding Standards

1.  **Strict Typing**: No `any` types in TypeScript.
2.  **Result Propagation**: Use `AppError` for all cross-boundary communication.
3.  **Hooks First**: UI components should be "dumb"; all logic resides in custom hooks (`useDiscordAuth`, etc.).

_Last updated: February 25, 2026_
