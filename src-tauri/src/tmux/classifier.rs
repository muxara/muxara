use regex::Regex;
use std::sync::LazyLock;

use crate::session::{NeedsInputType, SessionState};

/// Number of bottom lines to focus classification on.
const CLASSIFY_TAIL_LINES: usize = 50;

/// How recently output must have changed (in seconds) to be considered "working".
const WORKING_THRESHOLD_SECS: f64 = 5.0;

/// Minimum seconds of no output change before transitioning from Working → Idle.
pub const WORKING_IDLE_COOLOFF_SECS: f64 = 300.0;

// ---------------------------------------------------------------------------
// Regex patterns (compiled once)
// ---------------------------------------------------------------------------

// Hard signals: NeedsInput (permission)
static PERMISSION_PROCEED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Do you want to proceed\?").unwrap());
static PERMISSION_CREATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Do you want to create\b").unwrap());
static PERMISSION_APPROVAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"This command requires approval").unwrap());
static PERMISSION_FOOTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Esc to cancel\s*·\s*Tab to amend").unwrap());

// Hard signals: NeedsInput (AskUserQuestion)
static ASK_QUESTION_MARKER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"☐\s+\S").unwrap());
static ASK_QUESTION_FOOTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Enter to select\s*·\s*↑/↓ to navigate").unwrap());

// Hard signals: Errored
static ERROR_SHELL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?im)^error:\s").unwrap());
static ERROR_TOOL_OUTPUT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"⎿\s+Error:\s").unwrap());
static ERROR_EXIT_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Error: Exit code \d+").unwrap());

// Plan mode signals
static PLAN_MODE_TRANSITION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Entered plan mode").unwrap());
static PLAN_MODE_SPINNER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*✻\s").unwrap());
static PLAN_MODE_STATUS_BAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"⏸\s*plan mode on").unwrap());

// Recognizable Claude Code output markers
static CLAUDE_MARKERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"❯|▐▛███▜▌|⏺").unwrap());

// Claude TUI header (to detect session recovery after error)
static CLAUDE_TUI_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"▐▛███▜▌").unwrap());

// ---------------------------------------------------------------------------
// Classifier input/output
// ---------------------------------------------------------------------------

/// Everything the classifier needs to decide a session's state.
pub struct ClassifierInput<'a> {
    pub normalized_output: &'a str,
    pub output_hash: &'a str,
    pub previous_hash: Option<&'a str>,
    pub previous_state: Option<&'a SessionState>,
    pub seconds_since_last_change: f64,
    pub consecutive_idle_count: u32,
}

pub struct ClassifierResult {
    pub state: SessionState,
    pub needs_input_type: Option<NeedsInputType>,
    pub is_in_plan_mode: Option<bool>,
    /// True when the classifier held Working state due to debounce,
    /// even though the natural classification would have been Idle/Unknown.
    pub debounce_applied: bool,
}

// ---------------------------------------------------------------------------
// Signal detectors
// ---------------------------------------------------------------------------

fn get_tail(output: &str, lines: usize) -> &str {
    let line_positions: Vec<usize> = output.match_indices('\n').map(|(i, _)| i).collect();
    if line_positions.len() < lines {
        return output;
    }
    let start = line_positions[line_positions.len() - lines] + 1;
    &output[start..]
}

fn detect_needs_input(tail: &str) -> Option<NeedsInputType> {
    // Check AskUserQuestion first (more specific)
    if ASK_QUESTION_MARKER.is_match(tail) || ASK_QUESTION_FOOTER.is_match(tail) {
        return Some(NeedsInputType::Question);
    }

    // Check permission prompts
    if PERMISSION_PROCEED.is_match(tail)
        || PERMISSION_CREATE.is_match(tail)
        || PERMISSION_APPROVAL.is_match(tail)
        || PERMISSION_FOOTER.is_match(tail)
    {
        return Some(NeedsInputType::Permission);
    }

    None
}

fn detect_errored(tail: &str, full_output: &str) -> bool {
    // Shell-level errors: check full output
    if let Some(error_idx) = ERROR_SHELL.find(full_output) {
        let after_error = &full_output[error_idx.start()..];
        // If there's a Claude TUI header after the error (skip first 20 chars),
        // Claude started a new session — not errored
        if after_error.len() > 20 && CLAUDE_TUI_HEADER.is_match(&after_error[20..]) {
            return false;
        }
        return true;
    }

    // Tool-level errors in the tail
    if ERROR_TOOL_OUTPUT.is_match(tail) || ERROR_EXIT_CODE.is_match(tail) {
        return true;
    }

    false
}

fn detect_plan_mode(tail: &str, full_output: &str) -> Option<bool> {
    // Hard signals from main pane output (preferred)
    if PLAN_MODE_SPINNER.is_match(tail) {
        return Some(true);
    }
    if PLAN_MODE_TRANSITION.is_match(full_output) {
        return Some(true);
    }

    // Soft signal from status bar (fallback)
    if PLAN_MODE_STATUS_BAR.is_match(tail) {
        return Some(true);
    }

    // Can't determine — return None rather than guessing false
    None
}

