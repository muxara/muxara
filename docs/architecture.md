# Architecture

This document covers the full architecture of Muxara — a Tauri v2 desktop app with a Rust backend and React frontend.

## Tech Stack

- **Desktop framework**: Tauri v2 (Rust backend + web frontend in a single binary)
- **Frontend**: React + TypeScript + Tailwind CSS (via Vite)
- **Backend**: Rust (Tauri commands invoked from the frontend via `@tauri-apps/api`)
- **Session layer**: tmux (managed by the backend, hidden from the user)

## Project Structure

```
muxara/
├── src/                         Frontend (React + TypeScript)
│   ├── main.tsx                 React entry point, mounts App
│   ├── App.tsx                  Root component — uses useSessions hook, renders SessionGrid
│   ├── types.ts                 Shared TypeScript types (Session, SessionState, RuntimeState)
│   ├── styles.css               Tailwind base styles
│   ├── hooks/
│   │   └── useSessions.ts       Polling hook — calls get_sessions every 1.5s, returns sessions/loading/error
│   └── components/
│       ├── SessionGrid.tsx      Grid layout for session cards, handles loading/error/empty states
│       ├── SessionCard.tsx      Two-zone card: orientation (status, title, dir, recency) + context (output)
│       └── StatusBadge.tsx      Colored status dot per session state
├── src-tauri/                   Backend (Rust)
│   ├── src/
│   │   ├── main.rs              Entry point, delegates to lib.rs
│   │   ├── lib.rs               Tauri app builder, module registration, managed state
│   │   ├── commands.rs          Tauri command handlers (invoked from frontend)
│   │   ├── session.rs           Frontend-facing data model (Session, SessionState, RuntimeState)
│   │   ├── store.rs             In-memory session store with reconciliation logic
│   │   └── tmux/
│   │       ├── mod.rs           Module declaration
│   │       ├── classifier.rs    State classification: regex-based pattern matching on pane output
│   │       └── client.rs        tmux interaction: discovery, capture, ANSI stripping, process detection
│   └── tests/                   Integration tests (see tests/README.md)
├── docs/                        Project documentation
├── spike/                       Phase 0 spike code and fixtures
├── vite.config.ts               Vite config (Tauri plugin)
├── tailwind.config.js           Tailwind config
└── package.json                 Frontend dependencies
```

## Frontend (ticket #4)

The frontend is a React SPA bundled by Vite and rendered inside the Tauri webview.

### Polling hook (`src/hooks/useSessions.ts`)

`useSessions()` encapsulates the polling loop. It calls `invoke<Session[]>("get_sessions")` every 1.5 seconds and returns `{ sessions, loading, error }`. Loading is `true` only until the first successful or failed fetch. The `active` flag and `clearInterval` cleanup prevent state updates after unmount. `App.tsx` consumes this hook and passes the result to `SessionGrid`.

### Components

- **`SessionGrid`** — renders a responsive CSS grid (`1 / 2 / 3` columns at sm/lg breakpoints) of `SessionCard` components. Handles three non-data states: loading (shown during first fetch), error (shown when the backend call fails), and empty (no sessions exist). NeedsInput sessions appear first (sorting is handled by the backend).
- **`SessionCard`** — two-zone card layout, clickable to focus the session:
  - **Click handler**: calls `invoke("focus_session", { sessionId })` to open a Terminal.app window attached to the tmux session. Brief scale-down + brightness animation on click.
  - **Orientation zone** (top): status dot (`StatusBadge`), session title, abbreviated working directory, state label + recency (e.g. "Working · 2m ago"). NeedsInput cards additionally show the input type (Permission / Question).
  - **Context zone** (bottom, separated by a subtle divider): last terminal output lines in monospace.
  - State styling (left border color, background tint) is driven by a `stateConfig` record keyed by `SessionState`.
- **`StatusBadge`** — colored dot indicating session state. Working state pulses via `animate-pulse`.

### Types (`src/types.ts`)

All types mirror the Rust `Session` struct with camelCase field names:
- `SessionState` — `"needs-input" | "working" | "idle" | "errored" | "unknown"`
- `NeedsInputType` — `"permission" | "question"`
- `RuntimeState` — `{ tmuxAlive: boolean, claudeAlive: boolean }`
- `Session` — the full session object received from the backend

