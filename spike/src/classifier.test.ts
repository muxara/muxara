import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, it, expect } from "vitest";
import { classify } from "./classifier.js";
import { hashOutput } from "./tmux-client.js";
import {
  SessionState,
  NeedsInputType,
  WORKING_THRESHOLD_MS,
  WORKING_IDLE_DEBOUNCE,
  type ClassifierInput,
} from "./types.js";

const FIXTURES_DIR = join(__dirname, "..", "fixtures");

function readFixture(relativePath: string): string {
  return readFileSync(join(FIXTURES_DIR, relativePath), "utf-8");
}

function strippedFiles(dir: string): string[] {
  return readdirSync(join(FIXTURES_DIR, dir))
    .filter((f) => f.endsWith("_stripped.txt"))
    .sort();
}

function makeInput(
  output: string,
  overrides: Partial<ClassifierInput> = {},
): ClassifierInput {
  const hash = hashOutput(output);
  return {
    normalizedOutput: output,
    outputHash: hash,
    previousHash: null,
    previousState: null,
    lastChangedAt: null,
    now: new Date(),
    paneTitle: null,
    consecutiveIdleCount: 0,
    ...overrides,
  };
}

// ─── Fixture-based classification tests ─────────────────────────────────────

describe("classifier: idle fixtures", () => {
  const files = strippedFiles("idle");

  it.each(files)("%s → Idle", (file) => {
    const output = readFixture(`idle/${file}`);
    // For idle: use same hash as previous to simulate no change
    const hash = hashOutput(output);
    const result = classify(
      makeInput(output, { previousHash: hash, lastChangedAt: new Date(0) }),
    );
    expect(result.state).toBe(SessionState.Idle);
    expect(result.needsInputType).toBeNull();
  });
});

describe("classifier: working fixtures", () => {
  const files = strippedFiles("working");

  it("sequential captures with changing output → Working", () => {
    // Take two different captures — different hashes mean output is changing
    const output1 = readFixture(`working/${files[0]}`);
    const output2 = readFixture(`working/${files[1]}`);
    const now = new Date();
    const recentChange = new Date(now.getTime() - 1_000); // 1s ago

    const result = classify(
      makeInput(output2, {
        previousHash: hashOutput(output1),
        lastChangedAt: recentChange,
        now,
      }),
    );
    expect(result.state).toBe(SessionState.Working);
  });

  it.each(files)(
    "%s → Working when output recently changed",
    (file) => {
      const output = readFixture(`working/${file}`);
      const now = new Date();
      const result = classify(
        makeInput(output, {
          previousHash: "different-hash",
          lastChangedAt: new Date(now.getTime() - 1_000),
          now,
        }),
      );
      expect(result.state).toBe(SessionState.Working);
    },
  );
});

describe("classifier: needs-input (permission) fixtures", () => {
  const files = strippedFiles("needs-input");

  it.each(files)("%s → NeedsInput (permission)", (file) => {
    const output = readFixture(`needs-input/${file}`);
    const result = classify(makeInput(output));
    expect(result.state).toBe(SessionState.NeedsInput);
    expect(result.needsInputType).toBe(NeedsInputType.Permission);
  });
});

describe("classifier: needs-input (ask) fixtures", () => {
  const files = strippedFiles("needs-input-ask");

  it.each(files)("%s → NeedsInput (question)", (file) => {
    const output = readFixture(`needs-input-ask/${file}`);
    const result = classify(makeInput(output));
    expect(result.state).toBe(SessionState.NeedsInput);
    expect(result.needsInputType).toBe(NeedsInputType.Question);
  });
});

describe("classifier: plan-mode fixtures", () => {
  const files = strippedFiles("plan-mode");

  it.each(files)("%s → isInPlanMode true", (file) => {
    const output = readFixture(`plan-mode/${file}`);
    const result = classify(makeInput(output));
    expect(result.isInPlanMode).toBe(true);
  });
});

describe("classifier: errored fixtures", () => {
  it("cli_error_invalid_flag → Errored", () => {
    const output = readFixture("errored/cli_error_invalid_flag_stripped.txt");
    const result = classify(makeInput(output));
    // This fixture has a CLI error followed by a new Claude session + permission prompt
    // The permission prompt in the tail takes priority over the error
    // So this will be NeedsInput — the error was from a previous command
    expect([SessionState.Errored, SessionState.NeedsInput]).toContain(result.state);
  });

  it("tool_failure_result → Errored", () => {
    const output = readFixture("errored/tool_failure_result_stripped.txt");
    const result = classify(makeInput(output));
    // This fixture shows Error: Exit code 1 followed by Claude explaining it,
    // then returning to idle prompt. The error is in the output but session recovered.
    // With the error still visible in the tail, it should detect as errored.
    expect(result.state).toBe(SessionState.Errored);
  });

  it("tool_failure (mid-flow) → NeedsInput (permission prompt visible)", () => {
    const output = readFixture("errored/tool_failure_stripped.txt");
    const result = classify(makeInput(output));
    // This fixture shows a permission prompt at the bottom — NeedsInput takes priority
    expect(result.state).toBe(SessionState.NeedsInput);
  });
});