fn detect_working(input: &ClassifierInput) -> bool {
    let previous_hash = match input.previous_hash {
        Some(h) => h,
        None => return false, // First capture — can't determine delta
    };

    if input.output_hash == previous_hash {
        return false;
    }

    // Output changed — check if it changed recently enough
    input.seconds_since_last_change <= WORKING_THRESHOLD_SECS
}

fn is_output_recognizable(output: &str) -> bool {
    CLAUDE_MARKERS.is_match(output)
}

// ---------------------------------------------------------------------------
// Main classifier
// ---------------------------------------------------------------------------

/// Classify a session's state from its pane output and temporal context.
///
/// Priority: NeedsInput > Errored > Working > Idle (with debounce) > Unknown
pub fn classify(input: &ClassifierInput) -> ClassifierResult {
    let tail = get_tail(input.normalized_output, CLASSIFY_TAIL_LINES);
    let full_output = input.normalized_output;

    // Evaluate all signals independently
    let needs_input_type = detect_needs_input(tail);
    let errored = detect_errored(tail, full_output);
    let working = detect_working(input);
    let is_in_plan_mode = detect_plan_mode(tail, full_output);

    // Resolve by priority
    let mut debounce_applied = false;
    let state = if needs_input_type.is_some() {
        SessionState::NeedsInput
    } else if errored {
        SessionState::Errored
    } else if working {
        SessionState::Working
    } else if matches!(input.previous_state, Some(SessionState::Working))
        && input.seconds_since_last_change < WORKING_IDLE_COOLOFF_SECS
    {
        // Cool-off: hold Working state until output has been unchanged for 5 minutes
        debounce_applied = true;
        SessionState::Working
    } else if is_output_recognizable(full_output) {
        SessionState::Idle
    } else {
        SessionState::Unknown
    };

    ClassifierResult {
        state: state.clone(),
        needs_input_type: if matches!(state, SessionState::NeedsInput) {
            needs_input_type
        } else {
            None
        },
        is_in_plan_mode,
        debounce_applied,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(output: &str) -> ClassifierInput<'_> {
        ClassifierInput {
            normalized_output: output,
            output_hash: "current-hash",
            previous_hash: None,
            previous_state: None,
            seconds_since_last_change: 0.0,
            consecutive_idle_count: 0,
        }
    }

    // -- get_tail --

    #[test]
    fn test_get_tail_short_output() {
        let output = "line1\nline2\nline3";
        assert_eq!(get_tail(output, 50), output);
    }

    #[test]
    fn test_get_tail_extracts_last_n_lines() {
        let output = "line1\nline2\nline3\nline4\nline5";
        let tail = get_tail(output, 2);
        assert_eq!(tail, "line4\nline5");
    }

    // -- detect_needs_input --

    #[test]
    fn test_permission_proceed() {
        assert!(matches!(
            detect_needs_input("Do you want to proceed?"),
            Some(NeedsInputType::Permission)
        ));
    }

    #[test]
    fn test_permission_create() {
        assert!(matches!(
            detect_needs_input("Do you want to create src/main.rs?"),
            Some(NeedsInputType::Permission)
        ));
    }

    #[test]
    fn test_permission_approval() {
        assert!(matches!(
            detect_needs_input("This command requires approval"),
            Some(NeedsInputType::Permission)
        ));
    }

    #[test]
    fn test_permission_footer() {
        assert!(matches!(
            detect_needs_input("Esc to cancel · Tab to amend"),
            Some(NeedsInputType::Permission)
        ));
    }

    #[test]
    fn test_ask_question_marker() {
        assert!(matches!(
            detect_needs_input("☐ Scope\nWhat should I refactor?"),
            Some(NeedsInputType::Question)
        ));
    }

    #[test]
    fn test_ask_question_footer() {
        assert!(matches!(
            detect_needs_input("Enter to select · ↑/↓ to navigate · Esc to cancel"),
            Some(NeedsInputType::Question)
        ));
    }

    #[test]
    fn test_no_input_needed() {
        assert!(detect_needs_input("❯ just a normal prompt").is_none());
    }

    // -- detect_errored --

    #[test]
    fn test_shell_error() {
        assert!(detect_errored("", "error: unknown option '--bad'\n(Did you mean ...?)"));
    }

    #[test]
    fn test_shell_error_recovered() {
        // Error followed by a new Claude TUI session — not errored
        let full = "error: bad flag\n01234567890123456789▐▛███▜▌ some header";
        assert!(!detect_errored("", full));
    }

    #[test]
    fn test_tool_error_in_tail() {
        assert!(detect_errored("⎿  Error: something failed", ""));
    }

    #[test]
    fn test_exit_code_error() {
        assert!(detect_errored("Error: Exit code 1", ""));
    }

    #[test]
    fn test_no_error() {
        assert!(!detect_errored("all good ❯", "all good ❯"));
    }

    // -- detect_plan_mode --

    #[test]
    fn test_plan_mode_spinner() {
        assert_eq!(detect_plan_mode("  ✻ Reading files", ""), Some(true));
    }

    #[test]
    fn test_plan_mode_transition() {
        assert_eq!(
            detect_plan_mode("", "Some output\nEntered plan mode\nMore output"),
            Some(true)
        );
    }

    #[test]
    fn test_plan_mode_status_bar() {
        assert_eq!(detect_plan_mode("⏸ plan mode on", ""), Some(true));
    }

    #[test]
    fn test_plan_mode_unknown() {
        assert_eq!(detect_plan_mode("❯ normal prompt", "❯ normal prompt"), None);
    }

    // -- detect_working --

    #[test]
    fn test_working_output_changed_recently() {
        let input = ClassifierInput {
            normalized_output: "",
            output_hash: "new-hash",
            previous_hash: Some("old-hash"),
            previous_state: None,
            seconds_since_last_change: 1.0,
            consecutive_idle_count: 0,
        };
        assert!(detect_working(&input));
    }

    #[test]
    fn test_not_working_same_hash() {
        let input = ClassifierInput {
            normalized_output: "",
            output_hash: "same",
            previous_hash: Some("same"),
            previous_state: None,
            seconds_since_last_change: 1.0,
            consecutive_idle_count: 0,
        };
        assert!(!detect_working(&input));
    }

    #[test]
    fn test_not_working_stale_change() {
        let input = ClassifierInput {
            normalized_output: "",
            output_hash: "new",
            previous_hash: Some("old"),
            previous_state: None,
            seconds_since_last_change: 10.0,
            consecutive_idle_count: 0,
        };
        assert!(!detect_working(&input));
    }

    #[test]
    fn test_not_working_first_capture() {
        let input = ClassifierInput {
            normalized_output: "",
            output_hash: "hash",
            previous_hash: None,
            previous_state: None,
            seconds_since_last_change: 0.0,
            consecutive_idle_count: 0,
        };
        assert!(!detect_working(&input));
    }

    // -- classify (integration) --

    #[test]
    fn test_classify_needs_input_highest_priority() {
        // Even with working signal, NeedsInput wins
        let input = ClassifierInput {
            normalized_output: "Do you want to proceed?\n❯ 1. Yes",
            output_hash: "new",
            previous_hash: Some("old"),
            previous_state: Some(&SessionState::Working),
            seconds_since_last_change: 1.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::NeedsInput));
        assert!(matches!(result.needs_input_type, Some(NeedsInputType::Permission)));
    }

    #[test]
    fn test_classify_errored_over_working() {
        let input = ClassifierInput {
            normalized_output: "error: bad flag",
            output_hash: "new",
            previous_hash: Some("old"),
            previous_state: None,
            seconds_since_last_change: 1.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::Errored));
    }

    #[test]
    fn test_classify_working() {
        let input = ClassifierInput {
            normalized_output: "⏺ Writing code...",
            output_hash: "new",
            previous_hash: Some("old"),
            previous_state: None,
            seconds_since_last_change: 2.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::Working));
    }

    #[test]
    fn test_classify_idle() {
        let input = ClassifierInput {
            normalized_output: "❯ ",
            output_hash: "same",
            previous_hash: Some("same"),
            previous_state: None,
            seconds_since_last_change: 30.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::Idle));
    }

    #[test]
    fn test_classify_unknown() {
        let input = ClassifierInput {
            normalized_output: "some random text with no claude markers",
            output_hash: "same",
            previous_hash: Some("same"),
            previous_state: None,
            seconds_since_last_change: 30.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::Unknown));
    }

    #[test]
    fn test_classify_cooloff_holds_working() {
        // Output stopped changing 60s ago — still within 5min cool-off
        let input = ClassifierInput {
            normalized_output: "⏺ some output\n❯ ",
            output_hash: "same",
            previous_hash: Some("same"),
            previous_state: Some(&SessionState::Working),
            seconds_since_last_change: 60.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::Working));
        assert!(result.debounce_applied);
    }

    #[test]
    fn test_classify_cooloff_releases_to_idle() {
        // Output stopped changing 301s ago — past 5min cool-off
        let input = ClassifierInput {
            normalized_output: "⏺ some output\n❯ ",
            output_hash: "same",
            previous_hash: Some("same"),
            previous_state: Some(&SessionState::Working),
            seconds_since_last_change: 301.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::Idle));
        assert!(!result.debounce_applied);
    }

    #[test]
    fn test_classify_plan_mode_orthogonal() {
        let input = ClassifierInput {
            normalized_output: "Entered plan mode\n✻ Reading files\nDo you want to proceed?",
            output_hash: "hash",
            previous_hash: None,
            previous_state: None,
            seconds_since_last_change: 0.0,
            consecutive_idle_count: 0,
        };
        let result = classify(&input);
        assert!(matches!(result.state, SessionState::NeedsInput));
        assert_eq!(result.is_in_plan_mode, Some(true));
    }
}
