# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Muxara is a lightweight desktop application that serves as a control plane for developers running multiple parallel Claude Code sessions. It provides a persistent, always-visible dashboard showing session cards with status, context, and quick-switch capabilities. It builds on top of tmux as the session execution layer.

See `plan.md` for the full product brief and design vision.

## Architecture

Tauri v2 desktop app: Rust backend + React/TypeScript/Tailwind frontend.

See `docs/architecture.md` for the full architecture reference including project structure, module responsibilities, data flow, key patterns, and testing strategy.

## Design Constraints

- Optimized for single-monitor setups — compact, always-on-top window that doesn't dominate screen space
- Scales gracefully with growing number of sessions
- Reduces cognitive load rather than adding process overhead
- tmux is managed by Muxara and hidden from the user
- Intended for open-source publication under the name "Muxara"

## Development Commands

```sh
# Run the full app (frontend + backend) in dev mode
npm run tauri dev

# Run Rust backend tests
cd src-tauri && cargo test

# Build for production
npm run tauri build

# Frontend only (without Tauri shell, for UI work)
npm run dev
```

## Development Practices

### Documentation updates

Every ticket must include documentation updates alongside code changes. This keeps docs in sync with the codebase and ensures that developers and Claude Code sessions working on future tickets have accurate context.

- **`CLAUDE.md`**: Update the "Current State" section and any architectural descriptions that change
- **`docs/architecture.md`**: Update when backend modules, data flow, or key patterns change
- **Architecture docs should cover**: module responsibilities, data flow, key patterns/conventions, error handling strategy, and boundaries between components

## Current State

The Tauri scaffold is in place (ticket #4), the tmux integration layer is implemented (ticket #5), the state classifier is wired up (ticket #6), the frontend is connected to live backend data (ticket #7), and session cards use the two-zone layout with status indicators (ticket #8). The Rust backend can discover tmux sessions, capture pane output with ANSI stripping, detect running Claude processes, classify session state (NeedsInput, Working, Idle, Errored, Unknown), and maintain an in-memory session store that reconciles with live tmux state. The classifier uses regex-based pattern matching on pane output with temporal delta detection and Working→Idle debounce to prevent flicker. The frontend polls the backend every 1.5s via a `useSessions` hook and renders live session data with loading, error, and empty states. Session cards display an orientation zone (status dot, title, working directory, state + recency) and a context zone (last terminal output lines), with NeedsInput sessions sorted to the top.

Clicking a session card opens a new iTerm2 window attached to that tmux session (ticket #9). A "+" button in the header creates new Claude Code sessions with an optional name and working directory (ticket #10).

**Not yet implemented:** attention signals, kill/rename sessions.
