# 🛠️ Contributor Onboarding: Welcome to the Team!

Thank you for your interest in contributing to **Discord Purge**. We aim for a "Masterpiece" standard in our engineering, and this guide will help you get started with the right mindset and setup.

---

## 🌟 Our Standards

1.  **Code is Poetry**: We prioritize readability over cleverness. If a piece of logic is complex, it deserves an architectural explanation in the Wiki.
2.  **Safety First**: Any change touching the `auth` or `api` modules requires rigorous manual testing and a unit test update.
3.  **Strict Typing**: We avoid `any` in TypeScript and `unwrap()` in Rust unless absolutely necessary.

---

## 🚀 Local Development Setup

### 1. The Rust Backend

Install `rustup` and ensure you are on the `stable` channel.

```bash
cd src-tauri
cargo run
```

### 2. The Frontend

We use **Vite** for fast HMR (Hot Module Replacement).

```bash
yarn install
yarn dev
```

---

## 🧹 Quality Control Tools

We expect every contributor to run these tools before submitting a PR:

- **`npm run format:fix`**: Runs Prettier to ensure style consistency.
- **`npm run lint`**: Runs ESLint to catch common React/TS pitfalls.
- **`cargo fmt --all`**: Ensures the Rust code follows standard styling.
- **`cargo clippy`**: Our "Robot Mentor" that suggests idiomatic Rust improvements.

---

## 📝 Documenting as Code

If you add a new core feature, you are **required** to:

1.  Update the `README.md` features list.
2.  Add a detailed architectural overview in `wiki_content/`.
3.  Add a new **User Story** to ensure we understand the value provided.

---

## ✅ Pull Request Checklist

- [ ] Does the build pass on Windows, macOS, and Linux?
- [ ] Is every new function documented?
- [ ] Have you added a "Before & After" screenshot for UI changes?
- [ ] Is the PR title following Conventional Commits (e.g., `feat(ui): Add dark mode toggle`)?

_Last updated: February 25, 2026_
