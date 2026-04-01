use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;

use tauri::State;

use crate::session::Session;
use crate::store::SessionStore;
use crate::tmux::client;

#[tauri::command]
pub fn focus_session(session_id: String) -> Result<(), String> {
    // session_id is a pane target like "sess1:0.0" — extract the session name
    let session_name = session_id
        .split(':')
        .next()
        .unwrap_or(&session_id);

    // Verify the tmux session exists
    let sessions = client::list_sessions().map_err(|e| e.to_string())?;
    if !sessions.iter().any(|s| s.name == session_name) {
        return Err(format!("Session '{}' not found", session_name));
    }

    // Check if session already has an attached client (i.e. a terminal is already open)
    let client_tty = client::list_client_tty(session_name);

    let script = if let Some(tty) = client_tty {
        // Find the existing iTerm2 window by matching the tty and focus it
        format!(
            r#"tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if tty of s is "{}" then
                    select t
                    set index of w to 1
                    activate
                    return
                end if
            end repeat
        end repeat
    end repeat
    activate
end tell"#,
            tty
        )
    } else {
        // Resolve tmux absolute path (iTerm2's `command` doesn't use $PATH)
        let tmux_path = Command::new("which")
            .arg("tmux")
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            })
            .unwrap_or_else(|| "tmux".to_string());

        // No client attached — open a new tab in the current iTerm2 window
        // (or create a window if none exists)
        format!(
            r#"tell application "iTerm2"
    if (count of windows) is 0 then
        set newWindow to (create window with default profile command "{tmux} attach -t {sess}")
        tell newWindow
            set columns of current session of current tab to 200
            set rows of current session of current tab to 50
        end tell
    else
        tell current window
            create tab with default profile command "{tmux} attach -t {sess}"
        end tell
    end if
    activate
end tell"#,
            tmux = tmux_path, sess = session_name
        )
    };

    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_sessions(store: State<'_, Mutex<SessionStore>>) -> Vec<Session> {
    let tmux_alive = client::is_tmux_alive();

    if !tmux_alive {
        // Try to start the server; if it fails, return empty
        if client::ensure_server().is_err() {
            let mut store = store.lock().unwrap();
            store.reconcile(&[], &HashMap::new(), &HashMap::new(), false);
            return store.to_sessions();
        }
    }

    let panes = client::list_panes(None).unwrap_or_default();

    // Get process table once for all panes
    let ps_output = client::get_process_table();

    let mut captures = HashMap::new();
    let mut claude_status = HashMap::new();

    for pane in &panes {
        let target = pane.target();
        if let Ok(captured) = client::capture_pane(&target) {
            captures.insert(target.clone(), captured);
        }
        claude_status.insert(target, client::is_claude_running(&ps_output, pane.pane_pid));
    }

    let mut store = store.lock().unwrap();
    store.reconcile(&panes, &captures, &claude_status, true);
    store.to_sessions()
}
