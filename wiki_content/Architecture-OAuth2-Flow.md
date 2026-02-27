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

## 🔄 Alternative Authentication Flows

While OAuth2 PKCE is the primary method, the utility supports two high-fidelity fallbacks for enhanced resilience.

### 1. QR Login (Remote Auth V2)

A "passwordless" flow using Discord's official Mobile App scanner.

- **Security**: Uses 2048-bit RSA key exchange. The private key never leaves the application.
- **Genuineness**: The connection utilizes high-fidelity browser fingerprinting (Chrome 144+, Super Properties) to mirror legitimate client behavior.
- **Workflow**:
  1. App generates RSA keypair.
  2. Establishes a Secure WebSocket (WSS) to Discord's gateway.
  3. Proof-of-nonce is performed via RSA decryption.
  4. Discord issues a temporary token which is decrypted and promoted to a session.

### 2. Desktop RPC Authorization

Direct interaction with a running Discord Desktop client.

- **Trigger**: Uses the `AUTHORIZE` command via the local RPC socket.
- **Prompting**: Set to `consent` mode to ensure the user is explicitly presented with a native Discord authorization popup.
- **Integration**: Seamlessly captures the `access_token` from the local client session without browser redirects.

---

## ⚠️ Troubleshooting the Flow

- **Port Blocking**: If a firewall blocks the temporary server or RPC port (6463-6472), the authentication will fail.
- **Browser Isolation**: The app requires a functional system browser to complete the consent screen.
- **Clock Skew**: If the user's system time is significantly off, the token exchange might fail due to timestamp verification.

_Last updated: February 26, 2026_
