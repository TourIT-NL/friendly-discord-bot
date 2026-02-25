# 🔐 Architecture: Secure OAuth2 PKCE Flow

Security is our first law. This document details the cryptographic handshake used to connect **Discord Purge** to the Discord API without ever exposing user passwords or storing long-term master secrets in cleartext.

---

## 🏗️ Technical Workflow

We implement **RFC 7636 (PKCE)**, which is the industry standard for secure desktop applications. Unlike the traditional "Implicit Flow", PKCE prevents "Authorization Code Interception" attacks.

### The Handshake Sequence:

1.  **Initiation**: User clicks "Login". Frontend calls `invoke('start_oauth_flow')`.
2.  **Secret Generation**: Rust generates a high-entropy `code_verifier` (a random string) and hashes it using SHA-256 to create a `code_challenge`.
3.  **Local Callback Server**: A temporary, isolated HTTP server starts on a random available port (e.g., `58123`) bound only to `localhost`.
4.  **Handoff**: `tauri::api::shell::open` sends the user to Discord's authorize URL with the `code_challenge` and a unique `state` string.
5.  **User Authorization**: User clicks "Authorize" on Discord's website.
6.  **Secure Redirect**: Discord redirects the browser to `http://localhost:58123/callback?code=...&state=...`.
7.  **Validation**: Rust backend validates that the returned `state` matches the one sent, preventing Cross-Site Request Forgery (CSRF).
8.  **Exchange**: Rust backend sends the `code` and the original `code_verifier` to Discord's token endpoint. Discord verifies the hash and issues the tokens.
9.  **Storage**: Tokens are immediately stored in the **OS Keychain** (Encrypted).
10. **Success**: The local server is destroyed, and an `auth_success` event is emitted.

---

## 🛡️ Security Analysis: Why this is safe

- **Zero Knowledge**: The application never touches your Discord password.
- **Encrypted at Rest**: We use the **Keyring** library to access the platform's secure vault (Windows Credential Manager, macOS Keychain).
- **Memory Safety**: The `code_verifier` is held in Rust memory and cleared after use.
- **Ephemeral Server**: The local HTTP server exists for only a few seconds and only accepts requests from the local loopback interface.

---

## 🔑 Permissions Requested (Scopes)

We strictly follow the **Principle of Least Privilege**:

| Scope           | Reason                                                              |
| :-------------- | :------------------------------------------------------------------ |
| `identify`      | To display your username and avatar in the dashboard.               |
| `guilds`        | To list the servers you are in for mass-leaving or message cleanup. |
| `messages.read` | Required to scan for and identify your own messages.                |
| `gdm.join`      | Allows the tool to interact with group DM conversations.            |

---

## ⚠️ Troubleshooting the Flow

- **Port Blocking**: If a firewall blocks the temporary server, the authentication will fail.
- **Browser Isolation**: The app requires a functional system browser to complete the consent screen.
- **Clock Skew**: If the user's system time is significantly off, the token exchange might fail due to timestamp verification.

_Last updated: February 25, 2026_
