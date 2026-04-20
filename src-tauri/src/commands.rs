use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;

use tauri::State;

use crate::git;
use crate::preferences::{ConfigDir, Preferences};
use crate::session::Session;
use crate::store::SessionStore;
use crate::tmux::client;

/// Resolve the absolute path to tmux.
/// Terminal apps launched via AppleScript `command` parameters may not inherit
/// the user's shell $PATH, so we need the full path. Tries `which` first
/// (works when Muxara is launched from a terminal), then checks common
/// installation paths (works when launched from Spotlight/Dock where PATH
/// is the system default and won't include Homebrew).
fn resolve_tmux_path() -> String {
    // Try `which` first
    if let Some(path) = Command::new("which")
        .arg("tmux")
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() || !s.starts_with('/') {
                None
            } else {
                Some(s)
            }
        })
    {
        return path;
    }

    // Fallback: check common installation paths
    for candidate in &[
        "/opt/homebrew/bin/tmux", // Homebrew on Apple Silicon
        "/usr/local/bin/tmux",    // Homebrew on Intel, MacPorts
        "/usr/bin/tmux",          // System install
    ] {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }

    "tmux".to_string()
}

/// Escape a string for safe inclusion inside an AppleScript double-quoted string.
/// AppleScript's only escape is `\"` for a literal quote and `\\` for a literal
/// backslash.
fn applescript_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Focus an existing iTerm2 tab that owns the given tty.
fn focus_iterm2_tab(tty: &str) -> Result<(), String> {
    let tty = applescript_escape(tty);
    let script = format!(
        r#"tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if tty of s is "{tty}" then
                    select t
                    set index of w to 1
                    activate
                    return
                end if
            end repeat
        end repeat
    end repeat
    activate
end tell"#
    );
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to focus terminal: {}", e))?;
    Ok(())
}

/// Open a new iTerm2 tab (or window) attached to the given tmux session.
fn open_iterm2_session(session_name: &str, tmux_path: &str) -> Result<(), String> {
    let tmux = applescript_escape(tmux_path);
    let sess = applescript_escape(session_name);
    let script = format!(
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
end tell"#
    );
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;
    Ok(())
}

/// Focus an existing Terminal.app tab that owns the given tty.
fn focus_terminal_tab(tty: &str) -> Result<(), String> {
    let tty = applescript_escape(tty);
    let script = format!(
        r#"tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            if tty of t is "{tty}" then
                set selected tab of w to t
                set index of w to 1
                activate
                return
            end if
        end repeat
    end repeat
    activate
end tell"#
    );
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to focus terminal: {}", e))?;
    Ok(())
}

/// Check whether the configured terminal app owns the given tty.
/// Used to avoid reusing a tmux client attached in the wrong terminal.
/// Checks if the app is running first to avoid accidentally launching it.
fn terminal_owns_tty(terminal_app: &str, tty: &str) -> bool {
    let app_name = match terminal_app {
        "terminal" => "Terminal",
        _ => "iTerm2",
    };

    // Check if the app is running first — `tell application` would launch it
    let running_check = format!(r#"application "{}" is running"#, app_name);
    let is_running = Command::new("osascript")
        .args(["-e", &running_check])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);
    if !is_running {
        return false;
    }

    let tty = applescript_escape(tty);
    let script = match terminal_app {
        "terminal" => format!(
            r#"tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            if tty of t is "{tty}" then return true
        end repeat
    end repeat
    return false
end tell"#
        ),
        _ => format!(
            r#"tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if tty of s is "{tty}" then return true
            end repeat
        end repeat
    end repeat
    return false
end tell"#
        ),
    };
    Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false)
}

