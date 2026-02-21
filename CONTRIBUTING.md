# Contributing to Discord Purge

We welcome contributions to the Discord Purge project! Your help is invaluable in making this tool better for everyone.

Please take a moment to review this document to understand how you can contribute.

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

There are several ways you can contribute:

1.  **Reporting Bugs:** If you find a bug, please open an issue on our [GitHub Issue Tracker](https://github.com/TourIT-NL/friendly-discord-bot/issues).
    - Provide a clear and concise description of the bug.
    - Include steps to reproduce the issue.
    - Mention your operating system and application version.
    - (Optional) Include screenshots or error logs.

2.  **Suggesting Features:** Have an idea for a new feature or improvement?
    - Open an issue on the [GitHub Issue Tracker](https://github.com/TourIT-NL/friendly-discord-bot/issues).
    - Clearly describe the feature and its benefits.
    - Explain how it would be used.

3.  **Submitting Pull Requests:**

## Development Workflow

We follow a feature-branch workflow to keep our `main` branch stable and to facilitate collaboration.

1.  **Fork the Repository:** Start by forking the official repository to your GitHub account.
2.  **Clone Your Fork:** Clone your forked repository to your local machine:
    ```bash
    git clone https://github.com/YOUR_USERNAME/friendly-discord-bot.git
    cd friendly-discord-bot/discord-privacy-util
    ```
3.  **Create a New Branch:**
    - From the `main` branch, create a new branch for your feature or bug fix. Use descriptive names:
      ```bash
      git checkout main
      git pull origin main # Ensure your main is up-to-date
      git checkout -b feature/your-descriptive-feature-name # For new features
      # OR
      git checkout -b bugfix/issue-number-short-description # For bug fixes
      ```
    - Work on your changes in this new branch.

## Testing

Before submitting a Pull Request, ensure your changes are well-tested and do not introduce regressions.

- **Frontend Tests (TypeScript/React):**
  ```bash
  npm test
  ```
  This command will run Vitest tests and generate coverage reports in `coverage/frontend`.
- **Backend Tests (Rust):**
  ```bash
  cd src-tauri
  cargo test
  ```
  This command will run your Rust tests. During CI, coverage reports are also generated.
- **Code Style & Linting:**
  - Ensure your code adheres to the project's style guidelines. Run the format and lint checks locally:
    ```bash
    npm run format:check
    npm run lint
    cd src-tauri
    cargo fmt -- --check
    cargo clippy -- -D warnings
    ```
  - To automatically fix formatting issues: `npm run format:fix`

## Commit Message Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) to provide a clear and structured commit history. Each commit message should follow this format:

```
<type>(<scope>): <short description>

[optional body]

[optional footer(s)]
```

**Examples:**

- `feat(auth): Implement OAuth2 PKCE flow`
- `fix(ui): Correct button alignment in login screen`
- `docs(readme): Update installation instructions`
- `chore(ci): Add Prettier check to workflow`

## Submitting a Pull Request

Once your changes are complete, thoroughly tested, and adhere to the commit guidelines:

1.  **Push Your Branch:** Push your feature/bugfix branch to your forked repository on GitHub.
2.  **Open a Pull Request (PR):**
    - Go to the original `friendly-discord-bot` repository on GitHub.
    - You should see a prompt to open a Pull Request from your branch.
    - Ensure your PR targets the `main` branch of the upstream repository.
3.  **Provide Details:** In your Pull Request description, provide:
    - A clear summary of the changes.
    - References to any related issues (e.g., `Fixes #123`, `Closes #456`).
    - Any specific instructions for testing your changes.

## Development Setup

For detailed instructions on how to set up your local development environment, including prerequisites and how to run the application, please refer to the [README.md](README.md) file.

## Thank You!

Your contributions are greatly appreciated and help make Discord Purge a better tool for everyone.
