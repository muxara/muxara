import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Ansi from "ansi-to-react";
import type { Session, SessionState } from "../types";
import { StatusBadge } from "./StatusBadge";
import { usePreferences } from "../hooks/usePreferences";

const stateConfig: Record<
  SessionState,
  { label: string; border: string; bg: string }
> = {
  "needs-input": {
    label: "Needs Input",
    border: "border-l-amber-400",
    bg: "bg-amber-950/20",
  },
  working: {
    label: "Working",
    border: "border-l-blue-400",
    bg: "bg-blue-950/15",
  },
  idle: {
    label: "Idle",
    border: "border-l-gray-600",
    bg: "bg-gray-800/30",
  },
  errored: {
    label: "Errored",
    border: "border-l-red-400",
    bg: "bg-red-950/15",
  },
  unknown: {
    label: "Unknown",
    border: "border-l-gray-500 border-dashed",
    bg: "bg-gray-800/20",
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

export function SessionCard({ session, onScrollActivity, focused, selected, onFocus }: { session: Session; onScrollActivity: () => void; focused: boolean; selected: boolean; onFocus: (id: string) => void }) {
  const { prefs } = usePreferences();
  const config = stateConfig[session.state];
  const [clicking, setClicking] = useState(false);
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);
  const [showConfirmKill, setShowConfirmKill] = useState(false);
  const [killError, setKillError] = useState<string | null>(null);
  const [renaming, setRenaming] = useState(false);
  const [renameValue, setRenameValue] = useState(session.name);
  const menuRef = useRef<HTMLDivElement>(null);
  const renameRef = useRef<HTMLInputElement>(null);

  // Close context menu on outside click
  useEffect(() => {
    if (!menuPos) return;
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuPos(null);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [menuPos]);

  // Focus rename input when entering rename mode
  useEffect(() => {
    if (renaming && renameRef.current) {
      renameRef.current.focus();
      renameRef.current.select();
    }
  }, [renaming]);

  async function handleClick() {
    if (renaming) return;
    setClicking(true);
    onFocus(session.id);
    try {
      await invoke("focus_session", { sessionId: session.id });
    } catch (err) {
      console.error("Failed to focus session:", err);
    } finally {
      setTimeout(() => setClicking(false), 300);
    }
  }

  function handleContextMenu(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    setMenuPos({ x: e.clientX, y: e.clientY });
  }

  async function handleKill() {
    try {
      await invoke("kill_session", { sessionId: session.id });
      setShowConfirmKill(false);
      setKillError(null);
    } catch (err) {
      setKillError(String(err));
    }
  }

  async function handleRenameSubmit() {
    const trimmed = renameValue.trim();
    if (trimmed && trimmed !== session.name) {
      try {
        await invoke("rename_session", { sessionId: session.id, newName: trimmed });
      } catch (err) {
        console.error("Failed to rename session:", err);
      }
    }
    setRenaming(false);
  }

  return (
    <>
      <div
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        className={`flex flex-col rounded-lg cursor-pointer transition-all duration-150 ${
          clicking ? "scale-[0.97] brightness-125" : "hover:brightness-110"
        } ${config.border} ${config.bg} ${
          focused ? "border-l-[6px] !border-l-emerald-400 ring-1 ring-emerald-400/40 shadow-[0_4px_16px_rgba(0,0,0,0.4),0_0_20px_rgba(52,211,153,0.2)] -translate-y-1" : selected ? "border-l-2 ring-1 ring-gray-500/60 shadow-lg" : "border-l-2 shadow-md"
        } ${
          clicking ? "focused-glow" : ""
        }`}
      >
        {/* ── Orientation zone ── */}
        <div className="px-3 pt-3 pb-2">
          <div className="flex items-center gap-2 mb-1">
            <StatusBadge state={session.state} />
            {renaming ? (
              <input
                ref={renameRef}
                className="font-semibold text-sm text-gray-100 bg-gray-700 border border-gray-600 rounded px-1 py-0.5 outline-none focus:border-blue-400 w-full"
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleRenameSubmit();
                  if (e.key === "Escape") setRenaming(false);
                }}
                onBlur={handleRenameSubmit}
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <h3 className="font-semibold text-base text-gray-100 truncate">
                {session.name}
              </h3>
            )}
          </div>
        <p
          className="text-xs text-gray-400 truncate mb-1"
          title={session.workingDirectory}
        >
          {session.workingDirectory.startsWith("/")
            ? `~/${session.workingDirectory.split("/").slice(-2).join("/")}`
            : dirBasename(session.workingDirectory)}
        </p>
        {session.gitBranch && (
          <p className="text-[11px] text-gray-400 truncate mb-1">
            <span className="text-gray-400">branch:</span> {session.gitBranch}
            {session.isWorktree && (
              <span className="ml-1.5 text-[10px] text-violet-400/70 font-medium">WT</span>
            )}
          </p>
        )}
        <p className="text-xs text-gray-400">
          {stateLabel(session)} · {timeAgo(session.lastChangedAt)}
        </p>
      </div>

        {/* ── Context zone ── */}
        {session.lastOutputLines.length > 0 &&
          (prefs.showIdleOutput || (session.state !== "idle" && session.state !== "unknown")) && (
          <div className="relative border-t border-gray-700/50 mt-auto">
            <div className="px-3 py-2 overflow-y-auto" style={{ maxHeight: prefs.contextZoneMaxHeight }} onScroll={onScrollActivity}>
              {(session.lastOutputLinesAnsi ?? session.lastOutputLines).map((line, i) => (
                <p
                  key={i}
                  className="text-[11px] leading-5 font-mono text-gray-500 truncate"
                >
                  {line ? <Ansi>{line}</Ansi> : "\u00A0"}
                </p>
              ))}
            </div>
            <div className="absolute bottom-0 left-0 right-0 h-4 bg-gradient-to-t from-gray-950/80 to-transparent pointer-events-none rounded-b-lg" />
          </div>
        )}
      </div>

      {/* ── Context menu ── */}
      {menuPos && (
        <div
          ref={menuRef}
          className="fixed z-50 bg-gray-800 border border-gray-600 rounded-lg shadow-xl py-1 min-w-[140px]"
          style={{ left: menuPos.x, top: menuPos.y }}
        >
          <button
            className="w-full text-left px-3 py-1.5 text-sm text-gray-200 hover:bg-gray-700 transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              setMenuPos(null);
              setRenameValue(session.name);
              setRenaming(true);
            }}
          >
            Rename
          </button>
          <button
            className="w-full text-left px-3 py-1.5 text-sm text-red-400 hover:bg-gray-700 transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              setMenuPos(null);
              setKillError(null);
              setShowConfirmKill(true);
            }}
          >
            Kill Session
          </button>
        </div>
      )}

      {/* ── Kill confirmation dialog ── */}
      {showConfirmKill && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={() => setShowConfirmKill(false)}>
          <div className="bg-gray-800 border border-gray-600 rounded-lg p-4 shadow-xl max-w-xs w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <p className="text-sm text-gray-200 mb-3">
              Kill session <strong>{session.name}</strong>? This will terminate the session and any running processes.
              {session.isWorktree && (
                <span className="block text-xs text-gray-400 mt-1">
                  The associated git worktree will also be removed.
                </span>
              )}
            </p>
            <div className="flex justify-end gap-2">
              <button
                className="px-3 py-1.5 text-sm text-gray-300 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
                onClick={() => setShowConfirmKill(false)}
              >
                Cancel
              </button>
              <button
                className="px-3 py-1.5 text-sm text-white bg-red-600 hover:bg-red-500 rounded transition-colors"
                onClick={handleKill}
              >
                Kill
              </button>
            </div>
            {killError && (
              <p className="text-xs text-red-400 mt-2">{killError}</p>
            )}
          </div>
        </div>
      )}
    </>
  );
}