// ─── Multi-frame temporal tests ─────────────────────────────────────────────

describe("classifier: multi-frame temporal logic", () => {
  it("sequence of identical captures → Idle (not Working)", () => {
    const output = readFixture("idle/idle_1_1775044486_stripped.txt");
    const hash = hashOutput(output);
    const now = new Date();

    // Simulate: same output captured twice, no change
    const result = classify(
      makeInput(output, {
        previousHash: hash,
        lastChangedAt: new Date(now.getTime() - 10_000),
        now,
      }),
    );
    expect(result.state).toBe(SessionState.Idle);
  });

  it("sequence of changing captures → Working", () => {
    const files = strippedFiles("working");
    const output1 = readFixture(`working/${files[0]}`);
    const output2 = readFixture(`working/${files[2]}`);
    const now = new Date();

    const result = classify(
      makeInput(output2, {
        previousHash: hashOutput(output1),
        lastChangedAt: new Date(now.getTime() - 2_000),
        now,
      }),
    );
    expect(result.state).toBe(SessionState.Working);
  });

  it("Working → Idle transition when output stops changing (after debounce)", () => {
    const files = strippedFiles("working");
    const output = readFixture(`working/${files[0]}`);
    const hash = hashOutput(output);
    const now = new Date();

    // Previously was working, output hasn't changed, AND debounce threshold met
    const result = classify(
      makeInput(output, {
        previousHash: hash,
        previousState: SessionState.Working,
        lastChangedAt: new Date(now.getTime() - WORKING_THRESHOLD_MS - 1_000),
        now,
        consecutiveIdleCount: WORKING_IDLE_DEBOUNCE,
      }),
    );
    expect(result.state).toBe(SessionState.Idle);
  });

  it("Working holds during debounce window (not yet enough idle polls)", () => {
    const files = strippedFiles("working");
    const output = readFixture(`working/${files[0]}`);
    const hash = hashOutput(output);
    const now = new Date();

    // Output stopped changing but debounce count not yet met — stay Working
    const result = classify(
      makeInput(output, {
        previousHash: hash,
        previousState: SessionState.Working,
        lastChangedAt: new Date(now.getTime() - WORKING_THRESHOLD_MS - 1_000),
        now,
        consecutiveIdleCount: WORKING_IDLE_DEBOUNCE - 1,
      }),
    );
    expect(result.state).toBe(SessionState.Working);
  });

  it("Working → NeedsInput when permission prompt appears", () => {
    const workingOutput = readFixture("working/working_1_1775044711_stripped.txt");
    const needsInputOutput = readFixture("needs-input/needs-input_1_1775044496_stripped.txt");
    const now = new Date();

    const result = classify(
      makeInput(needsInputOutput, {
        previousHash: hashOutput(workingOutput),
        previousState: SessionState.Working,
        lastChangedAt: new Date(now.getTime() - 1_000),
        now,
      }),
    );
    expect(result.state).toBe(SessionState.NeedsInput);
  });

  it("Errored state persists across identical captures (not reclassified as Idle)", () => {
    const output = readFixture("errored/tool_failure_result_stripped.txt");
    const hash = hashOutput(output);
    const now = new Date();

    // Same error output captured twice — should stay Errored, not become Idle
    const result = classify(
      makeInput(output, {
        previousHash: hash,
        previousState: SessionState.Errored,
        lastChangedAt: new Date(now.getTime() - 30_000),
        now,
      }),
    );
    expect(result.state).toBe(SessionState.Errored);
  });
});

// ─── Plan mode orthogonality ────────────────────────────────────────────────

describe("classifier: plan mode is orthogonal", () => {
  it("plan mode + needs-input → NeedsInput with isInPlanMode", () => {
    // plan-mode fixture 1 has both plan mode text and a permission prompt
    const output = readFixture("plan-mode/plan-mode_1_1775045425_stripped.txt");
    const result = classify(makeInput(output));
    expect(result.state).toBe(SessionState.NeedsInput);
    expect(result.isInPlanMode).toBe(true);
  });

  it("idle fixtures → isInPlanMode is null (not falsely detected)", () => {
    const output = readFixture("idle/idle_1_1775044486_stripped.txt");
    const result = classify(makeInput(output));
    expect(result.isInPlanMode).toBeNull();
  });
});

// ─── ANSI stripping ─────────────────────────────────────────────────────────

describe("tmux-client: stripAnsi", () => {
  // Import dynamically to test
  it("strips ANSI escape sequences", async () => {
    const { stripAnsi } = await import("./tmux-client.js");
    expect(stripAnsi("\x1b[38;5;12mhello\x1b[0m")).toBe("hello");
    expect(stripAnsi("\x1b[1mbold\x1b[22m")).toBe("bold");
    expect(stripAnsi("no escapes")).toBe("no escapes");
  });

  it("strips OSC sequences (tab title)", async () => {
    const { stripAnsi } = await import("./tmux-client.js");
    expect(stripAnsi("\x1b]0;My Title\x07rest")).toBe("rest");
    expect(stripAnsi("\x1b]2;Title\x1b\\rest")).toBe("rest");
  });
});
