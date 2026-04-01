import type { Session, SessionState } from "../types";

const stateConfig: Record<
  SessionState,
  { label: string; border: string; bg: string; badge: string }
> = {
  "needs-input": {
    label: "Needs Input",
    border: "border-l-amber-400",
    bg: "bg-amber-950/40",
    badge: "bg-amber-400/20 text-amber-300",
  },
  working: {
    label: "Working",
    border: "border-l-blue-400",
    bg: "bg-blue-950/30",
    badge: "bg-blue-400/20 text-blue-300",
  },
  idle: {
    label: "Idle",
    border: "border-l-gray-600",
    bg: "bg-gray-800/50",
    badge: "bg-gray-600/20 text-gray-400",
  },
  errored: {
    label: "Errored",
    border: "border-l-red-400",
    bg: "bg-red-950/30",
    badge: "bg-red-400/20 text-red-300",
  },
  unknown: {
    label: "Unknown",
    border: "border-l-gray-500 border-dashed",
    bg: "bg-gray-800/30",
    badge: "bg-gray-500/20 text-gray-500",
  },
};

function dirLabel(path: string): string {
  const parts = path.split("/").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : path;
}

export function SessionCard({ session }: { session: Session }) {
  const config = stateConfig[session.state];

  return (
    <div
      className={`rounded-lg border-l-4 ${config.border} ${config.bg} p-4 shadow-md`}
    >
      <div className="flex items-start justify-between gap-2 mb-2">
        <h3 className="font-semibold text-gray-100 truncate">{session.name}</h3>
        <span
          className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${config.badge}`}
        >
          {config.label}
        </span>
      </div>

      {session.state === "needs-input" && session.needsInputType && (
        <span className="inline-block mb-2 rounded bg-amber-400/10 px-1.5 py-0.5 text-xs text-amber-200">
          {session.needsInputType === "permission" ? "Permission" : "Question"}
        </span>
      )}

      <p className="text-xs text-gray-400 mb-2 truncate" title={session.workingDirectory}>
        {dirLabel(session.workingDirectory)}
      </p>

      {session.lastOutputLines.length > 0 && (
        <div className="mt-auto rounded bg-gray-900/60 px-2 py-1.5">
          {session.lastOutputLines.slice(-2).map((line, i) => (
            <p key={i} className="text-xs font-mono text-gray-300 truncate">
              {line}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
