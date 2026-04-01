import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Session, SessionState } from "../types";
import { StatusBadge } from "./StatusBadge";

const stateConfig: Record<
  SessionState,
  { label: string; border: string; bg: string }
> = {
  "needs-input": {
    label: "Needs Input",
    border: "border-l-amber-400",
    bg: "bg-amber-950/40",
  },
  working: {
    label: "Working",
    border: "border-l-blue-400",
    bg: "bg-blue-950/30",
  },
  idle: {
    label: "Idle",
    border: "border-l-gray-600",
    bg: "bg-gray-800/50",
  },
  errored: {
    label: "Errored",
    border: "border-l-red-400",
    bg: "bg-red-950/30",
  },
  unknown: {
    label: "Unknown",
    border: "border-l-gray-500 border-dashed",
    bg: "bg-gray-800/30",
  },
};

function dirBasename(path: string): string {
  const parts = path.split("/").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : path;
}

function timeAgo(iso: string): string {
  const diffMs = Date.now() - new Date(iso).getTime();
  if (diffMs < 0) return "just now";
  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

function stateLabel(session: Session): string {
  const config = stateConfig[session.state];
  if (session.state === "needs-input" && session.needsInputType) {
    const sub =
      session.needsInputType === "permission" ? "Permission" : "Question";
    return `${config.label} · ${sub}`;
  }
  return config.label;
}

export function SessionCard({ session, onScrollActivity }: { session: Session; onScrollActivity: () => void }) {
  const config = stateConfig[session.state];
  const [clicking, setClicking] = useState(false);

  async function handleClick() {
    setClicking(true);
    try {
      await invoke("focus_session", { sessionId: session.id });
    } catch (err) {
      console.error("Failed to focus session:", err);
    } finally {
      setTimeout(() => setClicking(false), 300);
    }
  }

  return (
    <div
      onClick={handleClick}
      className={`flex flex-col rounded-lg border-l-4 cursor-pointer transition-all duration-150 ${
        clicking ? "scale-[0.97] brightness-125" : "hover:brightness-110"
      } ${config.border} ${config.bg} shadow-md`}
    >
      {/* ── Orientation zone ── */}
      <div className="px-3 pt-3 pb-2">
        <div className="flex items-center gap-2 mb-1">
          <StatusBadge state={session.state} />
          <h3 className="font-semibold text-sm text-gray-100 truncate">
            {session.name}
          </h3>
        </div>
        <p
          className="text-xs text-gray-400 truncate mb-1"
          title={session.workingDirectory}
        >
          {session.workingDirectory.startsWith("/")
            ? `~/${session.workingDirectory.split("/").slice(-2).join("/")}`
            : dirBasename(session.workingDirectory)}
        </p>
        <p className="text-[11px] text-gray-500">
          {stateLabel(session)} · {timeAgo(session.lastChangedAt)}
        </p>
      </div>

      {/* ── Context zone ── */}
      {session.lastOutputLines.length > 0 &&
        session.state !== "idle" &&
        session.state !== "unknown" && (
        <div className="border-t border-gray-700/50 px-3 py-2 mt-auto max-h-48 overflow-y-auto" onScroll={onScrollActivity}>
          {session.lastOutputLines.map((line, i) => (
            <p
              key={i}
              className="text-[11px] leading-4 font-mono text-gray-400 truncate"
            >
              {line || "\u00A0"}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
