import type { Preferences, SettingDefinition } from "./types";

export const DEFAULT_PREFERENCES: Preferences = {
  cooloffMinutes: 5,
  pollIntervalSecs: 1.5,
  outputLines: 30,
  showIdleOutput: false,
  contextZoneMaxHeight: 192,
  gridColumns: 2,
  scrollPauseSecs: 5,
  bootstrapCommand: "claude",
  useWorktree: true,
  projectOverrides: {},
};

export const CATEGORIES = ["Sessions", "Polling", "Display", "Classifier", "Projects"] as const;

export const SETTINGS_SCHEMA: SettingDefinition[] = [
  // Sessions
  {
    key: "bootstrapCommand",
    label: "Default Command",
    description:
      "The command sent when creating a new session. Can be overridden per project.",
    category: "Sessions",
    type: "text",
    default: "claude",
    projectCompatible: true,
  },

  {
    key: "useWorktree",
    label: "Use Git Worktrees",
    description:
      "Automatically create an isolated git worktree for each new session. Only applies to git repositories.",
    category: "Sessions",
    type: "boolean",
    default: true,
    projectCompatible: true,
  },

  // Polling
  {
    key: "pollIntervalSecs",
    label: "Poll Interval",
    description: "How frequently the app checks for session updates.",
    category: "Polling",
    type: "number",
    default: 1.5,
    min: 0.5,
    max: 30,
    step: 0.5,
    unit: "seconds",
  },
  {
    key: "scrollPauseSecs",
    label: "Scroll Pause Duration",
    description:
      "How long polling pauses when you scroll inside a card's output area.",
    category: "Polling",
    type: "number",
    default: 5,
    min: 0,
    max: 60,
    step: 1,
    unit: "seconds",
  },

  // Display
  {
    key: "gridColumns",
    label: "Grid Columns",
    description: "Number of card columns in the dashboard grid.",
    category: "Display",
    type: "select",
    default: 2,
    options: [
      { value: 1, label: "1" },
      { value: 2, label: "2" },
      { value: 3, label: "3" },
      { value: 4, label: "4" },
      { value: 5, label: "5" },
      { value: 6, label: "6" },
    ],
  },
  {
    key: "contextZoneMaxHeight",
    label: "Context Zone Height",
    description: "Maximum height of the scrollable output area within a card.",
    category: "Display",
    type: "number",
    default: 192,
    min: 48,
    max: 800,
    step: 8,
    unit: "px",
  },
  {
    key: "outputLines",
    label: "Output Lines Per Card",
    description: "Number of terminal output lines stored and displayed per session card.",
    category: "Display",
    type: "number",
    default: 20,
    min: 1,
    max: 200,
    step: 1,
    unit: "lines",
  },
  {
    key: "showIdleOutput",
    label: "Show Output for Idle / Unknown Sessions",
    description:
      "Whether to display the output area on idle and unknown session cards.",
    category: "Display",
    type: "boolean",
    default: false,
  },

  // Classifier
  {
    key: "cooloffMinutes",
    label: "Working → Idle Cool-off",
    description:
      "How long a session must have no output change before transitioning from Working to Idle.",
    category: "Classifier",
    type: "number",
    default: 5,
    min: 0,
    max: 60,
    step: 0.5,
    unit: "minutes",
  },
];
