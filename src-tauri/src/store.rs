use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::git;
use crate::session::{NeedsInputType, RuntimeState, Session, SessionState};
use crate::tmux::classifier::{self, ClassifierInput};
use crate::tmux::client::{CapturedPane, TmuxPaneInfo};

const DEFAULT_OUTPUT_TAIL_LINES: usize = 20;
const DEFAULT_COOLOFF_SECS: f64 = 300.0;

fn state_priority(state: &SessionState) -> u8 {
    match state {
        SessionState::NeedsInput => 0,
        SessionState::Errored => 1,
        SessionState::Working => 2,
        // Idle and Unknown share priority so they stay in stable
        // alphabetical order instead of shuffling on every poll.
        SessionState::Idle => 3,
        SessionState::Unknown => 3,
    }
}

#[derive(Debug, Clone)]
pub struct TrackedSession {
    pub tmux_session_name: String,
    pub pane_target: String,
    pub pane_pid: u32,
    pub working_directory: String,
    pub display_name: String,
    pub pane_title: Option<String>,
    pub last_output_hash: Option<String>,
    pub last_output_lines: Vec<String>,
    pub last_output_lines_ansi: Vec<String>,
    pub last_changed_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub claude_alive: bool,
    pub tmux_alive: bool,
    pub state: SessionState,
    pub needs_input_type: Option<NeedsInputType>,
    pub is_in_plan_mode: Option<bool>,
    pub previous_state: Option<SessionState>,
    pub consecutive_idle_count: u32,
    pub git_branch: Option<String>,
    pub is_worktree: Option<bool>,
    pub last_git_checked_dir: Option<String>,
}

pub struct SessionStore {
    sessions: HashMap<String, TrackedSession>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn reconcile(
        &mut self,
        panes: &[TmuxPaneInfo],
        captures: &HashMap<String, CapturedPane>,
        claude_status: &HashMap<String, bool>,
        tmux_alive: bool,
        output_tail_lines: usize,
        cooloff_secs: f64,
    ) {
        let now = Utc::now();
        let mut seen_targets: Vec<String> = Vec::new();

        for pane in panes {
            let target = pane.target();
            seen_targets.push(target.clone());

            let session = self.sessions.entry(target.clone()).or_insert_with(|| {
                TrackedSession {
                    tmux_session_name: pane.session_name.clone(),
                    pane_target: target.clone(),
                    pane_pid: pane.pane_pid,
                    working_directory: pane.current_path.clone(),
                    display_name: pane.session_name.clone(),
                    pane_title: None,
                    last_output_hash: None,
                    last_output_lines: vec![],
                    last_output_lines_ansi: vec![],
                    last_changed_at: now,
                    last_seen_at: now,
                    created_at: now,
                    claude_alive: false,
                    tmux_alive,
                    state: SessionState::Unknown,
                    needs_input_type: None,
                    is_in_plan_mode: None,
                    previous_state: None,
                    consecutive_idle_count: 0,
                    git_branch: None,
                    is_worktree: None,
                    last_git_checked_dir: None,
                }
            });

            session.tmux_alive = tmux_alive;
            session.pane_pid = pane.pane_pid;
            session.working_directory = pane.current_path.clone();
            session.last_seen_at = now;

            // Refresh git info only when working directory changes
            let dir_changed = session
                .last_git_checked_dir
                .as_ref()
                .map(|d| d != &pane.current_path)
                .unwrap_or(true);
            if dir_changed {
                session.git_branch = git::detect_branch(&pane.current_path);
                session.is_worktree = Some(git::is_worktree(&pane.current_path));
                session.last_git_checked_dir = Some(pane.current_path.clone());
            }

            if let Some(alive) = claude_status.get(&target) {
                session.claude_alive = *alive;
            }

            if let Some(captured) = captures.get(&target) {
                session.pane_title = captured.pane_title.clone();

                let is_first_capture = session.last_output_hash.is_none();
                let hash_changed = session
                    .last_output_hash
                    .as_ref()
                    .map(|h| h != &captured.output_hash)
                    .unwrap_or(true);

                if hash_changed {
                    session.last_changed_at = now;
                    session.last_output_hash = Some(captured.output_hash.clone());
                    session.last_output_lines = captured
                        .normalized_output
                        .lines()
                        .rev()
                        .take(output_tail_lines)
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    session.last_output_lines_ansi = captured
                        .raw_output
                        .lines()
                        .rev()
                        .take(output_tail_lines)
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                }

                // Classify session state
                let seconds_since_change =
                    (now - session.last_changed_at).num_milliseconds() as f64 / 1000.0;
                // On first capture (no previous hash), pass None so the classifier
                // treats it as initial state rather than a delta. On subsequent
                // captures, pass a sentinel if changed so the classifier sees a delta.
                let previous_hash_for_classifier = if is_first_capture {
                    None
                } else if hash_changed {
                    Some("__changed__")
                } else {
                    Some(captured.output_hash.as_str())
                };
                let classifier_input = ClassifierInput {
                    normalized_output: &captured.normalized_output,
                    output_hash: &captured.output_hash,
                    previous_hash: previous_hash_for_classifier,
                    previous_state: Some(&session.state),
                    seconds_since_last_change: seconds_since_change,
                    consecutive_idle_count: session.consecutive_idle_count,
                    cooloff_secs,
                };
                let result = classifier::classify(&classifier_input);

                // Update consecutive idle count for debounce.
                // Increment when debounce is actively holding Working state,
                // so the counter eventually reaches the threshold and releases.
                if result.debounce_applied {
                    session.consecutive_idle_count += 1;
                } else {
                    session.consecutive_idle_count = 0;
                }

                // Track state transitions
                if !matches!((&session.state, &result.state),
                    (SessionState::NeedsInput, SessionState::NeedsInput)
                    | (SessionState::Working, SessionState::Working)
                    | (SessionState::Idle, SessionState::Idle)
                    | (SessionState::Errored, SessionState::Errored)
                    | (SessionState::Unknown, SessionState::Unknown)
                ) {
                    session.previous_state = Some(session.state.clone());
                }

                session.state = result.state;
                session.needs_input_type = result.needs_input_type;
                session.is_in_plan_mode = result.is_in_plan_mode;
            }
        }

        // Prune sessions that no longer exist in tmux
        self.sessions.retain(|target, _| seen_targets.contains(target));
    }

