import { listSessions, listPanes, capturePaneOutput, hashOutput } from "./tmux-client.js";
import { classify } from "./classifier.js";
import {
  SessionState,
  type ClassifierInput,
  type ClassifierResult,
  type NeedsInputType,
} from "./types.js";

// ─── Configuration ──────────────────────────────────────────────────────────

const POLL_INTERVAL_MS = parseInt(process.env.POLL_INTERVAL ?? "2000", 10);
const VERBOSE = process.argv.includes("--verbose") || process.argv.includes("-v");
const TAIL_LINES_VERBOSE = 10;

// ─── Session registry ───────────────────────────────────────────────────────

interface TrackedSession {
  name: string;
  target: string;
  workingDirectory: string;
  state: SessionState;
  needsInputType: NeedsInputType | null;
  isInPlanMode: boolean | null;
  previousState: SessionState | null;
  lastOutputHash: string | null;
  lastChangedAt: Date | null;
  lastSeenAt: Date;
  transitions: Array<{ from: SessionState | null; to: SessionState; at: Date }>;
  classificationCount: number;
  stateHistogram: Record<SessionState, number>;
  flickerCount: number;
  consecutiveIdleCount: number;
  lastSignals: string[];
}

const registry = new Map<string, TrackedSession>();

// ─── Metrics ────────────────────────────────────────────────────────────────

interface Metrics {
  totalClassifications: number;
  stateHistogram: Record<SessionState, number>;
  totalFlickers: number;
  startTime: Date;
}

const metrics: Metrics = {
  totalClassifications: 0,
  stateHistogram: {
    [SessionState.NeedsInput]: 0,
    [SessionState.Errored]: 0,
    [SessionState.Working]: 0,
    [SessionState.Idle]: 0,
    [SessionState.Unknown]: 0,
  },
  totalFlickers: 0,
  startTime: new Date(),
};

// ─── Display helpers ────────────────────────────────────────────────────────

const STATE_COLORS: Record<SessionState, string> = {
  [SessionState.NeedsInput]: "\x1b[31m",   // red
  [SessionState.Errored]: "\x1b[33m",      // yellow
  [SessionState.Working]: "\x1b[32m",      // green
  [SessionState.Idle]: "\x1b[90m",         // gray
  [SessionState.Unknown]: "\x1b[35m",      // magenta
};
const RESET = "\x1b[0m";
const BOLD = "\x1b[1m";
const DIM = "\x1b[2m";

function colorState(state: SessionState): string {
  return `${STATE_COLORS[state]}${state}${RESET}`;
}

function formatAge(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  return `${m}m${s % 60}s`;
}

function clearScreen() {
  process.stdout.write("\x1b[2J\x1b[H");
}

// ─── Signal description (for verbose mode) ──────────────────────────────────

function describeSignals(
  result: ClassifierResult,
  outputChanged: boolean,
  timeSinceChange: number | null,
): string[] {
  const signals: string[] = [];

  if (result.state === SessionState.NeedsInput) {
    signals.push(
      result.needsInputType === "question"
        ? "ask_user_question_prompt"
        : "permission_prompt",
    );
  }
  if (result.state === SessionState.Errored) signals.push("error_marker");
  if (outputChanged) signals.push("output_delta");
  if (!outputChanged) signals.push("no_output_delta");
  if (timeSinceChange !== null) {
    signals.push(`time_since_change=${formatAge(timeSinceChange)}`);
  }
  if (result.isInPlanMode === true) signals.push("plan_mode");
  if (result.state === SessionState.Working) signals.push("within_working_threshold");

  return signals;
}

// ─── Core polling loop ──────────────────────────────────────────────────────

