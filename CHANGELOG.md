# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-04-22

### Fixed

- App could not find tmux when launched from Spotlight, Dock, or Homebrew install — PATH resolution now checks common Homebrew/MacPorts/system paths as fallback

## [0.1.0] - 2026-04-22

### Added

- Session dashboard with live-polling session cards showing status, working directory, and terminal output context
- tmux integration layer: automatic session discovery, pane output capture with ANSI stripping, and Claude process detection
- Session state classification engine (NeedsInput, Working, Idle, Errored, Unknown) using regex-based pattern matching on pane output with temporal delta detection
- Working-to-Idle debounce (configurable cool-off period) to prevent status flicker during brief pauses
- ANSI color rendering in session card output for faithful terminal display
- iTerm2 session switching: click a card to open or focus the tmux session in iTerm2
- Single-tab session switching using `tmux switch-client` instead of spawning new windows
- Arrow-key navigation across session cards in grid order (left/right, up/down) with Enter to switch
- Emerald glow on the last-focused card, distinct gray ring on keyboard-selected card
- New session creation with optional name, directory picker, and inline command editing
- Configurable bootstrap command with layered settings model: hardcoded defaults, user preferences, and per-project overrides
- Automatic git worktree isolation for new sessions via Claude Code's `-w` flag, with per-project toggle
- Git metadata on session cards: current branch display and "WT" indicator for worktree sessions
- Kill and rename session actions via right-click context menu, with confirmation dialog on kill
- Worktree cleanup on kill with uncommitted-changes guard
- VS Code-style settings panel with schema-driven UI; configurable poll interval, scroll pause duration, grid columns, context zone height, output lines per card, idle/unknown output visibility, and cool-off period
- Per-project settings management in the settings panel, filtered by `projectCompatible` flag
- Scroll pause on user interaction to prevent content jumping while reading output
- Mouse scrolling support in tmux sessions
- Compact, always-on-top overlay window with custom grid icon and spaced uppercase title bar

### Changed

- Default output lines per card increased from 20 to 30 for better context visibility

### Fixed

- Initial classification on restart: first captures treated as initial state so sessions classify from output content rather than all appearing as Working
- Title bar drag reliability with overlay window style
- Stable sort order for Working and Idle sessions using alphabetical tie-breaking within state tiers

[Unreleased]: https://github.com/muxara/muxara/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/muxara/muxara/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/muxara/muxara/releases/tag/v0.1.0
