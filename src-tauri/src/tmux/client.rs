use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::process::Command;
use std::sync::LazyLock;

// Strips ALL ANSI sequences (CSI, OSC) — used for hashing and classification
static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?(\x07|\x1b\\)").unwrap()
});

// Strips only non-visual ANSI sequences (cursor movement, OSC) — preserves SGR color/style codes
static ANSI_CONTROL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[A-HJKSTfhilnsu]|\x1b\].*?(\x07|\x1b\\)").unwrap()
});

const CAPTURE_SCROLLBACK_LINES: u32 = 200;
// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum TmuxError {
    NotInstalled,
    ServerNotRunning,
    CommandFailed {
        stderr: String,
        exit_code: Option<i32>,
    },
    ParseError(String),
}

impl fmt::Display for TmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TmuxError::NotInstalled => write!(f, "tmux is not installed"),
            TmuxError::ServerNotRunning => write!(f, "tmux server is not running"),
            TmuxError::CommandFailed { stderr, exit_code } => {
                write!(f, "tmux command failed (exit {:?}): {}", exit_code, stderr)
            }
            TmuxError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Raw data structs (internal, not serialized to frontend)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TmuxSessionInfo {
    pub name: String,
    pub windows: u32,
    pub created_epoch: i64,
    pub attached: bool,
}

#[derive(Debug, Clone)]
pub struct TmuxPaneInfo {
    pub session_name: String,
    pub window_index: u32,
    pub pane_index: u32,
    pub pane_pid: u32,
    pub width: u32,
    pub height: u32,
    pub current_path: String,
}

impl TmuxPaneInfo {
    pub fn target(&self) -> String {
        format!("{}:{}.{}", self.session_name, self.window_index, self.pane_index)
    }
}

#[derive(Debug, Clone)]
pub struct CapturedPane {
    pub target: String,
    pub normalized_output: String,
    pub raw_output: String,
    pub output_hash: String,
    pub pane_title: Option<String>,
}

// ---------------------------------------------------------------------------
// Shell helpers
// ---------------------------------------------------------------------------

fn run_tmux(args: &[&str]) -> Result<String, TmuxError> {
    let output = Command::new("tmux")
        .args(args)
        .env("TERM", "xterm-256color")
        .output();

    match output {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(TmuxError::NotInstalled),
        Err(e) => Err(TmuxError::CommandFailed {
            stderr: e.to_string(),
            exit_code: None,
        }),
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if !o.status.success() {
                if stderr.contains("no server running")
                    || stderr.contains("no current client")
                    || stderr.contains("server not found")
                {
                    return Err(TmuxError::ServerNotRunning);
                }
                return Err(TmuxError::CommandFailed {
                    stderr,
                    exit_code: o.status.code(),
                });
            }
            Ok(String::from_utf8_lossy(&o.stdout).to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Pure parsing functions (testable without tmux)
// ---------------------------------------------------------------------------

pub fn strip_ansi(input: &str) -> String {
    ANSI_RE.replace_all(input, "").to_string()
}

/// Strip only cursor/control ANSI sequences, preserving SGR color codes (e.g. \x1b[31m).
pub fn strip_ansi_controls(input: &str) -> String {
    ANSI_CONTROL_RE.replace_all(input, "").to_string()
}

pub fn hash_output(normalized: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .take(8) // 8 bytes = 16 hex chars
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub fn parse_sessions_output(output: &str) -> Vec<TmuxSessionInfo> {
    output
        .trim()
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
                return None;
            }
            Some(TmuxSessionInfo {
                name: parts[0].to_string(),
                windows: parts[1].parse().unwrap_or(0),
                created_epoch: parts[2].parse().unwrap_or(0),
                attached: parts[3] == "1",
            })
        })
        .collect()
}

pub fn parse_panes_output(output: &str) -> Vec<TmuxPaneInfo> {
    output
        .trim()
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 7 {
                return None;
            }
            Some(TmuxPaneInfo {
                session_name: parts[0].to_string(),
                window_index: parts[1].parse().unwrap_or(0),
                pane_index: parts[2].parse().unwrap_or(0),
                pane_pid: parts[3].parse().unwrap_or(0),
                width: parts[4].parse().unwrap_or(0),
                height: parts[5].parse().unwrap_or(0),
                current_path: parts[6].to_string(),
            })
        })
        .collect()
}