async function poll() {
  const now = new Date();

  const sessions = await listSessions();
  if (sessions.length === 0) {
    clearScreen();
    console.log(`${DIM}No tmux sessions found. Start some Claude Code sessions in tmux.${RESET}`);
    console.log(`${DIM}Polling every ${POLL_INTERVAL_MS / 1000}s...${RESET}`);
    return;
  }

  // Get all panes across sessions
  const allPanes = await listPanes();

  // Capture output for each pane
  for (const pane of allPanes) {
    const key = pane.target;

    let captured;
    try {
      captured = await capturePaneOutput(pane.target);
    } catch {
      continue;
    }

    const tracked = registry.get(key);
    const previousHash = tracked?.lastOutputHash ?? null;
    const outputChanged = previousHash !== null && previousHash !== captured.hash;
    const lastChangedAt =
      outputChanged ? now : (tracked?.lastChangedAt ?? null);

    const input: ClassifierInput = {
      normalizedOutput: captured.normalized,
      outputHash: captured.hash,
      previousHash,
      previousState: tracked?.state ?? null,
      lastChangedAt,
      now,
      paneTitle: captured.paneTitle,
      consecutiveIdleCount: tracked?.consecutiveIdleCount ?? 0,
    };

    const result = classify(input);
    const timeSinceChange = lastChangedAt
      ? now.getTime() - lastChangedAt.getTime()
      : null;

    const signals = describeSignals(result, outputChanged, timeSinceChange);

    // Detect flicker: state changed back to what it was 2 transitions ago
    let flickered = false;
    if (tracked && result.state !== tracked.state) {
      const history = tracked.transitions;
      if (history.length >= 2) {
        const twoBack = history[history.length - 2];
        if (twoBack && twoBack.to === result.state) {
          flickered = true;
        }
      }
    }

    // Update or create registry entry
    if (tracked) {
      if (result.state !== tracked.state) {
        tracked.transitions.push({
          from: tracked.state,
          to: result.state,
          at: now,
        });
        tracked.previousState = tracked.state;
      }
      tracked.state = result.state;
      tracked.needsInputType = result.needsInputType;
      tracked.isInPlanMode = result.isInPlanMode;
      tracked.lastOutputHash = captured.hash;
      tracked.lastChangedAt = lastChangedAt;
      tracked.lastSeenAt = now;
      tracked.classificationCount++;
      tracked.stateHistogram[result.state]++;
      tracked.lastSignals = signals;
      if (flickered) tracked.flickerCount++;
      // Track consecutive idle readings for debounce
      if (result.state === SessionState.Working && tracked.previousState === SessionState.Working) {
        // Still working (possibly held by debounce) — increment if output stopped changing
        tracked.consecutiveIdleCount = outputChanged ? 0 : tracked.consecutiveIdleCount + 1;
      } else if (result.state !== SessionState.Working) {
        tracked.consecutiveIdleCount = 0;
      } else {
        tracked.consecutiveIdleCount = 0;
      }
    } else {
      const entry: TrackedSession = {
        name: pane.sessionName,
        target: pane.target,
        workingDirectory: pane.currentPath,
        state: result.state,
        needsInputType: result.needsInputType,
        isInPlanMode: result.isInPlanMode,
        previousState: null,
        lastOutputHash: captured.hash,
        lastChangedAt: lastChangedAt,
        lastSeenAt: now,
        transitions: [{ from: null, to: result.state, at: now }],
        classificationCount: 1,
        stateHistogram: {
          [SessionState.NeedsInput]: 0,
          [SessionState.Errored]: 0,
          [SessionState.Working]: 0,
          [SessionState.Idle]: 0,
          [SessionState.Unknown]: 0,
          [result.state]: 1,
        },
        flickerCount: 0,
        consecutiveIdleCount: 0,
        lastSignals: signals,
      };
      registry.set(key, entry);
    }

    // Update global metrics
    metrics.totalClassifications++;
    metrics.stateHistogram[result.state]++;
    if (flickered) metrics.totalFlickers++;
  }

  // Prune sessions no longer in tmux
  const activeTargets = new Set(allPanes.map((p) => p.target));
  for (const key of registry.keys()) {
    if (!activeTargets.has(key)) registry.delete(key);
  }

  render(now);
}

