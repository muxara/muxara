use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Newtype wrapper so Tauri can manage the config directory path
/// without ambiguity against other PathBuf-typed managed state.
pub struct ConfigDir(pub std::path::PathBuf);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_worktree: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub cooloff_minutes: f64,
    pub poll_interval_secs: f64,
    pub output_lines: usize,
    pub show_idle_output: bool,
    pub context_zone_max_height: u32,
    pub grid_columns: u32,
    pub scroll_pause_secs: f64,
    pub bootstrap_command: String,
    #[serde(default = "default_true")]
    pub use_worktree: bool,
    #[serde(default)]
    pub project_overrides: HashMap<String, ProjectOverrides>,
}

fn default_true() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            cooloff_minutes: 5.0,
            poll_interval_secs: 1.5,
            output_lines: 30,
            show_idle_output: false,
            context_zone_max_height: 192,
            grid_columns: 2,
            scroll_pause_secs: 5.0,
            bootstrap_command: "claude".to_string(),
            use_worktree: true,
            project_overrides: HashMap::new(),
        }
    }
}

impl Preferences {
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=60.0).contains(&self.cooloff_minutes) {
            return Err("Cool-off period must be between 0 and 60 minutes".to_string());
        }
        if !(0.5..=30.0).contains(&self.poll_interval_secs) {
            return Err("Poll interval must be between 0.5 and 30 seconds".to_string());
        }
        if !(1..=200).contains(&self.output_lines) {
            return Err("Output lines must be between 1 and 200".to_string());
        }
        if !(48..=800).contains(&self.context_zone_max_height) {
            return Err("Context zone height must be between 48 and 800 pixels".to_string());
        }
        if !(1..=6).contains(&self.grid_columns) {
            return Err("Grid columns must be between 1 and 6".to_string());
        }
        if !(0.0..=60.0).contains(&self.scroll_pause_secs) {
            return Err("Scroll pause must be between 0 and 60 seconds".to_string());
        }
        if self.bootstrap_command.trim().is_empty() {
            return Err("Bootstrap command must not be empty".to_string());
        }
        if self.bootstrap_command.len() > 500 {
            return Err("Bootstrap command must be 500 characters or fewer".to_string());
        }
        for (path, overrides) in &self.project_overrides {
            if path.trim().is_empty() {
                return Err("Project path must not be empty".to_string());
            }
            if let Some(ref cmd) = overrides.bootstrap_command {
                if cmd.trim().is_empty() {
                    return Err(format!(
                        "Bootstrap command for project '{}' must not be empty",
                        path
                    ));
                }
                if cmd.len() > 500 {
                    return Err(format!(
                        "Bootstrap command for project '{}' must be 500 characters or fewer",
                        path
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn effective_use_worktree(&self, working_dir: &str) -> bool {
        self.project_overrides
            .get(working_dir)
            .and_then(|o| o.use_worktree)
            .unwrap_or(self.use_worktree)
    }

    pub fn effective_bootstrap_command(&self, working_dir: &str) -> &str {
        self.project_overrides
            .get(working_dir)
            .and_then(|o| o.bootstrap_command.as_deref())
            .unwrap_or(&self.bootstrap_command)
    }

    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join("preferences.json");
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, config_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        let path = config_dir.join("preferences.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize preferences: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("Failed to write preferences file: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_values() {
        let prefs = Preferences::default();
        assert_eq!(prefs.cooloff_minutes, 5.0);
        assert_eq!(prefs.poll_interval_secs, 1.5);
        assert_eq!(prefs.output_lines, 30);
        assert!(!prefs.show_idle_output);
        assert_eq!(prefs.context_zone_max_height, 192);
        assert_eq!(prefs.grid_columns, 2);
        assert_eq!(prefs.scroll_pause_secs, 5.0);
        assert_eq!(prefs.bootstrap_command, "claude");
        assert!(prefs.project_overrides.is_empty());
    }

    #[test]
    fn test_validate_accepts_defaults() {
        assert!(Preferences::default().validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_out_of_range() {
        let mut prefs = Preferences::default();

        prefs.poll_interval_secs = 0.1;
        assert!(prefs.validate().is_err());
        prefs.poll_interval_secs = 1.5;

        prefs.cooloff_minutes = -1.0;
        assert!(prefs.validate().is_err());
        prefs.cooloff_minutes = 5.0;

        prefs.output_lines = 0;
        assert!(prefs.validate().is_err());
        prefs.output_lines = 20;

        prefs.context_zone_max_height = 10;
        assert!(prefs.validate().is_err());
        prefs.context_zone_max_height = 192;

        prefs.grid_columns = 0;
        assert!(prefs.validate().is_err());
        prefs.grid_columns = 2;

        prefs.scroll_pause_secs = -1.0;
        assert!(prefs.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_empty_bootstrap_command() {
        let mut prefs = Preferences::default();
        prefs.bootstrap_command = "  ".to_string();
        assert!(prefs.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_long_bootstrap_command() {
        let mut prefs = Preferences::default();
        prefs.bootstrap_command = "x".repeat(501);
        assert!(prefs.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_empty_project_override_command() {
        let mut prefs = Preferences::default();
        prefs.project_overrides.insert(
            "/some/path".to_string(),
            ProjectOverrides {
                bootstrap_command: Some("".to_string()),
                ..Default::default()
            },
        );
        assert!(prefs.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_empty_project_path() {
        let mut prefs = Preferences::default();
        prefs.project_overrides.insert(
            "  ".to_string(),
            ProjectOverrides {
                bootstrap_command: Some("claude".to_string()),
                ..Default::default()
            },
        );
        assert!(prefs.validate().is_err());
    }

    #[test]
    fn test_effective_bootstrap_command_global() {
        let prefs = Preferences {
            bootstrap_command: "claude --verbose".to_string(),
            ..Preferences::default()
        };
        assert_eq!(
            prefs.effective_bootstrap_command("/some/dir"),
            "claude --verbose"
        );
    }

    #[test]
    fn test_effective_bootstrap_command_project_override() {
        let mut prefs = Preferences {
            bootstrap_command: "claude --verbose".to_string(),
            ..Preferences::default()
        };
        prefs.project_overrides.insert(
            "/projects/matrix".to_string(),
            ProjectOverrides {
                bootstrap_command: Some("claude --plugin ../morpheus".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            prefs.effective_bootstrap_command("/projects/matrix"),
            "claude --plugin ../morpheus"
        );
        assert_eq!(
            prefs.effective_bootstrap_command("/projects/other"),
            "claude --verbose"
        );
    }

    #[test]
    fn test_effective_bootstrap_command_project_no_override() {
        let mut prefs = Preferences {
            bootstrap_command: "claude --verbose".to_string(),
            ..Preferences::default()
        };
        prefs.project_overrides.insert(
            "/projects/matrix".to_string(),
            ProjectOverrides {
                bootstrap_command: None,
                ..Default::default()
            },
        );
        assert_eq!(
            prefs.effective_bootstrap_command("/projects/matrix"),
            "claude --verbose"
        );
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("muxara_test_prefs_roundtrip");
        let _ = fs::remove_dir_all(&dir);

        let mut prefs = Preferences::default();
        prefs.grid_columns = 3;
        prefs.poll_interval_secs = 2.0;
        prefs.save(&dir).unwrap();

        let loaded = Preferences::load(&dir);
        assert_eq!(prefs, loaded);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_load_roundtrip_with_overrides() {
        let dir = std::env::temp_dir().join("muxara_test_prefs_overrides");
        let _ = fs::remove_dir_all(&dir);

        let mut prefs = Preferences::default();
        prefs.bootstrap_command = "claude --model opus".to_string();
        prefs.project_overrides.insert(
            "/projects/matrix".to_string(),
            ProjectOverrides {
                bootstrap_command: Some("claude --plugin ../morpheus".to_string()),
                ..Default::default()
            },
        );
        prefs.save(&dir).unwrap();

        let loaded = Preferences::load(&dir);
        assert_eq!(prefs, loaded);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let dir = std::env::temp_dir().join("muxara_test_prefs_missing");
        let _ = fs::remove_dir_all(&dir);
        let loaded = Preferences::load(&dir);
        assert_eq!(loaded, Preferences::default());
    }

    #[test]
    fn test_default_use_worktree_is_true() {
        let prefs = Preferences::default();
        assert!(prefs.use_worktree);
    }

    #[test]
    fn test_effective_use_worktree_global() {
        let prefs = Preferences::default();
        assert!(prefs.effective_use_worktree("/some/dir"));

        let prefs = Preferences {
            use_worktree: false,
            ..Preferences::default()
        };
        assert!(!prefs.effective_use_worktree("/some/dir"));
    }

    #[test]
    fn test_effective_use_worktree_project_override() {
        let mut prefs = Preferences::default();
        prefs.project_overrides.insert(
            "/projects/legacy".to_string(),
            ProjectOverrides {
                use_worktree: Some(false),
                ..Default::default()
            },
        );
        assert!(!prefs.effective_use_worktree("/projects/legacy"));
        assert!(prefs.effective_use_worktree("/projects/other"));
    }

    #[test]
    fn test_effective_use_worktree_project_inherits_global() {
        let mut prefs = Preferences {
            use_worktree: false,
            ..Preferences::default()
        };
        prefs.project_overrides.insert(
            "/projects/matrix".to_string(),
            ProjectOverrides {
                bootstrap_command: Some("claude --plugin ../morpheus".to_string()),
                ..Default::default()
            },
        );
        // No use_worktree override, so inherits global false
        assert!(!prefs.effective_use_worktree("/projects/matrix"));
    }

    #[test]
    fn test_load_corrupt_file_returns_default() {
        let dir = std::env::temp_dir().join("muxara_test_prefs_corrupt");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("preferences.json"), "not json at all {{{").unwrap();

        let loaded = Preferences::load(&dir);
        assert_eq!(loaded, Preferences::default());

        let _ = fs::remove_dir_all(&dir);
    }
}
