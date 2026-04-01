import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { CAPTURE_SCROLLBACK_LINES } from "./types.js";

export interface TmuxSession {
  name: string;
  windows: number;
  created: Date;
  attached: boolean;
}

export interface TmuxPane {
  sessionName: string;
  windowIndex: number;
  paneIndex: number;
  target: string;
  width: number;
  height: number;
  currentPath: string;
}

export interface CapturedOutput {
  raw: string;
  normalized: string;
  hash: string;
  paneTitle: string | null;
}

function exec(cmd: string, args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    execFile(cmd, args, { timeout: 5_000 }, (err, stdout, stderr) => {
      if (err) {
        reject(new Error(`${cmd} ${args.join(" ")} failed: ${stderr || err.message}`));
        return;
      }
      resolve(stdout);
    });
  });
}

const ANSI_RE = /\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?(?:\x07|\x1b\\)/g;

export function stripAnsi(input: string): string {
  return input.replace(ANSI_RE, "");
}

export function hashOutput(normalized: string): string {
  return createHash("sha256").update(normalized).digest("hex").slice(0, 16);
}

export async function listSessions(): Promise<TmuxSession[]> {
  let stdout: string;
  try {
    stdout = await exec("tmux", [
      "list-sessions",
      "-F",
      "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
    ]);
  } catch {
    return [];
  }

  return stdout
    .trim()
    .split("\n")
    .filter((l) => l.length > 0)
    .map((line) => {
      const [name, windows, created, attached] = line.split("\t");
      return {
        name,
        windows: parseInt(windows, 10),
        created: new Date(parseInt(created, 10) * 1000),
        attached: attached === "1",
      };
    });
}

export async function listPanes(sessionName?: string): Promise<TmuxPane[]> {
  const args = [
    "list-panes",
    "-F",
    "#{session_name}\t#{window_index}\t#{pane_index}\t#{pane_width}\t#{pane_height}\t#{pane_current_path}",
  ];
  if (sessionName) {
    args.push("-t", sessionName);
  } else {
    args.push("-a");
  }

  let stdout: string;
  try {
    stdout = await exec("tmux", args);
  } catch {
    return [];
  }

  return stdout
    .trim()
    .split("\n")
    .filter((l) => l.length > 0)
    .map((line) => {
      const [session, winIdx, paneIdx, width, height, path] = line.split("\t");
      return {
        sessionName: session,
        windowIndex: parseInt(winIdx, 10),
        paneIndex: parseInt(paneIdx, 10),
        target: `${session}:${winIdx}.${paneIdx}`,
        width: parseInt(width, 10),
        height: parseInt(height, 10),
        currentPath: path,
      };
    });
}

export async function capturePaneOutput(target: string): Promise<CapturedOutput> {
  const [rawOutput, paneTitle] = await Promise.all([
    exec("tmux", [
      "capture-pane",
      "-p",
      "-S",
      `-${CAPTURE_SCROLLBACK_LINES}`,
      "-t",
      target,
    ]),
    exec("tmux", ["display-message", "-p", "-t", target, "#{pane_title}"]).then(
      (s) => s.trim() || null,
      () => null,
    ),
  ]);

  const normalized = stripAnsi(rawOutput);
  return {
    raw: rawOutput,
    normalized,
    hash: hashOutput(normalized),
    paneTitle,
  };
}