## Backend (ticket #5)

### `tmux/client.rs` — tmux interaction layer

Shells out to tmux and system commands to gather raw session data. All tmux interaction is isolated here.

**Key functions:**
- `list_sessions()` / `list_panes()` — discover tmux sessions and panes via `tmux list-sessions -F` / `tmux list-panes -F` with tab-delimited format strings
- `capture_pane(target)` — capture the last 200 lines of a pane's output, strip ANSI codes, trim trailing blank lines (Claude Code's TUI pads panes with empty lines), hash the result
- `is_claude_running(ps_output, pane_pid)` — walk the process tree from a pane's shell PID to detect if a `claude` process is running as a descendant
- `ensure_server()` — start the tmux server if not already running (Muxara manages tmux on behalf of the user)
- `strip_ansi(input)` — remove ANSI escape sequences using a compiled regex
- `hash_output(normalized)` — SHA-256 hash (first 16 hex chars) for change detection

**Raw data structs** (`TmuxSessionInfo`, `TmuxPaneInfo`, `CapturedPane`) are internal and never serialized to the frontend.

**Error handling:** `TmuxError` enum with variants `NotInstalled`, `ServerNotRunning`, `CommandFailed`, `ParseError`. Functions that query tmux return `Ok(empty_vec)` when the server isn't running, rather than propagating the error.

**Testability:** Parsing logic is extracted into pure functions (`parse_sessions_output`, `parse_panes_output`, `find_claude_in_process_tree`) that accept string input, making them testable without a live tmux server.

### `store.rs` — in-memory session store

Maintains a `HashMap<String, TrackedSession>` keyed by pane target string (e.g., `sess1:0.0`). Registered as `Mutex<SessionStore>` in Tauri's managed state.

**`TrackedSession`** holds:
- tmux identity (session name, pane target, pane PID, working directory)
- App-level metadata (display name, created_at, last_seen_at, last_changed_at)
- Capture state (output hash, last N output lines, pane title)
- Runtime state (claude_alive, tmux_alive)
- Classification fields (state, needs_input_type, is_in_plan_mode, consecutive_idle_count) — defaulted to `Unknown`, populated by the classifier each reconcile cycle

**`reconcile()`** is called each poll cycle:
1. For each live tmux pane, upsert a `TrackedSession`
2. Update runtime fields from fresh data
3. Compare output hash — if changed, update `last_changed_at` and `last_output_lines`
4. Prune sessions whose pane target no longer appears in tmux

**`to_sessions()`** converts tracked sessions to frontend-ready `Session` structs with ISO 8601 timestamps. Sessions are sorted by state priority (NeedsInput > Errored > Working > Idle > Unknown), then alphabetically by name.

### `session.rs` — data model

Frontend-facing types serialized via serde:
- `SessionState` — `NeedsInput`, `Working`, `Idle`, `Errored`, `Unknown` (kebab-case)
- `NeedsInputType` — `Permission`, `Question` (camelCase)
- `RuntimeState` — `tmux_alive`, `claude_alive` (camelCase)
- `Session` — the full session object sent to the frontend

### `commands.rs` — Tauri commands

`focus_session(session_id)` opens a new iTerm2 window attached to the tmux session. It extracts the session name from the pane target ID, verifies the session exists, then uses AppleScript (`osascript`) to launch iTerm2 with `tmux attach -t <session>`. Returns an error string if the session is not found or the terminal fails to open.

`get_sessions()` is called by the frontend every 2 seconds:
1. Check if tmux is alive; start server if needed
2. List all panes
3. Fetch process table once (`ps -ax`), check each pane for a running `claude` process
4. Capture pane output for each pane
5. Reconcile with the session store
6. Return frontend-ready sessions

## Data Flow

```
Frontend (2s poll)
  │
  ▼
get_sessions (Tauri command)
  │
  ├── is_tmux_alive() / ensure_server()
  ├── list_panes(None) → Vec<TmuxPaneInfo>
  ├── get_process_table() → ps output (once per cycle)
  │   └── is_claude_running(ps_output, pane_pid) per pane
  ├── capture_pane(target) per pane → CapturedPane
  │   ├── strip_ansi()
  │   └── hash_output()
  │
  ▼
SessionStore::reconcile(panes, captures, claude_status, tmux_alive)
  │
  ├── per pane: classifier::classify(output, hash, previous_state, timing)
  │   └── returns SessionState + NeedsInputType + isInPlanMode
  │
  ▼
SessionStore::to_sessions() → Vec<Session> → frontend
```

