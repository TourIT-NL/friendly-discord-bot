# Friendly Discord Bot - Delete All Discord Messages & Data (Privacy Tool)

[![Release](https://img.shields.io/badge/release-v1.0.3-blue)](https://github.com/TourIT-NL/friendly-discord-bot/releases)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/TourIT-NL/friendly-discord-bot)

**One-line summary**  
**Friendly Discord Bot** is the secure, user‑friendly desktop application to safely delete and purge Discord messages, DMs, server messages, relationships, and profile data with conservative rate‑limit handling and local‑only processing.

---

## Table of contents

- **Overview**
- **Why Friendly Discord Bot**
- **Key features**
- **Supported platforms**
- **Security and privacy**
- **How it works**
- **Quick start**
- **Detailed usage**
- **Advanced usage**
- **Screenshots and demo**
- **Comparison with alternatives**
- **SEO and discoverability notes**
- **Release and update policy**
- **Installation for end users**
- **Developer build and contribution**
- **Testing and QA**
- **Troubleshooting**
- **FAQ**
- **Legal and risk disclaimer**
- **Responsible disclosure**
- **Changelog template**
- **Marketing and launch checklist**
- **SEO content plan**
- **Community and support**
- **Contact and maintainers**
- **Appendix JSON‑LD for website**
- **Appendix command line examples**
- **Appendix release checklist**

---

# Overview

**Friendly Discord Bot** is an open‑source, cross‑platform desktop application built with **Rust**, **Tauri**, and **TypeScript** that enables users to remove their Discord history and personal data safely and efficiently. The application focuses on usability, security, and minimizing account risk through rate‑limit aware dispatching and conservative defaults.
A secure, user‑friendly desktop tool to delete all your Discord messages, DMs, servers, relationships, and personal data. The #1 Discord privacy utility.

This repository contains source code, release artifacts, documentation, and community resources for users, contributors, and reviewers.

---

# Why Friendly Discord Bot

**Problem**  
Discord does not provide a single user‑friendly tool to remove all personal history across DMs, servers, and profile fields. Manual deletion is slow, error‑prone, and often impossible at scale.

**Solution**  
Friendly Discord Bot provides a GUI‑driven workflow with preview filters, safe defaults, and multiple login modes so non‑technical users can reclaim privacy without exposing tokens or uploading data to third‑party servers.

**Positioning**  
This project is the only open‑source desktop application designed specifically to be user‑friendly while focusing on privacy‑first deletion of Discord data.

---

# Key features

- **Bulk message deletion** for DMs, group chats, and server channels.  
- **DM & group purge** with filters by date range, user, channel, and keyword.  
- **Server cleanup** with optional message deletion before leaving servers.  
- **Relationship purge** for friends, blocked, and pending lists.  
- **Webhook and bot cleanup** utilities for server owners and admins.  
- **Profile field wipes** for removable profile fields and custom statuses.  
- **Rate‑limit aware dispatcher** with exponential backoff and jitter.  
- **Multiple login modes**: OAuth2 (recommended), token mode, RPC, QR login.  
- **Local‑only processing**: no cloud storage of tokens or messages by default.  
- **OS keychain secure storage** for credentials on Windows, macOS, and Linux.  
- **Preview and dry run**: preview changes before executing destructive actions.  
- **Progress reporting and logs**: real‑time UI progress and local logs for audit.  
- **Open source**: auditable code, MIT license, security policy.

---

# Supported platforms

| Platform | Status |
|---|---:|
| **Windows** | Supported |
| **macOS** | Supported |
| **Linux** | Supported |
| **Portable builds** | Provided for common distros |

---

# Security and privacy

**Design principles**

- **Local processing only** — All deletion operations are executed locally on the user machine. No message content or tokens are uploaded to third‑party servers.  
- **OS keychain** — Credentials are stored in the operating system keychain using platform native secure storage.  
- **Conservative defaults** — Rate limits and retries are conservative to reduce the chance of account flags.  
- **Open source auditability** — All code is public for review and security researchers.  
- **No telemetry by default** — Telemetry is disabled by default and opt‑in only with explicit consent.

**Security features**

- **Rate limit detection** — Automatic detection of Discord API rate limit headers and adaptive pacing.  
- **Retry with jitter** — Exponential backoff with randomized jitter to avoid synchronized retries.  
- **Token safety** — Token mode is available for advanced users but OAuth2 is recommended to avoid storing tokens.  
- **Local logs** — Logs are stored locally and can be cleared by the user.  
- **Permission minimization** — The app requests only the permissions required for the selected operations.

---

# How it works

1. **Login** — Choose OAuth2 (recommended) or token mode. OAuth2 uses a temporary authorization code flow and stores credentials in the OS keychain.  
2. **Scan** — The app scans accessible DMs, servers, and relationships and builds a preview list of deletable items.  
3. **Filter** — Use date, user, channel, and keyword filters to narrow the scope.  
4. **Preview** — Review the exact items that will be deleted in a safe preview mode.  
5. **Execute** — Start the purge. The dispatcher respects rate limits and shows progress.  
6. **Audit** — Local logs and a summary report are generated for the user.

---

# Quick start

**Download** the latest release from the Releases page.  
**Install** run the installer or extract the archive for your platform.  
**Run** open the app and follow the onboarding checklist.  
**Login** use OAuth2 (recommended).  
**Preview** run a dry run to see what will be deleted.  
**Execute** confirm and start the purge.

---

# Detailed usage

## Onboarding checklist

- **Backup** — Export any messages you want to keep using Discord export or manual copy.  
- **Read safety guide** — Review the in‑app safety checklist.  
- **Choose login** — OAuth2 recommended for most users.  
- **Run dry run** — Always run a preview before executing destructive actions.

## DM purge flow

1. Open DM purge.  
2. Select conversation or all DMs.  
3. Set date range start and end.  
4. Add optional keyword filters or user filters.  
5. Click **Preview** to generate a list.  
6. Confirm and **Execute**.

## Server cleanup flow

1. Open Server cleanup.  
2. Select servers to process.  
3. Choose whether to delete your messages in channels before leaving.  
4. Optionally select channels to exclude.  
5. Preview then **Execute**.

## Relationship purge flow

1. Open Relationship purge.  
2. Choose friends, blocked, or pending lists.  
3. Preview and confirm.

## Profile wipe flow

1. Open Profile wipe.  
2. Select fields to clear.  
3. Preview and **Execute**.

---

# Advanced usage

- **Token mode** for advanced users who understand token risks.  
- **RPC mode** for local client integration.  
- **Scripting** advanced automation via local CLI helper for power users.  
- **Custom rate profiles** for users in different regions or with different account histories.  
- **Headless mode** for automated workflows (advanced, opt‑in).

---

# Screenshots and demo

Place screenshots in `/assets/screenshots` and link them here. Include a short GIF showing a purge preview and progress bar. Include a 60‑second demo video on the website and YouTube. Use clear captions and alt text for accessibility.

---

# Comparison with alternatives

**Why Friendly Discord Bot is the best choice for users who want to delete Discord history**

| Attribute | Friendly Discord Bot | Manual deletion | Browser scripts & bots |
|---|---:|---:|---:|
| GUI | **Yes** | No | Varies |
| Bulk DM deletion | **Yes** | No | Limited |
| Rate‑limit aware | **Yes** | N/A | Often no |
| Secure local storage | **Yes** | N/A | Often no |
| Open source | **Yes** | N/A | Varies |
| Preview & dry run | **Yes** | No | Varies |
| Cross platform | **Yes** | N/A | Varies |

**Feature level comparison**

| Feature | Friendly Discord Bot | Other GUI tools | CLI scripts |
|---|---:|---:|---:|
| Preview before delete | **Yes** | Varies | No |
| OAuth2 recommended | **Yes** | Rare | Rare |
| OS keychain storage | **Yes** | Rare | No |
| Rate limit handling | **Yes** | Varies | Often poor |
| Community support | **Yes** | Varies | Varies |

**Detailed capability matrix**

| Capability | Friendly Discord Bot | Browser extensions | Manual | Third‑party cloud services |
|---|---:|---:|---:|---:|
| Delete DMs in bulk | ✅ | ❌ | ❌ | ❌ |
| Delete server messages | ✅ | ❌ | ❌ | ❌ |
| Leave servers in bulk | ✅ | ❌ | ❌ | ❌ |
| Wipe profile fields | ✅ | ❌ | ❌ | ❌ |
| Local processing only | ✅ | ❌ | ✅ | ❌ |
| Rate limit aware | ✅ | ❌ | N/A | ❌ |
| Signed releases | ✅ | Varies | N/A | Varies |

---

# SEO and discoverability notes

**Primary target keywords**

- delete discord messages  
- delete discord history  
- discord message purge  
- discord dm deleter  
- discord data removal  
- discord privacy tool  
- discord purge tool

**Secondary keywords**

- discord cleanup tool  
- discord bulk delete  
- discord dm purge  
- discord server leave tool  
- discord account cleanup

**Long tail keywords**

- how to delete all discord messages at once  
- how to remove discord history safely  
- best discord message deletion tool  
- delete discord dms in bulk safely

**On‑page SEO recommendations**

- Use exact match keywords in headings (H1, H2, H3).  
- Add long‑form how‑to guides and step‑by‑step tutorials.  
- Publish a sitemap and JSON‑LD structured data for the website.  
- Add release notes and changelog pages for freshness signals.  
- Add canonical tags and social meta tags for sharing.

**Off‑page SEO recommendations**

- Submit to Product Hunt, Hacker News, Reddit, and relevant directories.  
- Acquire backlinks from privacy blogs, tech sites, and YouTube descriptions.  
- Encourage GitHub stars, forks, and community discussions.

**Content opportunities**

- “How to delete all Discord messages in 2026 — step‑by‑step”  
- “Why rate limit handling matters for Discord deletion tools”  
- “How to safely back up Discord messages before deleting”  
- “Comparison: Discord cleanup tools and scripts”  
- “Case study: reclaiming privacy with Friendly Discord Bot”

---

# Release and update policy

- **Semantic versioning** — Use MAJOR.MINOR.PATCH.  
- **Release artifacts** — Provide signed binaries for Windows, macOS, and Linux.  
- **Changelog** — Maintain `CHANGELOG.md` with clear notes.  
- **Security advisories** — Publish advisories and patches promptly.  
- **Support window** — Provide community support via Discussions and Discord.

---

# Installation for end users

## Windows

1. Download `friendly-discord-bot-setup.exe` from Releases.  
2. Run installer and follow prompts.  
3. Launch from Start Menu.

## macOS

1. Download `friendly-discord-bot.dmg` from Releases.  
2. Mount and drag to Applications.  
3. Open from Applications.

## Linux

1. Download the AppImage or distribution package from Releases.  
2. Make AppImage executable:  
   ```bash
   chmod +x FriendlyDiscordBot.AppImage
   ./FriendlyDiscordBot.AppImage
Or install via distro package if provided.

Developer build and contribution
Prerequisites
Rust toolchain (stable)

Node.js LTS

Yarn or npm

Tauri prerequisites for your platform

Build steps
bash
git clone https://github.com/TourIT-NL/friendly-discord-bot.git
cd friendly-discord-bot

# install frontend deps
yarn install

# build frontend
yarn build

# build backend
cargo build --release

# package
yarn tauri build
Contributing guidelines
Fork the repo and create feature branches.

Write tests for new features.

Follow code style and linting rules.

Open a pull request with a clear description and changelog entry.

Use issue templates for bug reports and feature requests.

Code of conduct
Be respectful and constructive. Follow the Contributor Covenant code of conduct in CODE_OF_CONDUCT.md.

Testing and QA
Unit tests for core logic and dispatcher.

Integration tests for API interactions using mocked endpoints.

End‑to‑end tests for UI flows using headless automation.

Security tests: static analysis and dependency scanning.

Manual QA on Windows, macOS, and Linux with test accounts.

Troubleshooting
Common issues and fixes

Login fails — Ensure OAuth2 redirect is allowed and system time is correct.

Rate limit errors — Use the conservative rate profile and enable backoff.

Missing messages in preview — Ensure the account has access to the conversation and the app has permission to read messages.

App crashes — Check logs in the app data folder and open an issue with logs attached.

Log locations

Platform	Log path
Windows	%APPDATA%/FriendlyDiscordBot/logs
macOS	~/Library/Logs/FriendlyDiscordBot
Linux	~/.config/FriendlyDiscordBot/logs
FAQ
Is this safe for my account?  
The app is designed to minimize risk through rate‑limit handling and conservative defaults. No tool can guarantee zero risk. Use OAuth2 and follow the safety checklist.

Does the app store my messages?  
No. Messages are processed locally and not uploaded.

Will Discord ban me?  
No guarantee. The app reduces risk but cannot eliminate it. Use at your own discretion.

Can I recover deleted messages?  
Deleted messages cannot be recovered through the app. Back up any messages you want to keep before deletion.

Is the app open source?  
Yes — MIT license. Contributions welcome.

How do I report a security issue?  
See SECURITY.md and the responsible disclosure section below.

Legal and risk disclaimer
Important — Use of this software may violate Discord terms of service or community guidelines. The maintainers are not responsible for account actions taken by Discord. Users accept responsibility for their actions and must ensure compliance with local laws and platform terms.

No warranty — The software is provided as‑is without warranty of any kind.

Responsible disclosure
If you discover a security vulnerability please follow the process in SECURITY.md. Provide a clear reproduction and avoid public disclosure until a fix is available. The maintainers will acknowledge receipt and provide a timeline for remediation.

Changelog template
Use CHANGELOG.md with the following format for each release:

markdown
## [Unreleased]

### Added
- Feature description

### Changed
- Behavior changes

### Fixed
- Bug fixes

### Security
- Security fixes and CVE references
Marketing and launch checklist
Pre‑launch

Finalize README, screenshots, logo, and demo video.

Register domain and deploy landing page with JSON‑LD and sitemap.

Prepare Product Hunt assets and press kit.

Create Discord server, subreddit, and GitHub Discussions.

Launch day

Publish release on GitHub with signed binaries.

Launch Product Hunt and coordinate Reddit, Hacker News, and Twitter posts.

Publish demo video on YouTube and embed on site.

Engage with early users and respond to feedback.

Post‑launch

Outreach to YouTube creators and privacy blogs.

Submit to AlternativeTo, Slant, and software directories.

Publish SEO blog posts and tutorials.

Monitor analytics and iterate.

SEO content plan
Core pages to publish on website

Home landing page with clear CTA and download links

Features page with comparison table

How to delete all Discord messages guide

How to remove Discord history safely guide

FAQ and safety checklist

Changelog and release notes

Security and privacy page

Blog for long tail content and tutorials

Suggested blog posts

How to delete all Discord messages in 2026 — step‑by‑step

Why rate limit handling matters for Discord deletion tools

How to safely back up Discord messages before deleting

Comparison of Discord cleanup tools and scripts

Case study: reclaiming privacy with Friendly Discord Bot

On‑page elements

Use H1, H2, H3 with target keywords

Add JSON‑LD product schema

Add Open Graph and Twitter card meta

Add canonical tags and sitemap

Community and support
Official channels

GitHub Discussions for support and feature requests

Official Discord server for real‑time help and community

Subreddit r/FriendlyDiscordBot for announcements and user stories

YouTube channel for tutorials and demos

How to get help

Search the FAQ and Wiki

Open an issue with logs and reproduction steps

Ask in Discord support channel for quick help

Contact and maintainers
Maintainers

Primary maintainer: TourIT NL (GitHub profile)

Contact: use GitHub Discussions or the project contact email in repository settings

Contributing  
See CONTRIBUTING.md for contribution guidelines, code style, and PR process.

Appendix — JSON‑LD for website
json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "Friendly Discord Bot",
  "url": "https://friendlydiscordbot.com",
  "description": "A secure user friendly desktop app to delete Discord messages, DMs, servers, relationships and profile data with rate limit aware processing",
  "applicationCategory": "Utilities",
  "operatingSystem": "Windows macOS Linux",
  "license": "MIT",
  "author": {
    "@type": "Organization",
    "name": "TourIT NL",
    "url": "https://github.com/TourIT-NL"
  },
  "offers": {
    "@type": "Offer",
    "price": "0",
    "priceCurrency": "USD"
  }
}
Appendix — command line examples
Build for development

bash
yarn install
yarn dev
Build release

bash
yarn build
yarn tauri build
Run tests

bash
cargo test
yarn test
Appendix — release checklist
[ ] Update CHANGELOG.md with release notes

[ ] Bump version in package.json and Cargo.toml

[ ] Build signed binaries for Windows, macOS, and Linux

[ ] Create GitHub Release with assets and release notes

[ ] Publish blog post and social announcements

[ ] Update website download links and changelog

[ ] Tag release and create release branch

Final notes for repository maintainers
SEO best practices for README

Keep the README H1 as the project name and include the primary keyword phrase within the first 20 words.

Use descriptive headings that match search queries such as "How to delete Discord messages" and "Discord data removal tool".

Include a comparison table and FAQ to capture featured snippet opportunities.

Add a short demo video and screenshots to increase time on page and click‑through rates.

Keep the README updated with release notes and changelog links.

Community growth tips

Encourage users to star the repo and share feedback in Discussions.

Run periodic community events and AMAs in the Discord server.

Offer early access to reviewers and creators to generate reviews and backlinks.

Security and trust

Maintain SECURITY.md and PRIVACY.md and link them prominently in the README.

Provide signed releases and checksums for binaries.

Respond to security reports promptly and publish advisories.
