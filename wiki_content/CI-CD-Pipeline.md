# 🚀 CI/CD Pipeline: Professional Automation Suite

Elite software requires elite automation. This document describes the "Platinum Tier" CI/CD infrastructure that powers **Discord Purge**, ensuring every release is secure, stable, and transparent.

---

## 🛠️ GitHub Actions Architecture

We use a multi-layered workflow system. Instead of one massive script, we use specialized engines for different quality gates.

### 1. The Release Controller (`main.yml`)

This is the heart of our distribution system.

- **Intelligent Path Filtering**: It only runs frontend tests if `src/` changes, or backend tests if `src-tauri/` changes, saving developer time.
- **Nightly Build Engine**: Every push to `main` generates a temporary executable. Developers can download these `.exe` or `.dmg` files immediately to verify bug fixes before a formal release.
- **Multi-Platform Matrix**: Builds are compiled on native Windows, macOS, and Linux runners to ensure true compatibility.

### 2. PR Quality & Health Gate (`pr-quality.yml`)

Acts as the "Entry Guard" for any new code.

- **Semantic PR Titles**: Enforces [Conventional Commits](https://www.conventionalcommits.org/). This ensures our automated changelogs are perfectly categorized.
- **Advanced Rust Audit**: Uses `cargo-deny` to check:
  - **Advisories**: Security vulnerabilities.
  - **Licenses**: Ensures all dependencies are MIT/Apache compatible.
  - **Bans**: Prevents "bad" or duplicate crates.
- **Frontend Health**: Uses `depcheck` to prevent dependency bloat and unused libraries.

### 3. Docs & Spelling Engine (`docs-engine.yml`)

Ensures our "Masterpiece" standards extend to our words.

- **Typo Detection**: Uses the `typos` crate to scan code and markdown for spelling errors.
- **Link Integrity**: Uses `lychee` to scan every URL in the README and Wiki. It fails the build if it finds a dead 404 link.

### 4. Wiki Sync Masterpiece (`wiki-masterpiece.yml`)

Treats documentation as code.

- **Auto-Deployment**: Automatically pushes markdown from the `wiki_content/` folder to the official GitHub Wiki tab on every push to `main`. This allows for documentation versioning and peer review via Pull Requests.

---

## 🛡️ Security Automation

- **SBOM Generation**: Every week, we generate a **Software Bill of Materials (SPDX)**. This machine-readable manifest lists every single library in the app, providing 100% transparency for security audits.
- **Dependabot Grouping**: We use a custom `dependabot.yml` that groups updates. Instead of 20 separate PRs, you get one clean "Backend Updates" PR, making maintenance effortless.
- **CodeQL**: Deep semantic analysis powered by GitHub to detect complex security flaws (SQLi, Buffer overflows).

---

## 🏆 Release Automation (`release-drafter.yml`)

Our releases are "Self-Drafting". The system watches PR labels and titles to build a beautiful, categorized changelog (`🚀 Features`, `🐛 Bug Fixes`) so that humans only have to hit "Publish".

_Last updated: February 25, 2026_
