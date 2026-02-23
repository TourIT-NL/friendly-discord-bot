# Discord Purge CI/CD Pipeline: GitHub Actions for Automated Builds & Releases

This document details the Continuous Integration and Continuous Deployment (CI/CD) pipeline for the **Discord Purge utility**, implemented using **GitHub Actions**. This robust automation ensures code quality, efficient testing, and streamlined releases for this **Discord message deletion and privacy management tool**.

The `.github/workflows/main.yml` file orchestrates the primary CI/CD flow, defining a series of automated jobs:

1.  **`lint`**: This job performs critical code quality checks by running `clippy` and `rustfmt` for the Rust backend, and `eslint` and `prettier` for the TypeScript frontend. It's configured to fail the build on any linting or formatting warnings, ensuring consistent code standards for the **Discord Purge project**.
2.  **`test`**: This job executes comprehensive unit and integration tests. It runs `cargo test` for the Rust backend and `pnpm test` for the TypeScript frontend. Test coverage reports are collected and uploaded as artifacts, providing insights into the test health of the **Discord cleanup tool**.
3.  **`build`**: This job is responsible for compiling the **Discord Purge application** across multiple platforms. It runs on a matrix for `windows-latest`, `macos-latest`, and `ubuntu-latest`, generating platform-specific binaries for the **desktop application**.
4.  **`release`**: Triggered exclusively on a git tag (e.g., `v1.0.0`), this job orchestrates the final release process. It utilizes the `build` job's artifacts and leverages Tauri's `gh-release` action to create a new GitHub Release, attaching all compiled binaries of the **Discord message deletion tool**.
