use crate::session::{mock_sessions, Session};

#[tauri::command]
pub fn get_sessions() -> Vec<Session> {
    mock_sessions()
}
