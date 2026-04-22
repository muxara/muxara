# Contributing to Muxara

Welcome, and thank you for considering a contribution to Muxara. Whether you are fixing a bug, improving documentation, or proposing a new feature, your help is genuinely appreciated. Muxara is open source under the MIT license and we are happy to have contributors of all experience levels involved. Please review our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

## Prerequisites

Muxara is a macOS-only desktop application. You will need the following installed before you can build and run it:

| Requirement | Minimum Version | Notes |
|---|---|---|
| **macOS** | 12 (Monterey) | Required for Tauri v2 and the AppleScript terminal integration |
| **Node.js** | 20+ | Used for the Vite dev server and frontend tooling |
| **Rust** | Stable toolchain | Install via [rustup](https://rustup.rs/) |
| **tmux** | Any recent version | `brew install tmux` -- Muxara manages tmux behind the scenes |
| **Terminal.app or iTerm2** | Any recent version | Session switching opens terminal windows via AppleScript. Terminal.app is the default; configurable in settings |
| **Claude Code CLI** | Any recent version | Needed if you want to create real test sessions |

## Development Setup

1. Clone the repository and install frontend dependencies:

   ```sh
   git clone https://github.com/muxara/muxara.git
   cd muxara
   npm install
   ```

2. Start the app in development mode:

   ```sh
   npm run tauri dev
   ```

   This single command starts both the Vite dev server (for the React frontend with hot reload) and the Tauri Rust backend. Changes to frontend code will hot-reload in the webview. Changes to Rust code will trigger a recompile and restart.

3. If you only need to work on the frontend UI without the Tauri shell:

   ```sh
   npm run dev
   ```

   This starts the Vite dev server alone, which is useful for layout and styling work. Backend commands will not be available in this mode.

4. To build for production:

   ```sh
   npm run tauri build
   ```

## Project Structure

```
muxara/
├── src/                  Frontend (React + TypeScript + Tailwind CSS)
│   ├── main.tsx          React entry point
│   ├── App.tsx           Root component
│   ├── types.ts          Shared TypeScript types
│   ├── hooks/            Custom React hooks (polling, preferences)
│   └── components/       UI components (SessionGrid, SessionCard, SettingsPanel, etc.)
├── src-tauri/            Backend (Rust)
│   ├── src/
│   │   ├── lib.rs        Tauri app builder and module registration
│   │   ├── commands.rs   Tauri command handlers (invoked from the frontend)
│   │   ├── session.rs    Session data model and state types
│   │   ├── store.rs      In-memory session store with reconciliation
│   │   ├── preferences.rs  User preferences with JSON persistence
│   │   ├── git.rs        Git repo, branch, and worktree detection
│   │   └── tmux/         tmux integration (discovery, capture, classification)
│   └── tests/            Integration tests
├── docs/                 Project documentation
├── spike/                Phase 0 spike code and fixtures
└── package.json          Frontend dependencies and scripts
```

For a detailed breakdown of module responsibilities, data flow, and key patterns, see [docs/architecture.md](docs/architecture.md).

## Running Tests

Run the Rust backend test suite with:

```sh
cd src-tauri && cargo test
```

A few things to keep in mind:

- Tests must be run on macOS. The tmux integration and AppleScript layers are platform-specific.
- Some integration tests interact with a live tmux server. Make sure tmux is installed and available on your PATH.
- There is no frontend test suite at this time. Manual verification in the running app is the current approach for UI changes.

## Code Style

**Rust:**

- Run `cargo fmt` to format your code before committing. CI enforces consistent formatting.
- Run `cargo clippy` to catch common mistakes and style issues. CI enforces clippy as well.
- Follow the patterns established in the existing codebase.

**TypeScript / React:**

- Follow the existing patterns in the `src/` directory.
- Use Tailwind CSS utility classes for styling, consistent with the rest of the UI.
- Keep components focused and composable.

**General:**

- You do not need to add comments or docstrings to code you did not change.
- If you are unsure about a convention, look at how similar code is handled elsewhere in the project.

## Making Changes

1. **Fork the repository** on GitHub and clone your fork locally.

2. **Create a feature branch** from `main`:

   ```sh
   git checkout -b my-feature
   ```

3. **Make your changes.** Keep commits focused and atomic.

4. **Run tests** to make sure nothing is broken:

   ```sh
   cd src-tauri && cargo test
   ```

5. **Update documentation** if your change affects architecture, data flow, or user-facing behavior. The two files to check are:
   - `CLAUDE.md` -- update the "Current State" section to reflect what changed.
   - `docs/architecture.md` -- update if you changed backend modules, data flow, or key patterns.

6. **Add a CHANGELOG.md entry** under the `[Unreleased]` section if your change is user-facing (new feature, bug fix, behavior change).

7. **Commit with a clear message** that describes the *why*, not just the *what*. For example, prefer "Debounce classifier to prevent flicker on slow connections" over "Update classifier.rs".

8. **Push your branch and open a pull request** against `main`.

## What Makes a Good PR

- **Small and focused.** One concern per pull request. A PR that fixes a bug and also refactors an unrelated module is harder to review and more likely to stall.
- **Tests pass.** Make sure `cargo test` is green before requesting review.
- **Docs updated alongside code.** If your change affects how the system works or what users see, update CLAUDE.md and docs/architecture.md in the same PR.
- **CHANGELOG entry for user-facing changes.** This helps maintainers write release notes.
- **Clear description.** Explain what you changed and why. If the PR addresses a GitHub issue, reference it (e.g., "Closes #42").

## Issue Labels

The issue tracker uses these labels to organize work:

| Label | Description |
|---|---|
| `bug` | Something is broken or behaving incorrectly |
| `enhancement` | A new feature or improvement to existing functionality |
| `good first issue` | Suitable for newcomers to the project |
| `help wanted` | Maintainers would appreciate community help here |
| `metadata` | Project configuration, CI, tooling |
| `documentation` | Improvements to docs, guides, or inline help |
| `community` | Community health files, templates, contributor experience |
| `ci/cd` | Continuous integration and deployment pipelines |
| `distribution` | Packaging, signing, release artifacts |
| `release` | Release management and versioning |

## Getting Help

If you have questions about the codebase, are unsure where to start, or want feedback on an idea before writing code, open a [Discussion](https://github.com/muxara/muxara/discussions) on GitHub. No question is too basic -- we would rather help you get unstuck than have you struggle in silence.

Thank you for helping make Muxara better.
