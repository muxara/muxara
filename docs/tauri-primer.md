# Tauri Primer

A quick reference for understanding Tauri in the context of Muxara.

## What is Tauri?

Tauri is a framework for building desktop apps using **web tech for the UI** and **Rust for the backend**. Instead of bundling Chromium (like Electron), it uses the OS's built-in webview (WebKit on macOS, WebView2 on Windows).

Why Tauri for Muxara:

- **Tiny footprint** -- no bundled browser, app stays at a few MB vs Electron's 100MB+
- **Rust backend** -- fast, safe system access for running tmux commands and parsing output
- **Web frontend** -- standard React/Tailwind, nothing special to learn on the UI side
- **Native window controls** -- always-on-top, min size, transparency, etc. are config options

## Architecture: the two halves

```
+----------------------------------+
|  Frontend (webview)              |  <-- React, runs in a native webview
|  HTML/CSS/JS -- what you see     |      (not a browser, no address bar)
+----------------------------------+
|  Backend (Rust)                  |  <-- Runs natively on the OS
|  System access, commands, IPC    |      (can spawn processes, read files, etc.)
+----------------------------------+
```

## IPC: how frontend and backend communicate

Frontend and backend communicate through **commands** -- Rust functions that the frontend can call via `invoke()`.

### Rust side -- define a command

```rust
#[tauri::command]
fn get_sessions() -> Vec<Session> {
    // return data -- this gets serialized to JSON automatically via serde
}
```

Register it when building the app:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![get_sessions])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Frontend side -- call the command

```typescript
import { invoke } from "@tauri-apps/api/core";

const sessions = await invoke<Session[]>("get_sessions");
```

`invoke()` sends a message to Rust, Rust runs the function, serializes the return value as JSON, and the frontend receives it. Serde (Rust's serialization library) handles the Rust-to-JSON conversion automatically.

## Configuration: `tauri.conf.json`

Controls window behavior, app metadata, and security. No code needed for basic window setup:

```jsonc
{
  "app": {
    "windows": [{
      "title": "Muxara",
      "width": 800,
      "height": 600,
      "minWidth": 400,
      "minHeight": 300,
      "resizable": true
    }]
  }
}
```

## Security model

Tauri v2 uses a **permissions system**. By default, the frontend cannot do anything dangerous. You explicitly grant capabilities (file access, shell commands, etc.) in config files under `src-tauri/capabilities/`. For Muxara, we need shell command permissions to run tmux.

## Dev workflow

```bash
cargo tauri dev     # starts Rust backend + frontend dev server with hot reload
cargo tauri build   # produces a distributable app (.dmg on macOS, .msi on Windows)
```

During dev, the frontend hot-reloads like a normal React app. Rust recompiles when you change backend code (slower, but only triggers when you modify Rust files).

## Key differences from pure web dev

| Web concept | Tauri equivalent |
|---|---|
| `fetch("/api/...")` | `invoke("command_name")` |
| Backend server | Rust process (no HTTP, direct IPC) |
| `process.env` | Tauri config + Rust env access |
| Node.js filesystem | Rust `std::fs` or Tauri's fs plugin |
| Running shell commands | Rust `std::process::Command` or Tauri shell plugin |

## Mental model for Muxara

- **React** handles everything visual -- cards, grid, polling, state differentiation
- **Rust** handles everything system -- talking to tmux, parsing output, classifying state
- **`invoke()`** is the bridge -- React asks Rust for data, Rust returns structured JSON

The frontend never touches tmux directly. Rust is the data provider, React is the presenter.
