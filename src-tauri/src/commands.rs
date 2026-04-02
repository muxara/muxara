use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;

use tauri::State;

use crate::git;
use crate::preferences::{ConfigDir, Preferences};
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

    // Check if ANY tmux client is attached (not just this session's).
    // If so, switch it to the target session instead of opening a new tab.
    let any_client_tty = client::list_any_client_tty();
    let mut needs_new_tab = true;

    if let Some(ref tty) = any_client_tty {
        if client::switch_client(tty, session_name).is_ok() {
            needs_new_tab = false;
            // Focus the existing iTerm2 tab
            let focus_script = format!(
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
            );
            Command::new("osascript")
                .args(["-e", &focus_script])
                .spawn()
                .map_err(|e| format!("Failed to focus terminal: {}", e))?;
        }
        // If switch_client failed (stale client), fall through to open a new tab
    }

    if needs_new_tab {
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
        let attach_script = format!(
            r#"tell application "iTerm2"
    if (count of windows) is 0 then
        set newWindow to (create window with default profile command "{tmux} attach-session -t \"{sess}\"")
        tell newWindow
            set columns of current session of current tab to 200
            set rows of current session of current tab to 50
        end tell
    else
        tell current window
            create tab with default profile command "{tmux} attach-session -t \"{sess}\""
        end tell
    end if
    activate
end tell"#,
            tmux = tmux_path, sess = session_name
        );
        Command::new("osascript")
            .args(["-e", &attach_script])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn create_session(
    name: String,
    working_dir: String,
    command: String,
    prefs: State<'_, Mutex<Preferences>>,
) -> Result<(), String> {
    if working_dir.is_empty() {
        return Err("Working directory is required".to_string());
    }
    let base_cmd = if command.trim().is_empty() { "claude" } else { command.trim() };

    let use_worktree = {
        let p = prefs.lock().unwrap();
        p.effective_use_worktree(&working_dir)
    };

    let final_cmd = if use_worktree && git::is_git_repo(&working_dir) {
        format!("{} -w {}", base_cmd, &name)
    } else {
        base_cmd.to_string()
    };

    client::create_session(&name, &working_dir, &final_cmd).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_git_repo(path: String) -> bool {
    git::is_git_repo(&path)
}

#[tauri::command]
pub fn resolve_bootstrap_command(
    working_dir: String,
    prefs: State<'_, Mutex<Preferences>>,
) -> String {
    let p = prefs.lock().unwrap();
    p.effective_bootstrap_command(&working_dir).to_string()
}

#[tauri::command]
pub fn kill_session(session_id: String, store: State<'_, Mutex<SessionStore>>) -> Result<(), String> {
    let session_name = session_id
        .split(':')
        .next()
        .unwrap_or(&session_id);

    client::kill_session(session_name).map_err(|e| e.to_string())?;

    // Remove from the store immediately so the UI updates without waiting for the next poll
    let mut store = store.lock().unwrap();
    store.remove_session(&session_id);

    Ok(())
}

#[tauri::command]
pub fn rename_session(session_id: String, new_name: String, store: State<'_, Mutex<SessionStore>>) -> Result<(), String> {
    if new_name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    let session_name = session_id
        .split(':')
        .next()
        .unwrap_or(&session_id);

    // Check for duplicate name
    let sessions = client::list_sessions().map_err(|e| e.to_string())?;
    if sessions.iter().any(|s| s.name == new_name) {
        return Err(format!("Session '{}' already exists", new_name));
    }

    client::rename_session(session_name, &new_name).map_err(|e| e.to_string())?;

    // Update the store so the UI reflects the rename immediately
    let mut store = store.lock().unwrap();
    store.rename_session(&session_id, &new_name);

    Ok(())
}

#[tauri::command]
pub fn get_sessions(
    store: State<'_, Mutex<SessionStore>>,
    prefs: State<'_, Mutex<Preferences>>,
) -> Vec<Session> {
    let (output_lines, cooloff_secs) = {
        let p = prefs.lock().unwrap();
        (p.output_lines, p.cooloff_minutes * 60.0)
    };

    let tmux_alive = client::is_tmux_alive();

    if !tmux_alive {
        // Try to start the server; if it fails, return empty
        if client::ensure_server().is_err() {
            let mut store = store.lock().unwrap();
            store.reconcile(&[], &HashMap::new(), &HashMap::new(), false, output_lines, cooloff_secs);
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
    store.reconcile(&panes, &captures, &claude_status, true, output_lines, cooloff_secs);
    store.to_sessions()
}

#[tauri::command]
pub fn get_preferences(prefs: State<'_, Mutex<Preferences>>) -> Preferences {
    prefs.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_preferences(
    new_prefs: Preferences,
    prefs: State<'_, Mutex<Preferences>>,
    config_dir: State<'_, ConfigDir>,
) -> Result<(), String> {
    new_prefs.validate()?;
    new_prefs.save(&config_dir.0)?;
    *prefs.lock().unwrap() = new_prefs;
    Ok(())
}