// ─── Rendering ──────────────────────────────────────────────────────────────

function render(now: Date) {
  clearScreen();

  const elapsed = now.getTime() - metrics.startTime.getTime();
  const sessionsArr = [...registry.values()];

  // Header
  console.log(
    `${BOLD}Muxara Spike Monitor${RESET}  ` +
      `${DIM}poll=${POLL_INTERVAL_MS / 1000}s  elapsed=${formatAge(elapsed)}  ` +
      `classifications=${metrics.totalClassifications}${RESET}`,
  );
  console.log();

  // Session table
  const COL = { name: 20, state: 16, plan: 6, age: 10, hash: 10, flicker: 8 };

  console.log(
    `${"Session".padEnd(COL.name)}` +
      `${"State".padEnd(COL.state)}` +
      `${"Plan".padEnd(COL.plan)}` +
      `${"Changed".padEnd(COL.age)}` +
      `${"Hash".padEnd(COL.hash)}` +
      `${"Flicker".padEnd(COL.flicker)}`,
  );
  console.log("─".repeat(COL.name + COL.state + COL.plan + COL.age + COL.hash + COL.flicker));

  for (const s of sessionsArr) {
    const age = s.lastChangedAt ? formatAge(now.getTime() - s.lastChangedAt.getTime()) : "—";
    const planStr = s.isInPlanMode === true ? "yes" : s.isInPlanMode === null ? "—" : "no";
    const hashStr = (s.lastOutputHash ?? "—").slice(0, 8);

    console.log(
      `${s.name.padEnd(COL.name)}` +
        `${colorState(s.state).padEnd(COL.state + 9)}` + // +9 for ANSI codes
        `${planStr.padEnd(COL.plan)}` +
        `${age.padEnd(COL.age)}` +
        `${hashStr.padEnd(COL.hash)}` +
        `${String(s.flickerCount).padEnd(COL.flicker)}`,
    );
  }
  console.log();

  // Global metrics
  const unknownRate =
    metrics.totalClassifications > 0
      ? ((metrics.stateHistogram[SessionState.Unknown] / metrics.totalClassifications) * 100).toFixed(1)
      : "0.0";
  const flickerRate =
    elapsed > 0
      ? (metrics.totalFlickers / (elapsed / 60_000)).toFixed(2)
      : "0.00";

  console.log(
    `${DIM}Global:  ` +
      `unknown=${unknownRate}%  ` +
      `flicker=${flickerRate}/min  ` +
      `states=[` +
      Object.entries(metrics.stateHistogram)
        .map(([k, v]) => `${k}:${v}`)
        .join(" ") +
      `]${RESET}`,
  );

  // Verbose output
  if (VERBOSE) {
    console.log();
    for (const s of sessionsArr) {
      console.log(`${BOLD}── ${s.target} ──${RESET}`);
      console.log(`  Signals: ${s.lastSignals.join(", ")}`);
      console.log(
        `  State histogram: ${Object.entries(s.stateHistogram)
          .filter(([, v]) => v > 0)
          .map(([k, v]) => `${k}:${v}`)
          .join("  ")}`,
      );

      if (s.transitions.length > 0) {
        const recent = s.transitions.slice(-5);
        console.log(
          `  Transitions (last ${recent.length}): ` +
            recent
              .map(
                (t) =>
                  `${t.from ?? "init"}→${t.to} @${t.at.toLocaleTimeString()}`,
              )
              .join("  "),
        );
      }

      // Show pane title if captured
      if (VERBOSE) {
        // Inline last N lines of output for debugging
        const lines = registry.get(s.target)
          ? ((): string[] => {
              // We don't store the full output in the registry to save memory,
              // but in verbose mode we re-capture inline context is already in the
              // signals. Show what we have.
              return [];
            })()
          : [];
        if (lines.length > 0) {
          console.log(`  Last ${TAIL_LINES_VERBOSE} lines:`);
          for (const line of lines) {
            console.log(`    ${DIM}${line}${RESET}`);
          }
        }
      }
      console.log();
    }
  }
}

