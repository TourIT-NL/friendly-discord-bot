# 🧪 Testing Methodology: Our Quality Standard

Reliability is non-negotiable. This document outlines the rigorous testing strategy that ensures **Discord Purge** performs safely and correctly.

---

## 🦀 1. Rust Backend Testing (Native)

### Unit Tests

We test the "brains" of the app in isolation.

- **Rate Limiter logic**: Ensuring jitter and backoff calculations are correct.
- **Data Parsers**: Verifying Discord API responses are mapped correctly to our types.
- **Encryption wrappers**: Testing the interface with the OS Keychain.

### Integration Tests

We test the interaction with external services.

- **Mock API**: We use `wiremock` to simulate Discord servers, ensuring our app handles `429`, `403`, and `500` status codes gracefully.

---

## 🚀 2. Frontend Testing (Web)

### Component Tests (`Vitest`)

We test our React UI components using **React Testing Library**.

- **Rendering**: Do buttons and inputs appear correctly?
- **Logic**: Does clicking "Delete" trigger the confirmation modal?
- **Accessibility**: Ensuring the app is usable by everyone (Aria labels, focus management).

---

## 🤖 3. End-to-End (E2E) Testing

Using **Webdriver** support in Tauri, we automate a "real user" session:

1.  Launch the app.
2.  Navigate the login screens.
3.  Simulate a dry run of a message deletion.
4.  Verify that the progress bar updates.

---

## 🛡️ 4. Security Audits

- **`cargo audit`**: Scans the backend for vulnerable crates.
- **`npm audit`**: Scans the frontend for compromised packages.
- **`cargo deny`**: Checks our entire dependency tree for license violations and banned sources.
- **Static Analysis**: We use `clippy` (Rust) and `eslint` (TS) to catch bugs before they are even compiled.

---

## 📊 Quality Targets

- **Coverage**: We aim for 80% code coverage on core logic modules.
- **Performance**: Any UI action must respond in under 100ms.
- **Zero Criticals**: No release is allowed if any security audit fails.

_Last updated: February 25, 2026_
