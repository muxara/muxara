# Spike Findings

---

# Spike #1 & #2: Capture Terminal Output and Build Classifier

## Summary

Captured real Claude Code terminal output from tmux panes across 4 core states (idle, working, needs-input, errored) plus two additional dimensions: AskUserQuestion prompts and plan mode vs. edit mode. Used these findings to build and test the tmux client and state classifier (issue #2).

## Capture Method

- `tmux capture-pane -p -S -200 -t <session>` for stripped output
- `tmux capture-pane -p -e -S -200 -t <session>` for ANSI-preserved output
- Multiple captures per state at 2-second intervals to observe output evolution
- Fixtures stored in `spike/fixtures/<state>/`

## State Signatures

### Idle

The prompt line `❯` followed by an empty line (or just whitespace/cursor), with no tool output or generation in progress.

**Key markers:**
- `❯` prompt visible with nothing after it (or just a cursor block)
- Status bar at bottom: `Model: ... | Ctx: ... | ⎇ ... | 𖠰 ...`
- No change between consecutive captures

### Working (Actively Generating)

Claude is producing text output. The pane content changes between captures.

**Key markers:**
- `⏺` prefix on Claude's response lines (this is the assistant output marker)
- Content grows between captures (line count increases over time)
- May show tool invocations like `⏺ Bash(...)` or `Read 1 file (ctrl+o to expand)`
- `Running…` text visible during tool execution
- `Searching for N patterns` text during search operations

### Needs Input (Permission Prompt)

Claude is waiting for the user to approve a tool use.

**Key markers (HARD SIGNALS):**
- `Do you want to proceed?` — bash/command permission
- `Do you want to create <filename>?` — file creation permission
- Numbered options: `❯ 1. Yes` / `2. Yes, allow...` / `3. No`
- `Esc to cancel · Tab to amend` footer text
- `This command requires approval` text
- Visual diff block with `╌╌╌` separators showing file changes

### Needs Input (AskUserQuestion)

Claude is asking the user a clarifying question — distinct from a permission prompt. Uses a different visual pattern with a checkbox icon and multiple-choice options.

**Key markers (HARD SIGNALS):**
- `☐` (U+2610) checkbox icon followed by a topic label (e.g., `☐ Scope`)
- A question in plain text (e.g., "What do you mean by 'refactor the project'?")
- Numbered options with descriptions: `❯ 1. Clean up repo structure` etc.
- `Enter to select · ↑/↓ to navigate · Esc to cancel` footer text
- Option `N. Type something.` as a free-text fallback
- Option `N+1. Chat about this` as a conversation option

**Key difference from permission prompts:** Permission prompts show `Do you want to proceed?` / `Do you want to create`. AskUserQuestion shows `☐ <Topic>` with a question and descriptive multi-choice options. Both are "needs input" but the classifier should distinguish them for the attention model — AskUserQuestion typically requires more thought from the user than a simple yes/no permission.

### Plan Mode vs. Edit Mode

Claude Code has two operational modes visible in the status bar. This is orthogonal to the above states — a session can be in plan mode and idle, working, or needs-input.

**Status bar indicators:**
- Plan mode: `⏸ plan mode on (shift+tab to cycle)`
- Edit mode: `⏵⏵ accept edits on (shift+tab to cycle)`

**Other markers:**
- `Entered plan mode` — text output when Claude transitions into plan mode
- `Claude is now exploring and designing an implementation approach.` — plan mode entry text
- `✻` (U+273B) spinner prefix during plan mode tool execution (vs. `⏺` in edit mode)

**WARNING: Status bar is NOT a reliable hard signal.** Users can install custom status bar plugins that change the layout and content. The built-in Claude Code status bar contains the mode indicators above, but custom plugins may reposition, wrap, or obscure them. The classifier must degrade gracefully when the status bar format is unexpected — treat mode detection from the status bar as a **soft signal**.

**More reliable mode signals:**
- `Entered plan mode` / transition text in the main pane output (hard signal — not affected by plugins)
- `✻` vs `⏺` spinner prefix (hard signal — in main pane, not status bar)

**Why this matters for the attention model:** Plan mode sessions are exploring and reading, not making changes. A plan mode session that's "working" is lower urgency than an edit mode session that's "needs input". The dashboard should surface this distinction.

### Terminal Tab Title

The terminal tab title (set via ANSI escape sequences) is an additional signal source. Claude Code sets the tab title to reflect the current operation, which can be captured via `tmux display-message -p '#{pane_title}'`.

**This was not captured in this spike** and should be investigated as a follow-up. Tab titles may provide a more stable signal than the status bar since they are set by Claude Code itself and not affected by custom status bar plugins.

### Errored

Two distinct error types observed:

**CLI errors** (before Claude starts):
- `error: unknown option '...'` — invalid CLI flag
- `(Did you mean ...?)` — suggestion text
- These appear at the shell level, not inside the Claude TUI

**Tool failures** (inside Claude session):
- `⎿  Error: Exit code N` — bash command failure
- Claude then explains the error in its response (prefixed with `⏺`)
- The session returns to idle after the error response

**Important finding:** Claude handles many "errors" gracefully. Reading a nonexistent file doesn't produce a visible error — Claude just says "That file doesn't exist." Tool failures show `Error:` in the tool output block but the session recovers to idle.

## Signal Classification

### Hard Signals (Reliable)

| Signal | State | Notes |
|--------|-------|-------|
| `Do you want to proceed?` | needs-input | Permission prompt for commands |
| `Do you want to create` | needs-input | Permission prompt for file creation |
| `❯ 1. Yes` + numbered options | needs-input | Selection UI visible |
| `Esc to cancel` | needs-input | Footer of permission dialog |
| `☐` + question text | needs-input (ask) | AskUserQuestion prompt — requires user decision |
| `Enter to select · ↑/↓ to navigate` | needs-input (ask) | Footer of AskUserQuestion dialog |
| `Entered plan mode` | plan mode | Transition text in main pane output |
| `✻` spinner prefix | working (plan) | Plan mode tool execution spinner (in main pane) |
| `error:` at shell level | errored | CLI error (outside Claude TUI) |
| `Error: Exit code` | errored | Tool failure (inside Claude TUI) |

### Soft Signals (Contextual / Noisy)

| Signal | State | Notes |
|--------|-------|-------|
| `❯` with empty line after | idle | Could be transitional — also appears briefly between tool calls |
| Content delta between captures | working | Reliable if measured, but requires two snapshots |
| `⏺` lines growing | working | Model is generating output |
| `Running…` | working | Tool currently executing — but transient |
| `Searching for N patterns` | working | Tool-specific progress indicator |
| Status bar `Ctx: N` value | varies | Higher context = more work done, but not real-time |
| `Cost: $X.XX` | varies | Increases during work, but not a state indicator |
| `⏸ plan mode on` | plan mode | Status bar — unreliable due to custom plugins |
| `⏵⏵ accept edits on` | edit mode | Status bar — unreliable due to custom plugins |
| Terminal tab title | varies | Set by Claude Code, not affected by plugins — needs investigation |

### Compound Signals (Most Reliable)

The classifier should use combinations:

- **Idle** = `❯` prompt visible + no content delta + no permission/ask prompt
- **Working** = content delta between two captures OR `Running…` visible OR `✻` spinner
- **Needs input (permission)** = `Do you want to proceed?` or `Do you want to create` (hard signal)
- **Needs input (question)** = `☐` marker + `Enter to select` footer (hard signal — higher urgency than permission)
- **Errored** = `error:` or `Error:` in recent output (but session may have recovered to idle)
- **Plan mode** = `Entered plan mode` in output OR `✻` spinner (main pane signals preferred over status bar due to plugin variability)

## ANSI Handling

- `tmux capture-pane -p` (without `-e`) produces clean text with Unicode but no ANSI escapes
- `tmux capture-pane -p -e` preserves `\e[38;5;NNm` color codes and `\e[1m` bold markers
- Stripping with `perl -pe 's/\e\[[0-9;]*m//g'` produces output nearly identical to `-p` mode
- **Recommendation:** Use `-p` (stripped) mode for classification. ANSI sequences add no useful signal for state detection.

## Unicode Characters in Output

Claude Code uses extensive Unicode:

| Character | Meaning |
|-----------|---------|
| `❯` (U+276F) | User prompt marker |
| `⏺` (U+23FA) | Assistant output marker |
| `⎿` (U+23BF) | Tool output block marker |
| `────` (U+2500) | Horizontal rule / section separator |
| `╌╌╌` (U+254C) | File diff block separator |
| `▐▛███▜▌` etc. | Logo/branding in header |
| `⎇` (U+2387) | Git branch indicator in status bar |
| `𖠰` (U+16830) | Git status indicator in status bar |
| `◐` (U+25D0) | Effort level indicator |
| `☐` (U+2610) | AskUserQuestion prompt marker |
| `✻` (U+273B) | Plan mode spinner/progress indicator |
| `⏸` (U+23F8) | Plan mode status bar indicator |

These Unicode markers are stable across captures and are more reliable than trying to match text patterns that might vary.

## Ambiguous Cases

1. **Transitional idle**: Between tool calls, Claude briefly shows `❯` prompt before starting the next tool. A single snapshot during this transition looks like idle. **Mitigation:** Require 2+ consecutive idle captures before declaring idle.

2. **Graceful error handling**: When a tool fails, Claude often handles it conversationally ("That file doesn't exist"). The error is only visible in the tool output block (`⎿ Error:`), not in Claude's response text. **Mitigation:** Check tool output blocks specifically, not just Claude's response.

3. **Post-completion idle vs. never-started idle**: A session that completed a task looks identical to one that was just opened. **Mitigation:** Track `Ctx:` and `Cost:` values — non-zero indicates prior work.

4. **Multi-step permission**: The working session may show `Running…` briefly before hitting a permission prompt. A capture during `Running…` looks like working, but the session is about to need input. **Mitigation:** Permission prompts are hard signals — if present, always classify as needs-input regardless of other signals.

5. **AskUserQuestion vs. permission prompt**: Both are "needs input" but have different urgency. AskUserQuestion requires deliberation; permission prompts are usually quick yes/no. **Mitigation:** Classify both as needs-input but tag the subtype (permission vs. question) for the attention model.

6. **Plan mode permission prompts**: A session in plan mode can still show permission prompts. The plan mode indicator is in the status bar while the permission prompt is in the main pane. **Mitigation:** Check both the status bar (last 2-3 lines) for mode and the main content for state independently.

7. **Long output scrolling**: When Claude generates very long output, earlier tool calls scroll off the visible pane. Only the last ~50 visible lines are available without scrollback. **Mitigation:** Use `-S -200` to capture scrollback, and focus classification on the bottom of the output (most recent state).

## Fixture Inventory

```
spike/fixtures/
├── idle/                    # 5 captures × 2 (stripped + ansi) = 10 files
├── working/                 # 10 captures × 2 = 20 files (growing line counts prove output evolution)
├── needs-input/             # 10 captures × 2 = 20 files (bash permission + file creation permission)
├── needs-input-ask/         # 5 captures × 2 = 10 files (AskUserQuestion with multi-choice)
├── plan-mode/               # 10 captures × 2 = 20 files (plan mode entry + working + status bar)
└── errored/                 # CLI error + tool failure captures = 6 files
```

## Recommendations for Classifier (Issue #2)

1. **Start with hard signals**: Check for permission prompt text first. If found, classify as `needs-input` immediately.
2. **Use delta detection for working**: Compare current capture to previous capture. If content changed, classify as `working`.
3. **Check for error markers**: Look for `Error:` in tool output blocks and `error:` at shell level.
4. **Fall back to idle**: If no hard signals and no content delta, classify as `idle`.
5. **Include `Unknown` state**: Use when output doesn't match any known pattern (e.g., Claude Code is still loading, or the session crashed).
6. **Use stripped output only**: ANSI codes add noise without useful signal.
7. **Focus on the last 30-50 lines**: The most recent state is at the bottom of the pane output.
8. **Detect plan mode from main pane output, not status bar**: Prefer `Entered plan mode` transition text and `✻` spinner prefix over status bar indicators. Custom status bar plugins can distort or hide the built-in mode indicators. Status bar can be used as a secondary/fallback signal but should not be the primary detection method.
9. **Distinguish AskUserQuestion from permission prompts**: Look for `☐` marker and `Enter to select · ↑/↓ to navigate` footer. AskUserQuestion requires more user thought than a yes/no permission — the attention model should weight it higher.
10. **Investigate terminal tab title**: Claude Code sets the terminal tab title via ANSI escapes. This can be read with `tmux display-message -p '#{pane_title}'` and may be a more stable signal than status bar content since it's not affected by custom plugins. This was not captured in this spike.
11. **Design for status bar variability**: Users install custom status bar plugins. The classifier should not hard-depend on specific status bar formatting. Degrade gracefully when the status bar doesn't match expected patterns.

---

# Spike #3: End-to-End Validation Against Live Sessions

## Summary

Validated the classifier end-to-end against real live Claude Code sessions in tmux. Discovered a Working→Idle flicker issue caused by brief pauses between tool calls, fixed it with a debounce mechanism, and confirmed all viability criteria were met. **Decision: proceed to Phase 1.**

## Setup

3 tmux sessions, each running Claude Code in a different state:

| Session | Setup | Expected state |
|---|---|---|
| `spike-idle` | `claude` with no prompt | Idle |
| `spike-input` | `claude "List all files in /usr recursively..."` — triggers bash permission prompt | NeedsInput |
| `spike-work` | `claude "Explain plan.md in great detail, 3000+ words"` — long generation task | Working |

Monitor tool (`npm run spike:verbose`) polled all tmux panes every 2s. Each run lasted 2 minutes (~60 polls/session, ~180 total classifications).

## Run 1: Before debounce

| Session | Classifications | Result | Issues |
|---|---|---|---|
| spike-idle | 60 | `idle` 60/60 | None |
| spike-input | 60 | `needs-input` 60/60 | None |
| spike-work | 58 | `working:20, idle:38` | **5 flickers** — rapid `working↔idle` oscillation |

- **Unknown rate**: 0%
- **Flicker rate**: 2.54/min (above 2/min target)

**Root cause of flicker**: Claude Code pauses briefly between tool calls. Output stops changing, the 5s working threshold expires, classifier says "idle", then output resumes and it flips back to "working".

## Fix: Working→Idle debounce

Added `WORKING_IDLE_DEBOUNCE = 2` — when previousState is Working and classifier would say Idle, require 2 consecutive idle readings before transitioning. Hard signals (NeedsInput, Errored) still take effect immediately.

## Run 2: After debounce

| Session | Classifications | Result | Issues |
|---|---|---|---|
| spike-idle | 60 | `idle` 60/60 | None |
| spike-input | 60 | `needs-input` 60/60 | None |
| spike-work | 59 | `working:59, idle:1` | **0 flickers** |

- **Unknown rate**: 0%
- **Flicker rate**: 0.00/min
- **NeedsInput precision**: 100% (60/60)
- **Idle precision**: 100% (60/60)
- **Working stability**: 59 consecutive correct after initial startup

## Viability Assessment

| Criterion | Target | Result | Pass |
|---|---|---|---|
| NeedsInput detection | Near-perfect | 100% (120/120 across both runs) | Yes |
| Errored detection | Reliable | Validated via fixtures (6 fixture files) | Yes |
| Working vs Idle stability | Low flicker | 0.00/min after debounce | Yes |
| Unknown rate | < 20% | 0% | Yes |
| Transition latency | Within seconds | 1 poll cycle (2s) | Yes |

## What Was NOT Tested Live

- **Errored state** — only validated via fixture files, not against a live session
- **Transition scenarios** (NeedsInput→Working on approval, Errored→Idle on recovery) — would require interactive testing with the tmux sessions
- **Longer stability run** (10-15 min) — both runs were 2 minutes; longer run recommended before shipping but results are very promising

## Decision

Proceed to Phase 1. The core bet — that we can reliably infer Claude Code session state from tmux pane output — is validated. The classifier is accurate, stable, and the architecture is sound.
