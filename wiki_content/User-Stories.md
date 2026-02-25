# 📋 User Stories: Defining the Mission

This document translates user needs into actionable technical requirements. Every feature in **Discord Purge** is driven by a specific user goal, ensuring we build a tool that solves real problems for privacy-conscious individuals.

---

## 🔐 2.1. Authentication (US-001)

**"Secure Access Without Compromise"**

- **User Story**: As a new user, I want to **securely log in** using my Discord account so that I can access features like bulk deletion without ever exposing my raw token or password to the application.
- **Acceptance Criteria**:
  1.  The app presents a prominent "Login with Discord" button on launch.
  2.  Clicking opens the system browser to the official Discord authorization page.
  3.  Scopes are clearly listed (Identity, Guilds, etc.).
  4.  Redirect is handled by a temporary local server, ensuring code-to-token exchange happens securely.
  5.  Success transitions the UI instantly to the Dashboard.
  6.  Session is persisted via OS Keychain for automatic login on next start.

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

---

## 👤 2.4. Relationship Purge (US-004)

**"Social Reset"**

- **User Story**: As a user wanting a social fresh start, I want to remove friend connections, clear blocked lists, and cancel pending requests in bulk.
- **Acceptance Criteria**:
  1.  View categorized lists of Friends, Blocked, and Pending users.
  2.  Select multiple relationships for removal.
  3.  Immediate execution with rate-limit protection.

_Last updated: February 25, 2026_
