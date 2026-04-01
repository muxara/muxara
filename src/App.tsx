import { useSessions } from "./hooks/useSessions";
import { SessionGrid } from "./components/SessionGrid";
import { NewSessionButton } from "./components/NewSessionButton";

function App() {
  const { sessions, loading, error } = useSessions();

  return (
    <div className="h-screen bg-gray-950 text-gray-100 p-4 flex flex-col overflow-hidden">
      <div className="flex items-center justify-between mb-4 shrink-0">
        <h1 className="text-xl font-semibold">Muxara</h1>
        <NewSessionButton />
      </div>
      <div className="flex-1 overflow-y-auto pb-2">
        <SessionGrid sessions={sessions} loading={loading} error={error} />
      </div>
    </div>
  );
}

export default App;
