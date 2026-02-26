# Changelog

All notable changes to the Discord Privacy and Cleanup Utility will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.3] - 2026-02-26

### Added

- **High-Fidelity Fingerprinting Engine**: New module `src-tauri/src/api/rate_limiter/fingerprint.rs` for realistic browser emulation.
- **Dynamic Locale Detection**: Automatic identification of system locale for `Accept-Language` parity.
- **Synthetic Cookie Generator**: Dynamically generates `__dcfduid`, `__sdcfduid`, and `_cfuvid` cookies.
- **Client Hints Support**: Added `Sec-CH-UA`, `Sec-CH-UA-Mobile`, and `Sec-CH-UA-Platform` headers.
- **Fetch Metadata Implementation**: Integrated `Sec-Fetch-Site`, `Sec-Fetch-Mode`, and `Sec-Fetch-Dest` for all requests.
- **Meaningfully Online Heartbeat**: Background actor task to simulate active session during operations.
- **Discord Error Mapping**: `AppError::from_discord_json` now maps hundreds of internal Discord API codes to user actions.
- **Telemetry Blocklist**: Global block for `/beaker` and `/metrics` to ensure zero behavioral leakage.

### Changed

- **API Request Signature**: Refactored `ApiHandle::send_request` to accept 10 arguments, enabling granular control over locale, referer, and browser profiles.
- **Standardized Headers**: Unified the header injection logic across all 45+ API calls in the backend.
- **Vault Persistence**: Updated `Vault` to handle user-specific localization and regional preferences.
- **OAuth2 Token Validation**: Implemented strict regex-based format verification for incoming tokens.

### Fixed

- **Rate Limiter Precision**: Improved `X-RateLimit-Reset-After` parsing to handle floating-point precision with 100ms jitter.
- **CI/CD Path Errors**: Corrected directory mapping in `.github/workflows/main.yml`.
- **Typo/Link Verification**: Fixed several documentation link failures detected by `lychee`.

### Removed

- **Hardcoded User-Agents**: Replaced with a dynamic profile-based rotation system.
- **Metric Telemetry**: Disabled all background reporting to Discord analytics.

## [1.0.2] - 2026-02-25

- Initial implementation of the Rate Limiting Actor.
- Basic Bulk Message Deletion logic.
- OAuth2 Identity management via Keyring.
