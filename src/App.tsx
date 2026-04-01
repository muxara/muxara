import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SessionGrid } from "./components/SessionGrid";
import type { Session } from "./types";

function App() {
  const [sessions, setSessions] = useState<Session[]>([]);

  useEffect(() => {
    const fetchSessions = async () => {
      try {
        const data = await invoke<Session[]>("get_sessions");
        setSessions(data);
      } catch (err) {
        console.error("Failed to fetch sessions:", err);
      }
    };

    fetchSessions();
    const interval = setInterval(fetchSessions, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 p-4">
      <h1 className="text-xl font-semibold mb-4">Muxara</h1>
      <SessionGrid sessions={sessions} />
    </div>
  );
}

export default App;
