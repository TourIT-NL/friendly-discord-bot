# 🧪 Testing Methodology: Our Quality Standard

Reliability is non-negotiable for a privacy tool. This document outlines the rigorous, multi-layered testing strategy that ensures **Discord Purge** performs safely, correctly, and predictably across all platforms.

---

## 🦀 1. Rust Backend Testing (Native Layer)

We use the standard `cargo test` harness, enhanced with specialized libraries.

### Unit Testing

- **Target**: Pure functions and logic components.
- **Examples**: Rate-limit jitter calculations, timestamp formatting, and permission check logic.
- **Mocking**: We use `mockall` to simulate OS filesystem or network traits without touching the actual machine.

### Integration Testing

- **Target**: The interaction between the Rust core and external APIs.
- **Tools**: **`wiremock`** is used to stand up a local HTTP server that acts like the Discord API. This allows us to test our app's response to `429`, `403`, and `500` status codes without needing a real Discord account.

---

## 🚀 2. Frontend Testing (Web Layer)

### Component Testing (`Vitest`)

- **Environment**: `jsdom` (simulates a browser environment).
- **Tools**: **React Testing Library**.
- **Focus**: User-centric testing. We don't test implementation details; we test that "When I click Delete, the Confirmation Modal appears."

### State Testing

- **Target**: **Zustand** stores.
- **Focus**: Verifying that events from Rust (e.g., `progress_update`) correctly update the global state and trigger UI re-renders.

---

## 🤖 3. End-to-End (E2E) Testing

Using **Webdriver** support integrated into Tauri, we automate full user journeys.

**Standard E2E Scenario:**

1.  **Launch**: App opens to the Login screen.
2.  **Auth**: Simulate a successful OAuth2 handshake.
3.  **Discovery**: App scans for DMs and Servers.
4.  **Dry Run**: Select a channel and run a "Simulation".
5.  **Validation**: Verify that no actual API requests were sent during simulation mode.

---

## 🛡️ 4. The Security Audit Pipeline

Quality includes security. Every Pull Request triggers:

- **`cargo audit`**: Checks for vulnerable crates in the dependency tree.
- **`cargo deny`**: Enforces license compliance (no GPL!) and bans unvetted crates.
- **`npm audit`**: Scans the frontend for compromised packages.
- **CodeQL**: Semantic analysis to detect memory leaks or potential buffer overflows in the Rust layer.

---

## 📊 Quality Targets

- **Logic Coverage**: 80%+ code coverage for the `api` and `auth` modules.
- **Zero Critical**: Release builds are blocked if a "High" or "Critical" vulnerability is detected.
- **Cross-Platform Parity**: All tests must pass on Windows, macOS, and Linux runners in GitHub Actions.

_Last updated: February 25, 2026_