/// Given raw `ps -o pid,ppid,comm` output and a root PID, check if any
/// descendant process has a command containing "claude".
pub fn find_claude_in_process_tree(ps_output: &str, pane_pid: u32) -> bool {
    let mut children: HashMap<u32, Vec<(u32, String)>> = HashMap::new();

    for line in ps_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let pid: u32 = match parts[0].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let ppid: u32 = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let comm = parts[2..].join(" ");
        children.entry(ppid).or_default().push((pid, comm));
    }

    // BFS from pane_pid
    let mut queue = vec![pane_pid];
    while let Some(pid) = queue.pop() {
        if let Some(kids) = children.get(&pid) {
            for (child_pid, comm) in kids {
                if comm.contains("claude") {
                    return true;
                }
                queue.push(*child_pid);
            }
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Public API (shells out to tmux / ps)
// ---------------------------------------------------------------------------

pub fn is_tmux_alive() -> bool {
    run_tmux(&["info"]).is_ok()
}

pub fn ensure_server() -> Result<(), TmuxError> {
    if is_tmux_alive() {
        return Ok(());
    }
    run_tmux(&["start-server"])?;
    Ok(())
}

pub fn list_sessions() -> Result<Vec<TmuxSessionInfo>, TmuxError> {
    let output = run_tmux(&[
        "list-sessions",
        "-F",
        "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
    ]);

    match output {
        Ok(stdout) => Ok(parse_sessions_output(&stdout)),
        Err(TmuxError::ServerNotRunning) => Ok(vec![]),
        Err(e) => Err(e),
    }
}

pub fn list_panes(session_name: Option<&str>) -> Result<Vec<TmuxPaneInfo>, TmuxError> {
    let format_str =
        "#{session_name}\t#{window_index}\t#{pane_index}\t#{pane_pid}\t#{pane_width}\t#{pane_height}\t#{pane_current_path}";

    let output = if let Some(name) = session_name {
        run_tmux(&["list-panes", "-F", format_str, "-t", name])
    } else {
        run_tmux(&["list-panes", "-F", format_str, "-a"])
    };

    match output {
        Ok(stdout) => Ok(parse_panes_output(&stdout)),
        Err(TmuxError::ServerNotRunning) => Ok(vec![]),
        Err(e) => Err(e),
    }
}

pub fn capture_pane(target: &str) -> Result<CapturedPane, TmuxError> {
    let scrollback = format!("-{}", CAPTURE_SCROLLBACK_LINES);
    // Plain capture (no ANSI) for hashing and classification
    let raw_output = run_tmux(&["capture-pane", "-p", "-S", &scrollback, "-t", target])?;
    // Capture with -e to preserve ANSI escape sequences for colored display
    let ansi_output = run_tmux(&["capture-pane", "-p", "-e", "-S", &scrollback, "-t", target])?;

    let pane_title = run_tmux(&["display-message", "-p", "-t", target, "#{pane_title}"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let normalized = strip_ansi(&raw_output).trim_end().to_string();
    let colored = strip_ansi_controls(&ansi_output).trim_end().to_string();
    let output_hash = hash_output(&normalized);

    Ok(CapturedPane {
        target: target.to_string(),
        normalized_output: normalized,
        raw_output: colored,
        output_hash,
        pane_title,
    })
}

/// Create a new tmux session with the given name and working directory,
/// then send the bootstrap command to start Claude Code in the new pane.
pub fn create_session(name: &str, working_dir: &str, command: &str) -> Result<(), TmuxError> {
    ensure_server()?;

    // Check for duplicate session name
    let sessions = list_sessions()?;
    if sessions.iter().any(|s| s.name == name) {
        return Err(TmuxError::CommandFailed {
            stderr: format!("Session '{}' already exists", name),
            exit_code: None,
        });
    }

    run_tmux(&["new-session", "-d", "-s", name, "-c", working_dir])?;
    run_tmux(&["set-option", "-t", name, "mouse", "on"])?;
    run_tmux(&["send-keys", "-t", name, command, "Enter"])?;
    Ok(())
}

/// Kill a tmux session by name.
pub fn kill_session(session_name: &str) -> Result<(), TmuxError> {
    run_tmux(&["kill-session", "-t", session_name])?;
    Ok(())
}

/// Rename a tmux session.
pub fn rename_session(old_name: &str, new_name: &str) -> Result<(), TmuxError> {
    run_tmux(&["rename-session", "-t", old_name, new_name])?;
    Ok(())
}

/// Return the tty of the first client attached to a tmux session, if any.
pub fn list_client_tty(session_name: &str) -> Option<String> {
    run_tmux(&["list-clients", "-t", session_name, "-F", "#{client_tty}"])
        .ok()
        .and_then(|output| {
            output
                .trim()
                .lines()
                .next()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
        })
}

/// Return the TTY of the first connected tmux client (any session), if any.
pub fn list_any_client_tty() -> Option<String> {
    run_tmux(&["list-clients", "-F", "#{client_tty}"])
        .ok()
        .and_then(|output| {
            output
                .trim()
                .lines()
                .next()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
        })
}

/// Switch an existing tmux client (identified by TTY) to a different session.
pub fn switch_client(client_tty: &str, session_name: &str) -> Result<(), TmuxError> {
    run_tmux(&["switch-client", "-c", client_tty, "-t", session_name])?;
    Ok(())
}

/// Get the full process table once and return it for reuse across panes.
pub fn get_process_table() -> String {
    Command::new("ps")
        .args(["-o", "pid,ppid,comm", "-ax"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Check if a claude process is running as a descendant of the given PID.
/// Takes a pre-fetched process table to avoid running `ps` per pane.
pub fn is_claude_running(ps_output: &str, pane_pid: u32) -> bool {
    find_claude_in_process_tree(ps_output, pane_pid)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- strip_ansi tests --

    #[test]
    fn test_strip_ansi_basic() {
        let input = "\x1b[32mhello\x1b[0m world";
        assert_eq!(strip_ansi(input), "hello world");
    }

    #[test]
    fn test_strip_ansi_osc() {
        // OSC sequence for setting terminal title
        let input = "\x1b]0;my title\x07some text";
        assert_eq!(strip_ansi(input), "some text");
    }

    #[test]
    fn test_strip_ansi_osc_st_terminator() {
        // OSC with ST (ESC \) terminator
        let input = "\x1b]0;my title\x1b\\some text";
        assert_eq!(strip_ansi(input), "some text");
    }

    #[test]
    fn test_strip_ansi_passthrough() {
        let input = "plain text with no escape codes";
        assert_eq!(strip_ansi(input), input);
    }

    #[test]
    fn test_strip_ansi_mixed() {
        let input = "\x1b[1;34m⏺\x1b[0m Writing file \x1b[4msrc/main.rs\x1b[0m";
        assert_eq!(strip_ansi(input), "⏺ Writing file src/main.rs");
    }

    #[test]
    fn test_strip_ansi_cursor_movement() {
        let input = "\x1b[2J\x1b[Hhello";
        assert_eq!(strip_ansi(input), "hello");
    }

    // -- hash_output tests --

    #[test]
    fn test_hash_output_deterministic() {
        let input = "hello world";
        assert_eq!(hash_output(input), hash_output(input));
    }

    #[test]
    fn test_hash_output_different() {
        assert_ne!(hash_output("hello"), hash_output("world"));
    }

    #[test]
    fn test_hash_output_length() {
        let hash = hash_output("test input");
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // -- parse_sessions_output tests --

    #[test]
    fn test_parse_list_sessions() {
        let output = "muxara-1\t3\t1711929600\t1\nmuxara-2\t1\t1711929700\t0\n";
        let sessions = parse_sessions_output(output);
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].name, "muxara-1");
        assert_eq!(sessions[0].windows, 3);
        assert_eq!(sessions[0].created_epoch, 1711929600);
        assert!(sessions[0].attached);
        assert_eq!(sessions[1].name, "muxara-2");
        assert!(!sessions[1].attached);
    }

    #[test]
    fn test_parse_list_sessions_empty() {
        assert!(parse_sessions_output("").is_empty());
        assert!(parse_sessions_output("  \n  ").is_empty());
    }

    #[test]
    fn test_parse_list_sessions_malformed() {
        let output = "only-two\tfields\n";
        assert!(parse_sessions_output(output).is_empty());
    }

    // -- parse_panes_output tests --

    #[test]
    fn test_parse_list_panes() {
        let output = "sess1\t0\t0\t12345\t200\t50\t/home/user/project\n";
        let panes = parse_panes_output(output);
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].session_name, "sess1");
        assert_eq!(panes[0].window_index, 0);
        assert_eq!(panes[0].pane_index, 0);
        assert_eq!(panes[0].pane_pid, 12345);
        assert_eq!(panes[0].width, 200);
        assert_eq!(panes[0].height, 50);
        assert_eq!(panes[0].current_path, "/home/user/project");
    }

    #[test]
    fn test_parse_list_panes_malformed() {
        let output = "sess1\t0\t0\n"; // only 3 fields, need 7
        assert!(parse_panes_output(output).is_empty());
    }

    #[test]
    fn test_pane_info_target() {
        let pane = TmuxPaneInfo {
            session_name: "sess1".to_string(),
            window_index: 2,
            pane_index: 1,
            pane_pid: 1234,
            width: 200,
            height: 50,
            current_path: "/tmp".to_string(),
        };
        assert_eq!(pane.target(), "sess1:2.1");
    }

    // -- process tree tests --

    #[test]
    fn test_find_claude_in_process_tree() {
        let ps_output = "\
  PID  PPID COMM
    1     0 init
  100     1 zsh
  200   100 claude
  300   200 node";
        assert!(find_claude_in_process_tree(ps_output, 100));
    }

    #[test]
    fn test_no_claude_in_process_tree() {
        let ps_output = "\
  PID  PPID COMM
    1     0 init
  100     1 zsh
  200   100 vim
  300   100 node";
        assert!(!find_claude_in_process_tree(ps_output, 100));
    }

    #[test]
    fn test_claude_nested_child() {
        let ps_output = "\
  PID  PPID COMM
    1     0 init
  100     1 zsh
  200   100 bash
  300   200 claude";
        assert!(find_claude_in_process_tree(ps_output, 100));
    }

    #[test]
    fn test_claude_in_different_tree() {
        // claude exists but is not a descendant of our pane_pid (100)
        let ps_output = "\
  PID  PPID COMM
    1     0 init
  100     1 zsh
  200   100 vim
  500     1 bash
  600   500 claude";
        assert!(!find_claude_in_process_tree(ps_output, 100));
    }

    #[test]
    fn test_empty_process_table() {
        assert!(!find_claude_in_process_tree("", 100));
    }
}
