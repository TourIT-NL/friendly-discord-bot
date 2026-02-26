# 🔒 Security & Privacy Deep Dive

Privacy isn't just a promise; it's a technical architecture. This document provides an intricate look at the security measures that make **Discord Purge** the safest cleanup tool in the ecosystem.

---

## 🔐 Multi-Layer Vault Security

We never store your Discord password. Even with **OAuth2**, your tokens are highly sensitive. We employ a multi-layered "Vault" strategy to protect them.

### 🛡️ Master Password (Optional but Recommended)

Users can set a Master Password that adds a secondary layer of encryption to the Vault.

- **KDF**: We use **Argon2id** (the winner of the Password Hashing Competition) to derive a 256-bit encryption key from your password.
- **AES-256-GCM**: The underlying Vault key is encrypted with the derived key before being stored. Even if an attacker gains physical access to your OS account, your tokens remain encrypted.

### 🔑 OS Keychain Integration

The application utilizes the **Keyring** crate to access native secure storage:

- **Windows**: Windows Credential Manager.
- **macOS**: Keychain Access.
- **Linux**: Secret Service API.

### 🧹 Memory Hardening

- **Zeroize**: We implement the `Zeroize` trait for all sensitive variables (tokens, keys, passwords). When these variables go out of scope, the memory they occupied is explicitly overwritten with zeros to prevent leakage in memory dumps.
- **Volatile Storage**: All scanned data (DMs, Servers) is held exclusively in volatile RAM and is never committed to disk.

---

## 🛡️ Data Minimization

We follow the strict principle of "No Persistence":

- **No DB**: The application has no database. All scanned data (DMs, Servers) is held in **volatile memory (RAM)**.
- **Zero Leakage**: When you close the app, the list of scanned messages is gone forever.
- **Sanitized Logs**: Our logs are strictly metadata-only. We use specialized Rust filters to ensure that even if an error contains a snippet of text, it is scrubbed before being written to disk.

---

## 🦀 Memory Safety (The Rust Advantage)

The majority of security exploits (70% according to Microsoft/Google) are due to memory management errors. By using **Rust** for our core engine, we leverage:

- **Compile-time Checks**: Buffer overflows and "Use-after-free" bugs are impossible by design.
- **Zero-Cost Abstractions**: We get this safety without sacrificing the raw performance needed to process thousands of Discord events.

---

## 🕵️ Transparency & Auditing

- **Public Code**: Every line of code is open for public review.
- **SBOM (Software Bill of Materials)**: We generate machine-readable manifests of all our dependencies, allowing anyone to verify our supply chain security.
- **Digitally Signed**: Our release binaries are cryptographically signed to ensure that the code you download is exactly the code we built, with no tampering in between.

---

## 🚫 No Third-Party Analytics

We do not use Google Analytics, Sentry, or any other cloud-based tracking.

- **Your usage is your business.**
- **No "Home Calling"**: The only server the application communicates with is `discord.com`.

_Last updated: February 25, 2026_
