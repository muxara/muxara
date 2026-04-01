export enum SessionState {
  NeedsInput = "needs-input",
  Errored = "errored",
  Working = "working",
  Idle = "idle",
  Unknown = "unknown",
}

export enum NeedsInputType {
  Permission = "permission",
  Question = "question",
}

export interface Session {
  id: string;
  name: string;
  state: SessionState;
  needsInputType: NeedsInputType | null;
  isInPlanMode: boolean | null;
  lastOutputLines: string[];
  lastOutputHash: string;
  lastChangedAt: Date;
  lastSeenAt: Date;
  previousState: SessionState | null;
  workingDirectory: string;
  paneTitle: string | null;
  createdAt: Date;
}

export interface ClassifierInput {
  normalizedOutput: string;
  outputHash: string;
  previousHash: string | null;
  previousState: SessionState | null;
  lastChangedAt: Date | null;
  now: Date;
  paneTitle: string | null;
}

export interface ClassifierResult {
  state: SessionState;
  needsInputType: NeedsInputType | null;
  isInPlanMode: boolean | null;
}

/** How recently output must have changed to be considered "working" */
export const WORKING_THRESHOLD_MS = 5_000;

/** Number of scrollback lines to capture from tmux */
export const CAPTURE_SCROLLBACK_LINES = 200;

/** Number of bottom lines to focus classification on */
export const CLASSIFY_TAIL_LINES = 50;
