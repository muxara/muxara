mod commands;
mod session;
mod store;
mod tmux;

use std::sync::Mutex;

use store::SessionStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(SessionStore::new()))
        .invoke_handler(tauri::generate_handler![commands::get_sessions, commands::focus_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
