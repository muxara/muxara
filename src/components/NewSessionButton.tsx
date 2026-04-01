import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export function NewSessionButton() {
  const [formOpen, setFormOpen] = useState(false);
  const [name, setName] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function reset() {
    setName("");
    setWorkingDir("");
    setError(null);
    setFormOpen(false);
  }

  async function pickDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      setWorkingDir(selected as string);
      setError(null);
    }
  }

  async function handleCreate() {
    if (!workingDir.trim()) {
      setError("Working directory is required");
      return;
    }

    const sessionName = name.trim() || `muxara-${Date.now()}`;

    setCreating(true);
    setError(null);
    try {
      await invoke("create_session", {
        name: sessionName,
        workingDir: workingDir.trim(),
      });
      reset();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  }

  if (!formOpen) {
    return (
      <button
        onClick={() => setFormOpen(true)}
        className="shrink-0 w-8 h-8 flex items-center justify-center rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-gray-200 transition-colors text-lg font-light"
        title="New session"
      >
        +
      </button>
    );
  }

  return (
    <div className="shrink-0 flex items-center gap-2">
      <input
        autoFocus
        type="text"
        placeholder="Session name (optional)"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") handleCreate();
          if (e.key === "Escape") reset();
        }}
        className="h-8 px-2 rounded-md bg-gray-800 border border-gray-700 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-500 w-36"
      />
      <button
        onClick={pickDirectory}
        className={`h-8 px-2 rounded-md border text-sm truncate max-w-56 text-left transition-colors ${
          workingDir
            ? "bg-gray-800 border-gray-600 text-gray-200"
            : "bg-gray-800 border-gray-700 text-gray-500 hover:border-gray-500"
        }`}
      >
        {workingDir
          ? workingDir.split("/").filter(Boolean).slice(-2).join("/")
          : "Pick directory..."}
      </button>
      <button
        onClick={handleCreate}
        disabled={creating || !workingDir.trim()}
        className="h-8 px-3 rounded-md bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-sm text-white transition-colors"
      >
        {creating ? "..." : "Create"}
      </button>
      <button
        onClick={reset}
        className="h-8 px-2 rounded-md text-gray-500 hover:text-gray-300 text-sm transition-colors"
      >
        Cancel
      </button>
      {error && (
        <span className="text-xs text-red-400 truncate max-w-48">{error}</span>
      )}
    </div>
  );
}
