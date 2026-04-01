import {
  SessionState,
  NeedsInputType,
  WORKING_THRESHOLD_MS,
  CLASSIFY_TAIL_LINES,
  type ClassifierInput,
  type ClassifierResult,
} from "./types.js";

// --- Hard signals: NeedsInput (permission) ---

const PERMISSION_PATTERNS = [
  /Do you want to proceed\?/i,
  /Do you want to create\b/i,
  /This command requires approval/i,
];

const PERMISSION_FOOTER = /Esc to cancel\s*·\s*Tab to amend/;

// --- Hard signals: NeedsInput (question / AskUserQuestion) ---

const ASK_QUESTION_MARKER = /☐\s+\S/;
const ASK_QUESTION_FOOTER = /Enter to select\s*·\s*↑\/↓ to navigate/;

// --- Hard signals: Errored ---

const ERROR_PATTERNS = [
  /^error:\s/im,
  /⎿\s+Error:\s/,
  /Error: Exit code \d+/,
];

// --- Plan mode signals (main pane, not status bar) ---

const PLAN_MODE_TRANSITION = /Entered plan mode/;
const PLAN_MODE_SPINNER = /^\s*✻\s/m;

// Status bar plan mode — soft signal, fallback only
const PLAN_MODE_STATUS_BAR = /⏸\s*plan mode on/;

function getTail(output: string, lines: number): string {
  const allLines = output.split("\n");
  return allLines.slice(-lines).join("\n");
}

function detectNeedsInput(tail: string): NeedsInputType | null {
  // Check AskUserQuestion first (more specific pattern)
  if (ASK_QUESTION_MARKER.test(tail) || ASK_QUESTION_FOOTER.test(tail)) {
    return NeedsInputType.Question;
  }

  // Check permission prompts
  const hasPermissionPrompt = PERMISSION_PATTERNS.some((p) => p.test(tail));
  const hasPermissionFooter = PERMISSION_FOOTER.test(tail);
  if (hasPermissionPrompt || hasPermissionFooter) {
    return NeedsInputType.Permission;
  }

  return null;
}

function detectErrored(tail: string, fullOutput: string): boolean {
  // Shell-level errors: check the full output since they appear early
  if (/^error:\s/im.test(fullOutput)) {
    // But only if the session hasn't recovered (no idle prompt after the error)
    const errorIdx = fullOutput.search(/^error:\s/im);
    const afterError = fullOutput.slice(errorIdx);
    // If there's a Claude TUI header after the error, Claude started a new session — not errored
    if (/▐▛███▜▌/.test(afterError.slice(20))) {
      return false;
    }
    // If the error is the last notable content, it's errored
    return true;
  }

  // Tool-level errors in the tail
  if (/⎿\s+Error:/.test(tail) || /Error: Exit code \d+/.test(tail)) {
    return true;
  }

  return false;
}

function detectPlanMode(tail: string, fullOutput: string): boolean | null {
  // Hard signals from main pane output (preferred)
  if (PLAN_MODE_SPINNER.test(tail)) return true;
  if (PLAN_MODE_TRANSITION.test(fullOutput)) return true;

  // Soft signal from status bar (fallback, may be unreliable)
  if (PLAN_MODE_STATUS_BAR.test(tail)) return true;

  // Can't determine — return null rather than guessing false
  return null;
}

function detectWorking(
  input: ClassifierInput,
): boolean {
  if (input.previousHash === null) {
    // First capture — can't determine delta. Not working.
    return false;
  }

  const outputChanged = input.outputHash !== input.previousHash;
  if (!outputChanged) return false;

  // Output changed — check if it changed recently enough
  const elapsed = input.now.getTime() - (input.lastChangedAt?.getTime() ?? 0);
  return elapsed <= WORKING_THRESHOLD_MS;
}

export function classify(input: ClassifierInput): ClassifierResult {
  const tail = getTail(input.normalizedOutput, CLASSIFY_TAIL_LINES);
  const fullOutput = input.normalizedOutput;

  // Evaluate all signals independently
  const needsInputType = detectNeedsInput(tail);
  const errored = detectErrored(tail, fullOutput);
  const working = detectWorking(input);
  const isInPlanMode = detectPlanMode(tail, fullOutput);

  // Resolve by priority: NeedsInput > Errored > Working > Idle > Unknown
  let state: SessionState;

  if (needsInputType !== null) {
    state = SessionState.NeedsInput;
  } else if (errored) {
    state = SessionState.Errored;
  } else if (working) {
    state = SessionState.Working;
  } else if (isOutputRecognizable(fullOutput)) {
    state = SessionState.Idle;
  } else {
    state = SessionState.Unknown;
  }

  return {
    state,
    needsInputType: state === SessionState.NeedsInput ? needsInputType : null,
    isInPlanMode,
  };
}

function isOutputRecognizable(output: string): boolean {
  // If we see any Claude Code markers, the output is recognizable
  return /❯/.test(output) || /▐▛███▜▌/.test(output) || /⏺/.test(output);
}
