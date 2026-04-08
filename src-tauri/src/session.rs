use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    NeedsInput,
    Working,
    Idle,
    Errored,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NeedsInputType {
    Permission,
    Question,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeState {
    pub tmux_alive: bool,
    pub claude_alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub name: String,
    pub state: SessionState,
    pub needs_input_type: Option<NeedsInputType>,
    pub is_in_plan_mode: Option<bool>,
    pub last_output_lines: Vec<String>,
    pub last_output_lines_ansi: Vec<String>,
    pub working_directory: String,
    pub last_changed_at: String,
    pub last_seen_at: String,
    pub created_at: String,
    pub previous_state: Option<String>,
    pub pane_title: Option<String>,
    pub runtime_state: RuntimeState,
    pub git_branch: Option<String>,
    pub is_worktree: Option<bool>,
}