/// Open a new Terminal.app window attached to the given tmux session.
fn open_terminal_session(session_name: &str, tmux_path: &str) -> Result<(), String> {
    let tmux = applescript_escape(tmux_path);
    let sess = applescript_escape(session_name);
    // The command is prefixed with spaces to survive shell plugins that read
    // from stdin during initialization (e.g. oh-my-zsh's update check does
    // `read -k 1` which consumes the first character from the stdin buffer).
    // If a character is consumed, it's a harmless space. If nothing consumes
    // it, leading spaces are ignored by sh/zsh command parsing.
    let script = format!(
        r#"tell application "Terminal"
    do script "    {tmux} attach-session -t \"{sess}\""
    activate
end tell"#
    );
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn focus_session(
    session_id: String,
    prefs: State<'_, Mutex<Preferences>>,
) -> Result<(), String> {
    // session_id is a pane target like "sess1:0.0" — extract the session name
    let session_name = session_id.split(':').next().unwrap_or(&session_id);

    // Verify the tmux session exists
    let sessions = client::list_sessions().map_err(|e| e.to_string())?;
    if !sessions.iter().any(|s| s.name == session_name) {
        return Err(format!("Session '{}' not found", session_name));
    }

    let terminal_app = prefs.lock().unwrap().terminal_app.clone();

    // Look through ALL attached tmux clients and find one owned by the
    // configured terminal app. This avoids reusing a client attached in
    // the wrong terminal (e.g. iTerm2 tab still open after switching to
    // Terminal.app in settings).
    let all_ttys = client::list_all_client_ttys();
    let mut switched = false;

    for tty in &all_ttys {
        if terminal_owns_tty(&terminal_app, tty)
            && client::switch_client(tty, session_name).is_ok()
        {
            switched = true;
            match terminal_app.as_str() {
                "terminal" => focus_terminal_tab(tty)?,
                _ => focus_iterm2_tab(tty)?,
            }
            break;
        }
    }

    if !switched {
        let tmux_path = resolve_tmux_path();
        match terminal_app.as_str() {
            "terminal" => open_terminal_session(session_name, &tmux_path)?,
            _ => open_iterm2_session(session_name, &tmux_path)?,
        }
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
    let base_cmd = if command.trim().is_empty() {
        "claude"
    } else {
        command.trim()
    };

    let use_worktree = {
        let p = prefs.lock().unwrap();
        p.effective_use_worktree(&working_dir)
    };

    let final_cmd = if use_worktree && git::is_git_repo(&working_dir) {
        let wt_name = git::sanitize_worktree_name(&name);
        format!("{} -w {}", base_cmd, wt_name)
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
pub fn kill_session(
    session_id: String,
    store: State<'_, Mutex<SessionStore>>,
) -> Result<(), String> {
    let session_name = session_id.split(':').next().unwrap_or(&session_id);

    // Check if session is in a worktree — if so, block on uncommitted changes
    // and clean up the worktree after killing.
    let worktree_path = {
        let store = store.lock().unwrap();
        store.get_session(&session_id).and_then(|s| {
            if s.is_worktree == Some(true) {
                Some(s.working_directory.clone())
            } else {
                None
            }
        })
    };

    if let Some(ref wt_path) = worktree_path {
        if git::has_uncommitted_changes(wt_path) {
            return Err(format!(
                "Session '{}' has uncommitted changes in its worktree. Commit or discard changes before killing.",
                session_name
            ));
        }
    }

    client::kill_session(session_name).map_err(|e| e.to_string())?;

    // Remove from the store immediately so the UI updates without waiting for the next poll
    let mut store = store.lock().unwrap();
    store.remove_session(&session_id);

    // Clean up the worktree after killing the tmux session
    if let Some(ref wt_path) = worktree_path {
        if let Err(e) = git::remove_worktree(wt_path) {
            eprintln!("Warning: failed to remove worktree: {}", e);
            // Don't fail the kill — the tmux session is already dead
        }
    }

    Ok(())
}

#[tauri::command]
pub fn rename_session(
    session_id: String,
    new_name: String,
    store: State<'_, Mutex<SessionStore>>,
) -> Result<(), String> {
    if new_name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    let session_name = session_id.split(':').next().unwrap_or(&session_id);

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
            store.reconcile(
                &[],
                &HashMap::new(),
                &HashMap::new(),
                false,
                output_lines,
                cooloff_secs,
            );
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
    store.reconcile(
        &panes,
        &captures,
        &claude_status,
        true,
        output_lines,
        cooloff_secs,
    );
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