    pub fn get_session(&self, pane_target: &str) -> Option<&TrackedSession> {
        self.sessions.get(pane_target)
    }

    pub fn remove_session(&mut self, pane_target: &str) {
        self.sessions.remove(pane_target);
    }

    pub fn rename_session(&mut self, old_pane_target: &str, new_name: &str) {
        // The pane target changes when a session is renamed (e.g. "old:0.0" → "new:0.0")
        if let Some(mut tracked) = self.sessions.remove(old_pane_target) {
            tracked.tmux_session_name = new_name.to_string();
            tracked.display_name = new_name.to_string();
            // Reconstruct pane target with new session name
            let new_target = format!(
                "{}:{}",
                new_name,
                old_pane_target.split_once(':').map(|(_, rest)| rest).unwrap_or("0.0")
            );
            tracked.pane_target = new_target.clone();
            self.sessions.insert(new_target, tracked);
        }
    }

    pub fn to_sessions(&self) -> Vec<Session> {
        let mut sessions: Vec<Session> = self
            .sessions
            .values()
            .map(|tracked| Session {
                id: tracked.pane_target.clone(),
                name: tracked.display_name.clone(),
                state: tracked.state.clone(),
                needs_input_type: tracked.needs_input_type.clone(),
                is_in_plan_mode: tracked.is_in_plan_mode,
                last_output_lines: tracked.last_output_lines.clone(),
                last_output_lines_ansi: tracked.last_output_lines_ansi.clone(),
                working_directory: tracked.working_directory.clone(),
                last_changed_at: tracked.last_changed_at.to_rfc3339(),
                last_seen_at: tracked.last_seen_at.to_rfc3339(),
                created_at: tracked.created_at.to_rfc3339(),
                previous_state: tracked
                    .previous_state
                    .as_ref()
                    .map(|s| format!("{:?}", s).to_lowercase()),
                pane_title: tracked.pane_title.clone(),
                runtime_state: RuntimeState {
                    tmux_alive: tracked.tmux_alive,
                    claude_alive: tracked.claude_alive,
                },
                git_branch: tracked.git_branch.clone(),
                is_worktree: tracked.is_worktree,
            })
            .collect();

        sessions.sort_by(|a, b| {
            state_priority(&a.state)
                .cmp(&state_priority(&b.state))
                .then_with(|| a.name.cmp(&b.name))
        });

        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pane(session: &str, window: u32, pane: u32, pid: u32, path: &str) -> TmuxPaneInfo {
        TmuxPaneInfo {
            session_name: session.to_string(),
            window_index: window,
            pane_index: pane,
            pane_pid: pid,
            width: 200,
            height: 50,
            current_path: path.to_string(),
        }
    }

    fn make_capture(target: &str, output: &str, hash: &str) -> CapturedPane {
        CapturedPane {
            target: target.to_string(),
            normalized_output: output.to_string(),
            raw_output: output.to_string(),
            output_hash: hash.to_string(),
            pane_title: Some("claude: test".to_string()),
        }
    }

    #[test]
    fn test_reconcile_adds_new_sessions() {
        let mut store = SessionStore::new();
        let panes = vec![make_pane("sess1", 0, 0, 1234, "/tmp")];
        let mut captures = HashMap::new();
        captures.insert(
            "sess1:0.0".to_string(),
            make_capture("sess1:0.0", "hello\nworld", "abc123"),
        );
        let mut claude_status = HashMap::new();
        claude_status.insert("sess1:0.0".to_string(), true);

        store.reconcile(&panes, &captures, &claude_status, true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);

        assert_eq!(store.sessions.len(), 1);
        let session = &store.sessions["sess1:0.0"];
        assert_eq!(session.display_name, "sess1");
        assert!(session.claude_alive);
        assert!(session.tmux_alive);
        assert_eq!(session.last_output_lines, vec!["hello", "world"]);
    }

    #[test]
    fn test_reconcile_updates_existing() {
        let mut store = SessionStore::new();
        let panes = vec![make_pane("sess1", 0, 0, 1234, "/tmp")];

        // First reconcile
        let mut captures = HashMap::new();
        captures.insert(
            "sess1:0.0".to_string(),
            make_capture("sess1:0.0", "first output", "hash1"),
        );
        store.reconcile(&panes, &captures, &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        let first_changed = store.sessions["sess1:0.0"].last_changed_at;

        // Second reconcile with different hash
        let mut captures2 = HashMap::new();
        captures2.insert(
            "sess1:0.0".to_string(),
            make_capture("sess1:0.0", "second output", "hash2"),
        );
        store.reconcile(&panes, &captures2, &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        let second_changed = store.sessions["sess1:0.0"].last_changed_at;

        assert!(second_changed >= first_changed);
        assert_eq!(
            store.sessions["sess1:0.0"].last_output_lines,
            vec!["second output"]
        );
    }

    #[test]
    fn test_reconcile_prunes_disappeared() {
        let mut store = SessionStore::new();

        // Add a session
        let panes = vec![make_pane("sess1", 0, 0, 1234, "/tmp")];
        store.reconcile(&panes, &HashMap::new(), &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        assert_eq!(store.sessions.len(), 1);

        // Reconcile with empty panes
        store.reconcile(&[], &HashMap::new(), &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        assert_eq!(store.sessions.len(), 0);
    }

    #[test]
    fn test_reconcile_preserves_created_at() {
        let mut store = SessionStore::new();
        let panes = vec![make_pane("sess1", 0, 0, 1234, "/tmp")];

        store.reconcile(&panes, &HashMap::new(), &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        let created = store.sessions["sess1:0.0"].created_at;

        store.reconcile(&panes, &HashMap::new(), &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        assert_eq!(store.sessions["sess1:0.0"].created_at, created);
    }

    #[test]
    fn test_to_sessions_format() {
        let mut store = SessionStore::new();
        let panes = vec![make_pane("sess1", 0, 0, 1234, "/home/user")];
        let mut captures = HashMap::new();
        captures.insert(
            "sess1:0.0".to_string(),
            make_capture("sess1:0.0", "output line", "hash1"),
        );
        store.reconcile(&panes, &captures, &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);

        let sessions = store.to_sessions();
        assert_eq!(sessions.len(), 1);
        let s = &sessions[0];
        assert_eq!(s.id, "sess1:0.0");
        assert_eq!(s.name, "sess1");
        assert_eq!(s.working_directory, "/home/user");
        assert!(s.created_at.contains('T')); // ISO 8601
        assert!(s.last_seen_at.contains('T'));
        assert!(s.last_changed_at.contains('T'));
        assert!(s.runtime_state.tmux_alive);
    }

    #[test]
    fn test_reconcile_no_tmux() {
        let mut store = SessionStore::new();
        // First add a session with tmux alive
        let panes = vec![make_pane("sess1", 0, 0, 1234, "/tmp")];
        store.reconcile(&panes, &HashMap::new(), &HashMap::new(), true, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        assert!(store.sessions["sess1:0.0"].tmux_alive);

        // Reconcile with no panes and tmux_alive=false prunes all sessions
        store.reconcile(&[], &HashMap::new(), &HashMap::new(), false, DEFAULT_OUTPUT_TAIL_LINES, DEFAULT_COOLOFF_SECS);
        assert_eq!(store.sessions.len(), 0);
    }
}
