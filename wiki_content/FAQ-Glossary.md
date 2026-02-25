# 📚 FAQ & Technical Glossary

This document clarifies common questions and defines the technical terminology used throughout the **Discord Purge** project.

---

## ❓ Frequently Asked Questions

### Is this safe for my Discord account?

**Yes.** We prioritize safety by using the official OAuth2 flow and an "Adaptive Rate Limiting Actor". This makes our application behave like a very fast but responsible human user, significantly reducing the risk of being flagged by anti-abuse systems.

### Can I recover messages once they are deleted?

**No.** Deletion on Discord is permanent. Once the purge is executed, the messages are removed from Discord's servers forever. Always use the **Preview (Dry Run)** feature before clicking the final button.

### Why do I need to download an installer?

Web-based "token cleaners" are extremely dangerous because they require you to paste your master token into a website. By being a native desktop app, we ensure that your credentials **never leave your computer**.

### What is the "Simulation" mode?

Simulation mode (or Dry Run) performs all the logic—scanning, filtering, and queueing—but skips the final "Delete" call. It allows you to see exactly what _would_ happen without any risk.

---

## 📖 Technical Glossary

### OAuth2 + PKCE

- **Definition**: Proof Key for Code Exchange.
- **In Discord Purge**: A secure way to log in where the app never sees your password. It creates a temporary secret to verify your identity.

### Jitter

- **Definition**: Adding small, random variations to timing.
- **In Discord Purge**: We add 50-200ms of random delay to requests so they don't look like a robotic "beat," making the application safer to use.

### M3 (Material 3)

- **Definition**: Google’s latest design system (Material You).
- **In Discord Purge**: Our UI follows these guidelines to ensure a beautiful, accessible, and modern user experience.

### Rate Limit (429)

- **Definition**: Discord's way of saying "Slow down."
- **In Discord Purge**: Our **Actor** system automatically detects these and pauses the application until it's safe to continue.

### IPC (Inter-Process Communication)

- **Definition**: How two different programs talk to each other.
- **In Discord Purge**: The mechanism that allows our TypeScript UI to send commands to our Rust backend.

_Last updated: February 25, 2026_
