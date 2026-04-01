import type { Session } from "../types";
import { SessionCard } from "./SessionCard";

interface SessionGridProps {
  sessions: Session[];
  loading: boolean;
  error: string | null;
}

export function SessionGrid({ sessions, loading, error }: SessionGridProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        <p>Loading sessions...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64 text-red-400">
        <p>Failed to load sessions: {error}</p>
      </div>
    );
  }

  if (sessions.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        <p>No active sessions</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      {sessions.map((session) => (
        <SessionCard key={session.id} session={session} />
      ))}
    </div>
  );
}
