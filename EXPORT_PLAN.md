# 📦 Master Blueprint: Discord Data Export Suite

This document serves as the integration, testing, and release plan for the **Full Data Export** functionality. It will be used as a living worksheet during implementation.

---

## 🎯 1. Objectives

Implement a robust, local-only export engine that allows users to archive their digital footprint before or during cleanup.

### Key Features:

1.  **Attachment Harvester**: Export all files (sent/received) with selectable filters.
2.  **HTML Chat Chronicler**: Generate beautiful, easy-to-read HTML logs of conversations.
3.  **User-Guild Archivist**: Extract every message and attachment a user has ever committed to a specific server.

---

## 🏗️ 2. Architecture Changes

### Backend (Rust)

- **New Module**: `src-tauri/src/api/export.rs`.
- **Tauri Commands**:
  - `start_attachment_export`: Parameters for `direction` (sent/received), `channel_ids`, `date_range`.
  - `start_chat_html_export`: Parameters for `channel_ids`, `include_attachments`.
  - `start_guild_user_archive`: Parameters for `guild_id`.
- **Logic Engine**:
  - Utilizes the `RateLimiterActor` for stable crawling.
  - Implements Discord message pagination (fetching 100 messages recursively).
  - Uses `tokio` for non-blocking file I/O.
  - Uses `zip-rs` for creating compressed archives.

### Frontend (TypeScript/React)

- **New Mode**: `export` mode in `DashboardView`.
- **Components**:
  - `ExportSettings`: Checkboxes for Sent/Received, HTML toggle, target directories.
  - `ExportProgress`: Detailed overlay showing current channel, file count, and estimated time.
- **State**: Integrate into `useDiscordOperations` hook.

---

## 🛡️ 3. Security & Safety

- **Rate Limit Protection**: The export engine MUST respect the same conservative limits as the deletion engine to prevent account flags.
- **Disk Space Guard**: Check available disk space before starting massive downloads.
- **Zero Leakage**: No data is uploaded. Exports are strictly written to the user's selected local folder.

---

## ✅ 4. Implementation Checklist (Task Breakdown)

### Phase 1: The Core Crawler [PENDING]

- [ ] Implement recursive message fetcher in Rust (handling pagination).
- [ ] Link crawler to `RateLimiterActor`.
- [ ] Add `AppError` variants for export failures (e.g., `disk_full`, `api_forbidden`).

### Phase 2: Attachment Harvesting [PENDING]

- [ ] Implement file downloader with `reqwest`.
- [ ] Implement Sent/Received filtering logic.
- [ ] Implement ZIP compression engine.

### Phase 3: HTML Rendering [PENDING]

- [ ] Create HTML/CSS templates for "Easy-Readable" logs.
- [ ] Implement message-to-HTML transformation logic.
- [ ] Handle embedded attachment links in HTML.

### Phase 4: Server Archive [PENDING]

- [ ] Implement guild-wide user-message filtering.
- [ ] Combine message text and attachment downloads into a single package.

### Phase 5: Frontend Integration [PENDING]

- [ ] Build the "Export" UI in the dashboard.
- [ ] Add real-time event listeners for `export_progress`.
- [ ] Add directory picker using Tauri's dialog API.

---

## 🧪 5. Testing Strategy

### Unit Tests

- [ ] Test the recursive pagination logic with mocked message counts.
- [ ] Test HTML template rendering with various message types (embeds, mentions, attachments).

### Integration Tests

- [ ] Use `wiremock` to simulate a server with 1000+ messages and verify the crawler fetches them all.
- [ ] Verify rate-limit pauses during long export cycles.

### Manual QA

- [ ] Export a small DM history (Windows).
- [ ] Export a large server history (macOS/Linux).
- [ ] Verify ZIP integrity and HTML readability.

---

## 🚀 6. Release Plan

1.  **Documentation**: Update the Wiki with the new "Export Suite" deep dive.
2.  **Versioning**: Bump to `v1.1.0` (Minor release for new features).
3.  **Nightly Test**: Push to `main` and verify Nightly Artifacts work correctly.
4.  **Tag & Release**: Formal release with changelog updated via Release Drafter.

---

_Date Created: February 25, 2026_
_Status: INITIAL PLANNING_
