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
pub struct Session {
    pub id: String,
    pub name: String,
    pub state: SessionState,
    pub needs_input_type: Option<NeedsInputType>,
    pub is_in_plan_mode: Option<bool>,
    pub last_output_lines: Vec<String>,
    pub working_directory: String,
    pub last_changed_at: String,
    pub last_seen_at: String,
    pub created_at: String,
    pub previous_state: Option<String>,
    pub pane_title: Option<String>,
}

pub fn mock_sessions() -> Vec<Session> {
    vec![
        Session {
            id: "sess_1".into(),
            name: "api-refactor".into(),
            state: SessionState::NeedsInput,
            needs_input_type: Some(NeedsInputType::Permission),
            is_in_plan_mode: Some(false),
            last_output_lines: vec![
                "Claude wants to edit src/api/handler.rs".into(),
                "Do you want to proceed? (y/n)".into(),
            ],
            working_directory: "/Users/dev/projects/api-service".into(),
            last_changed_at: "2026-04-01T10:30:00Z".into(),
            last_seen_at: "2026-04-01T10:30:05Z".into(),
            created_at: "2026-04-01T09:00:00Z".into(),
            previous_state: Some("working".into()),
            pane_title: Some("claude: api-refactor".into()),
        },
        Session {
            id: "sess_2".into(),
            name: "auth-migration".into(),
            state: SessionState::NeedsInput,
            needs_input_type: Some(NeedsInputType::Question),
            is_in_plan_mode: Some(false),
            last_output_lines: vec![
                "Should I use JWT or session-based auth for the new endpoint?".into(),
            ],
            working_directory: "/Users/dev/projects/auth-service".into(),
            last_changed_at: "2026-04-01T10:28:00Z".into(),
            last_seen_at: "2026-04-01T10:30:05Z".into(),
            created_at: "2026-04-01T08:30:00Z".into(),
            previous_state: Some("working".into()),
            pane_title: Some("claude: auth-migration".into()),
        },
        Session {
            id: "sess_3".into(),
            name: "test-coverage".into(),
            state: SessionState::Working,
            needs_input_type: None,
            is_in_plan_mode: Some(false),
            last_output_lines: vec![
                "Writing tests for UserService...".into(),
                "Created test file: tests/user_service_test.rs".into(),
            ],
            working_directory: "/Users/dev/projects/core-lib".into(),
            last_changed_at: "2026-04-01T10:30:02Z".into(),
            last_seen_at: "2026-04-01T10:30:05Z".into(),
            created_at: "2026-04-01T10:00:00Z".into(),
            previous_state: Some("working".into()),
            pane_title: Some("claude: test-coverage".into()),
        },
        Session {
            id: "sess_4".into(),
            name: "docs-update".into(),
            state: SessionState::Idle,
            needs_input_type: None,
            is_in_plan_mode: Some(false),
            last_output_lines: vec![
                "Done. Updated README.md and CONTRIBUTING.md.".into(),
            ],
            working_directory: "/Users/dev/projects/muxara".into(),
            last_changed_at: "2026-04-01T10:15:00Z".into(),
            last_seen_at: "2026-04-01T10:30:05Z".into(),
            created_at: "2026-04-01T09:45:00Z".into(),
            previous_state: Some("working".into()),
            pane_title: Some("claude: docs-update".into()),
        },
        Session {
            id: "sess_5".into(),
            name: "deploy-fix".into(),
            state: SessionState::Errored,
            needs_input_type: None,
            is_in_plan_mode: Some(false),
            last_output_lines: vec![
                "Error: process exited with code 1".into(),
                "ENOENT: no such file or directory".into(),
            ],
            working_directory: "/Users/dev/projects/infra".into(),
            last_changed_at: "2026-04-01T10:20:00Z".into(),
            last_seen_at: "2026-04-01T10:30:05Z".into(),
            created_at: "2026-04-01T07:00:00Z".into(),
            previous_state: Some("working".into()),
            pane_title: Some("claude: deploy-fix".into()),
        },
    ]
}
