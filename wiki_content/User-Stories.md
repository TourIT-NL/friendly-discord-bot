# 📋 User Stories: Defining the Mission

This document translates user needs into actionable technical requirements. Every feature in **Discord Purge** is driven by a specific user goal, ensuring we build a tool that solves real problems for privacy-conscious individuals.

---

## 👥 User Personas: Who are we building for?

1.  **Privacy-Conscious Pat**: A long-time Discord user who is uncomfortable with having years of personal conversations stored on a platform they no longer use as much. They want a "total wipe" before deleting their account.
2.  **Professional Penny**: A user who uses Discord for work or community management and needs to clear old logs or sensitive messages from specific channels periodically.
3.  **Security-Minded Sam**: A tech-savvy user who refuses to give their Discord token to random web-based "cleaners" and wants a local tool they can audit.
4.  **Social Steve**: A user who joined too many servers and has a cluttered sidebar. He needs to leave 50+ servers at once but wants to keep his "Home" servers untouched.

---

## 🔐 2.1. Authentication (US-001)

**"Secure Access Without Compromise"**

- **User Story**: As a new user, I want to **securely log in** using my Discord account so that I can access features like bulk deletion without ever exposing my raw token or password to the application.
- **Acceptance Criteria**:
  1.  The app presents a prominent "Login with Discord" button on launch.
  2.  Clicking opens the system browser to the official Discord authorization page.
  3.  Scopes are clearly listed (Identity, Guilds, messages.read, etc.).
  4.  Redirect is handled by a temporary local server, ensuring code-to-token exchange happens securely.
  5.  Success transitions the UI instantly to the Dashboard.
  6.  Session is persisted via OS Keychain for automatic login on next start.
- **Edge Cases**:
  - _Port Conflict_: If the default redirect port is blocked, the app should find an alternative.
  - _User Cancellation_: If the user closes the browser without authorizing, the app should return to the login screen gracefully.

---

## 🗑️ 2.2. Bulk Message Deletion (US-002)

**"Erasure at Scale"**

- **User Story**: As a privacy-conscious user, I want to **permanently delete my messages in bulk** from channels, DMs, and groups so that I can sanitize my chat history across the entire platform.
- **Acceptance Criteria**:
  1.  Users can see a full list of all DMs, Groups, and Servers.
  2.  Multi-selection is supported for massive cleanup operations.
  3.  Filters include: "Last 24 Hours," "Last 7 Days," "All Time," and Custom Date Ranges.
  4.  A **Confirmation Modal** must appear before execution, detailing the count of targets.
  5.  Users must type `DELETE` to proceed, preventing accidental data loss.
  6.  Real-time progress bars show exactly what is being deleted and how many remain.
- **Technical Implementation Details**:
  - Uses the **Rate Limiting Actor** to prevent account flags.
  - Handles "Message is too old to delete" errors gracefully.
  - Fetches message IDs in chunks of 100 to maximize efficiency.

---

## 👋 2.3. Bulk Server Departure (US-003)

**"Digital Decluttering"**

- **User Story**: As a user cleaning up my account, I want to **leave multiple servers at once** while easily whitelisting the communities I want to keep.
- **Acceptance Criteria**:
  1.  A checkbox list of every server the user is currently in.
  2.  "Select All" and "Unselect All" helpers for rapid management.
  3.  A final confirmation requiring the word `LEAVE` to proceed.
  4.  The application provides live feedback as it processes each departure.
  5.  Optional: Delete all user messages _before_ leaving the guild.
- **Edge Cases**:
  - _Owned Guilds_: If the user is the owner, the app informs them they must transfer ownership or delete the guild manually first.

---

## 👤 2.4. Relationship Purge (US-004)

**"Social Reset"**

- **User Story**: As a user wanting a social fresh start, I want to remove friend connections, clear blocked lists, and cancel pending requests in bulk.
- **Acceptance Criteria**:
  1.  View categorized lists of Friends, Blocked, and Pending users.
  2.  Select multiple relationships for removal.
  3.  Immediate execution with rate-limit protection.

---

## 🔐 2.5. Master Password Protection (US-008)

**"Encryption Beyond the OS"**

- **User Story**: As a security-conscious user, I want to protect my Discord tokens with a master password so that my data is safe even if someone gains access to my computer.
- **Acceptance Criteria**:
  1.  Option to set a Master Password during setup or from settings.
  2.  Uses Argon2id for key derivation and AES-256-GCM for vault encryption.
  3.  The vault remains locked until the password is provided at startup.
  4.  Tokens are zeroized in memory immediately after use.

---

## 🎭 2.6. Network Stealth & Proxy (US-009)

**"Masking the Trace"**

- **User Story**: As a user avoiding platform-level tracking, I want to route my cleanup operations through a proxy and use dynamic fingerprints.
- **Acceptance Criteria**:
  1.  Configure SOCKS5 or Tor proxies directly in the app.
  2.  The app rotates User-Agents from a pool of modern browsers.
  3.  Internal Discord headers (x-super-properties) are dynamically generated.

---

## 👻 2.7. Profile Ghosting (US-010)

**"Instant Anonymity"**

- **User Story**: As a user wanting to vanish instantly, I want to clear all my profile metadata (bio, avatar, banner, status) in one click.
- **Acceptance Criteria**:
  1.  A "Ghost Profile" button that triggers a bulk reset of all identity metadata.
  2.  Immediate execution across both User and Settings endpoints.

---

## 📂 2.8. GDPR Data Discovery (US-011)

**"Forensic Deep Cleaning"**

- **User Story**: As a user who wants to find every trace of my activity, I want to upload my Discord data package to find hidden or forgotten channels.
- **Acceptance Criteria**:
  1.  Upload a standard Discord `data.zip` package.
  2.  The app parses the package to extract channel and server IDs not currently visible in the active UI.
  3.  Discovered nodes can be targeted for mass deletion.

---

## 🔥 2.9. Nuclear Option (US-012)

**"Complete Trace Erasure"**

- **User Story**: As a user performing a total exit from the platform, I want a single command to trigger all sanitization protocols simultaneously so that I can vanish with minimal effort.
- **Acceptance Criteria**:
  1.  A "Nuclear Option" button in the sidebar.
  2.  A strong confirmation prompt with a warning about its irreversibility.
  3.  Sequentially executes: Ghost Profile, Max Privacy, Relationship Wipe, and Guild Departure.
  4.  Updates progress in real-time for each sub-protocol.

---

## 🔮 Future User Stories (Phase 3 & 4)

- **US-005: Keyword Filter**: "As a user, I want to delete only messages containing my phone number or email address across all channels."
- **US-006: Attachment Wipe**: "As a user, I want to delete only images and files I have uploaded, leaving the text messages intact."
- **US-007: Scheduled Purge**: "As a user, I want the app to automatically delete messages older than 30 days every time I launch it."

_Last updated: February 25, 2026_
