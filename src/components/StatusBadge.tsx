import type { SessionState } from "../types";

const dotColor: Record<SessionState, string> = {
  "needs-input": "bg-amber-400",
  working: "bg-blue-400 animate-pulse",
  idle: "bg-gray-500",
  errored: "bg-red-400",
  unknown: "bg-gray-600",
};

export function StatusBadge({ state }: { state: SessionState }) {
  return (
    <span
      className={`inline-block h-2.5 w-2.5 shrink-0 rounded-full ${dotColor[state]}`}
      aria-label={state}
    />
  );
}
