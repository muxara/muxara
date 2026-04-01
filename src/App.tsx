import { useSessions } from "./hooks/useSessions";
import { SessionGrid } from "./components/SessionGrid";

function App() {
  const { sessions, loading, error } = useSessions();

  return (
    <div className="h-screen bg-gray-950 text-gray-100 p-4 flex flex-col overflow-hidden">
      <h1 className="text-xl font-semibold mb-4 shrink-0">Muxara</h1>
      <div className="flex-1 overflow-y-auto pb-2">
        <SessionGrid sessions={sessions} loading={loading} error={error} />
      </div>
    </div>
  );
}

export default App;