## Key Patterns

- **`LazyLock` for regex:** The ANSI stripping regex is compiled once and reused across all calls via `std::sync::LazyLock`.
- **Single `ps -ax` per poll:** The process table is fetched once per poll cycle, then each pane's `claude` status is checked from the parsed table. This is O(n) in process count, done once — not O(panes * processes).
- **`Mutex<SessionStore>` managed state:** The session store persists across poll cycles via Tauri's `.manage()`. A `Mutex` (not `RwLock`) is used because every access both reads and writes.
- **Graceful degradation:** If tmux is not installed or the server isn't running, the system returns an empty session list rather than erroring.

## Testing Strategy

- **Unit tests**: Inline `#[cfg(test)] mod tests` in each source file. These test pure functions (parsing, ANSI stripping, hashing, process tree walking, store reconciliation) with mock data — no live tmux required.
- **Integration tests**: `src-tauri/tests/` directory. Each `.rs` file compiles as a separate crate testing the public API across modules. See `tests/README.md` for conventions. Tests requiring a live tmux server should be gated with `#[ignore]`.

Run all tests with `cargo test`. Run only integration tests with `cargo test --test '*'`.

## State Classification (ticket #6)

### `tmux/classifier.rs` — state classifier

Determines a session's state from its pane output and temporal context. Ported from the Phase 0 spike (`spike/src/classifier.ts`).

**Input:** `ClassifierInput` containing:
- `normalized_output` — ANSI-stripped pane content
- `output_hash` / `previous_hash` — for delta detection
- `previous_state` — for debounce logic
- `seconds_since_last_change` — time since output last changed
- `consecutive_idle_count` — for Working→Idle debounce

**Output:** `ClassifierResult` with `state`, `needs_input_type`, `is_in_plan_mode`, and `debounce_applied` (true when Working state was held by debounce rather than an active working signal).

**Priority order:** NeedsInput > Errored > Working > Idle > Unknown

**Signal detection:**

| Signal type | Patterns | State |
|---|---|---|
| Permission prompt | `Do you want to proceed?`, `Do you want to create`, `This command requires approval`, `Esc to cancel · Tab to amend` | NeedsInput (Permission) |
| AskUserQuestion | `☐` marker, `Enter to select · ↑/↓ to navigate` | NeedsInput (Question) |
| Shell error | `^error:` at line start | Errored |
| Tool error | `⎿ Error:`, `Error: Exit code N` | Errored |
| Output delta | Hash changed + change within 5s threshold | Working |
| Plan mode | `Entered plan mode`, `✻` spinner (hard); `⏸ plan mode on` status bar (soft fallback) | isInPlanMode=true |
| Claude markers | `❯`, `▐▛███▜▌`, `⏺` present + no other signals | Idle |
| No markers | No recognizable Claude output | Unknown |

**Working→Idle debounce:** When previous state is Working and classifier would say Idle, require 2 consecutive idle readings before transitioning. This prevents flicker during brief pauses between tool calls. Hard signals (NeedsInput, Errored) bypass the debounce immediately. The classifier reports `debounce_applied = true` when it holds Working state this way, so the store can correctly increment `consecutive_idle_count` — the counter is driven by the debounce flag, not the final state, to avoid a circular dependency where the debounced Working result would prevent the counter from ever reaching the threshold.

**Classification focus:** Only the last 50 lines of output are checked for most patterns (the "tail"), since the most recent state is at the bottom. Shell-level errors and plan mode transitions are checked against full output.

**Integration:** The classifier runs during `SessionStore::reconcile()` for each pane that has captured output. The store tracks `consecutive_idle_count` per session to support debounce.

## Spike Reference

The approach was validated in Phase 0 spikes — see `spike/findings.md` for details on:
- ANSI stripping requirements
- Process tree inspection reliability
- State classification patterns and signal taxonomy
- Debounce mechanics for Working→Idle transitions
- Fixture data in `spike/fixtures/` for testing
