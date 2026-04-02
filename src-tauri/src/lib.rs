mod commands;
mod git;
mod preferences;
mod session;
mod store;
mod tmux;

use std::sync::Mutex;

use tauri::Manager;

use preferences::{ConfigDir, Preferences};
use store::SessionStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(SessionStore::new()))
        .setup(|app| {
            let config_dir = app.path().app_config_dir()
                .map_err(|e: tauri::Error| e.to_string())?;
            let prefs = Preferences::load(&config_dir);
            app.manage(Mutex::new(prefs));
            app.manage(ConfigDir(config_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_sessions,
            commands::focus_session,
            commands::create_session,
            commands::kill_session,
            commands::rename_session,
            commands::get_preferences,
            commands::set_preferences,
            commands::resolve_bootstrap_command,
            commands::is_git_repo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