// ─── Main ───────────────────────────────────────────────────────────────────

async function main() {
  console.log(`${BOLD}Muxara Spike Monitor${RESET}`);
  console.log(`Polling every ${POLL_INTERVAL_MS / 1000}s${VERBOSE ? " (verbose)" : ""}`);
  console.log(`Press Ctrl+C to stop\n`);

  // Handle graceful shutdown
  process.on("SIGINT", () => {
    printSummary();
    process.exit(0);
  });

  // Initial poll immediately
  await poll();

  // Then poll on interval
  setInterval(async () => {
    try {
      await poll();
    } catch (err) {
      console.error("Poll error:", err);
    }
  }, POLL_INTERVAL_MS);
}

function printSummary() {
  const elapsed = Date.now() - metrics.startTime.getTime();
  console.log("\n");
  console.log(`${BOLD}═══ Session Summary ═══${RESET}`);
  console.log(`Duration: ${formatAge(elapsed)}`);
  console.log(`Total classifications: ${metrics.totalClassifications}`);
  console.log();

  const unknownRate =
    metrics.totalClassifications > 0
      ? ((metrics.stateHistogram[SessionState.Unknown] / metrics.totalClassifications) * 100).toFixed(1)
      : "0.0";
  const flickerRate =
    elapsed > 0
      ? (metrics.totalFlickers / (elapsed / 60_000)).toFixed(2)
      : "0.00";

  console.log(`${BOLD}Global Metrics${RESET}`);
  console.log(`  Unknown rate: ${unknownRate}%`);
  console.log(`  Flicker rate: ${flickerRate}/min`);
  console.log(`  State distribution:`);
  for (const [state, count] of Object.entries(metrics.stateHistogram)) {
    if (count > 0) {
      const pct = ((count / metrics.totalClassifications) * 100).toFixed(1);
      console.log(`    ${state}: ${count} (${pct}%)`);
    }
  }
  console.log();

  console.log(`${BOLD}Per-Session${RESET}`);
  for (const s of registry.values()) {
    console.log(`  ${s.name} (${s.target})`);
    console.log(`    Classifications: ${s.classificationCount}`);
    console.log(`    Flickers: ${s.flickerCount}`);
    console.log(`    Final state: ${s.state}`);
    console.log(
      `    State histogram: ${Object.entries(s.stateHistogram)
        .filter(([, v]) => v > 0)
        .map(([k, v]) => `${k}:${v}`)
        .join("  ")}`,
    );
    console.log(
      `    Transitions (${s.transitions.length}): ${s.transitions
        .map((t) => `${t.from ?? "init"}→${t.to}`)
        .join(" → ")}`,
    );
    console.log();
  }

  // Viability assessment
  console.log(`${BOLD}═══ Viability Assessment ═══${RESET}`);
  const unknownPct = parseFloat(unknownRate);
  const flickerPerMin = parseFloat(flickerRate);

  const checks = [
    {
      label: "Unknown rate < 20%",
      pass: unknownPct < 20,
      value: `${unknownRate}%`,
    },
    {
      label: "Flicker rate < 2/min",
      pass: flickerPerMin < 2,
      value: `${flickerRate}/min`,
    },
  ];

  for (const c of checks) {
    const icon = c.pass ? "\x1b[32m✓\x1b[0m" : "\x1b[31m✗\x1b[0m";
    console.log(`  ${icon} ${c.label}: ${c.value}`);
  }
  console.log();
  console.log(`${DIM}Note: NeedsInput precision and transition latency require manual observation.${RESET}`);
  console.log(`${DIM}Review the transition log above to assess classification accuracy.${RESET}`);
}

main().catch((err) => {
  console.error("Fatal:", err);
  process.exit(1);
});
