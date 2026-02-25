# 🚀 CI/CD Pipeline: Professional Automation Suite

Elite software requires elite automation. This document describes the "Platinum Tier" CI/CD infrastructure that powers **Discord Purge**.

---

## 🛠️ GitHub Actions Ecosystem

We use several interconnected workflows to ensure the repository is always in a high-integrity state.

### 1. The Controller (`main.yml`)

- **Path Filtering**: Detects if changes are in `frontend`, `backend`, or `docs`.
- **Parallel Linting**: Runs `eslint`, `prettier`, `clippy`, and `rustfmt` concurrently.
- **Release Engine**: On Git Tags (`v*`), it builds binaries for Windows, macOS, and Linux.
- **Nightly Builds**: Every push to `main` generates a testable "dev build" artifact.

### 2. PR Quality Gate (`pr-quality.yml`)

- **Semantic Check**: Ensures PR titles follow Conventional Commits (e.g., `feat:`, `fix:`).
- **Deep Audit**: Runs `cargo deny` to check for security vulnerabilities and prohibited licenses.
- **Frontend Health**: Uses `depcheck` to prevent unused code and dependency bloat.

### 3. Docs & Spelling (`docs-engine.yml`)

- **Spell Check**: Scans the entire project for typos using the `typos` crate.
- **Link Checker**: Uses `lychee` to ensure every URL in the documentation is alive and valid.

### 4. Wiki Sync (`wiki-masterpiece.yml`)

- **Auto-Sync**: Automatically pushes markdown from the `wiki_content/` folder to the GitHub Wiki tab on every push to `main`.

---

## 🛡️ Security Automation

- **Software Bill of Materials (SBOM)**: Weekly generation of SPDX-compliant manifests for complete dependency transparency.
- **Dependabot**: Automated weekly updates for both NPM and Cargo, grouped into clean, manageable PRs.
- **CodeQL**: Deep semantic analysis to detect potential SQL injections, XSS, and buffer overflows.

_Last updated: February 25, 2026_
