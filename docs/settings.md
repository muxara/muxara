# Settings Reference

Muxara includes a VS Code-style settings panel accessible from the gear icon in the dashboard header. All settings take effect immediately -- no restart required. Preferences are stored in `~/Library/Application Support/com.muxara.app/preferences.json`.

## Sessions

| Setting | Default | Description |
|---------|---------|-------------|
| **Default Command** | `claude` | The command run when creating a new session. This is pre-filled in the new session form and can be edited inline before creation. Can be overridden per project (see [Project Overrides](#project-overrides) below). |
| **Terminal Application** | Terminal.app | Which terminal app opens when you click a session card. Choose between Terminal.app (built-in) and iTerm2. |
| **Use Git Worktrees** | On | When enabled, new sessions created inside a git repository automatically get an isolated [git worktree](https://git-scm.com/docs/git-worktree) at `<repo>/.claude/worktrees/<session-name>`. This prevents file conflicts between parallel sessions on the same repo. Can be overridden per project. |

## Polling

| Setting | Default | Range | Description |
|---------|---------|-------|-------------|
| **Poll Interval** | 1.5 seconds | 0.5 -- 30s | How frequently Muxara checks tmux for session updates. Lower values feel more responsive but use more resources. |
| **Scroll Pause Duration** | 5 seconds | 0 -- 60s | When you scroll inside a card's output area, polling pauses for this duration so the content doesn't jump while you're reading. Set to 0 to disable. |

## Display

| Setting | Default | Range | Description |
|---------|---------|-------|-------------|
| **Grid Columns** | 2 | 1 -- 6 | Number of card columns in the dashboard grid. Adjust based on your window size and how many sessions you typically run. |
| **Context Zone Height** | 192 px | 48 -- 800 px | Maximum height of the scrollable terminal output area within each card. Increase to see more output without scrolling. |
| **Output Lines Per Card** | 30 | 1 -- 200 | Number of terminal output lines captured and displayed per session. Higher values give more context but increase memory usage with many sessions. |
| **Show Output for Idle / Unknown Sessions** | Off | -- | When off, idle and unknown session cards are compact (no output area). Turn on if you want to see the last output from inactive sessions. |

## Classifier

| Setting | Default | Range | Description |
|---------|---------|-------|-------------|
| **Working → Idle Cool-off** | 5 minutes | 0 -- 60 min | How long a session's output must remain unchanged before it transitions from Working to Idle. This prevents flicker when Claude pauses briefly between tool calls. Lower values make the status more responsive; higher values reduce false Idle transitions. |

## Project Overrides

Some settings can be overridden on a per-project basis. This is useful when different repositories need different bootstrap commands or worktree behavior.

Settings marked as project-compatible:
- **Default Command** -- e.g., use `claude --plugin ../morpheus` for a specific repo
- **Use Git Worktrees** -- disable for repos where worktrees cause issues

To manage project overrides, open the settings panel and select the **Projects** category. Add an override by specifying a directory path and the settings you want to change for that project.

When creating a new session, Muxara resolves the effective value by checking project overrides first (matched by the selected working directory), then falling back to the global default.
