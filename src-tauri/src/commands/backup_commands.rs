use std::path::PathBuf;
use tauri::State;

use crate::database::AppState;
use crate::services::backup_service;
use crate::AppError;

#[tauri::command]
pub fn export_database(state: State<AppState>, dest_path: String) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    backup_service::export_database(&conn, &state.db_path, &PathBuf::from(dest_path))
}

#[tauri::command]
pub fn import_database(state: State<AppState>, src_path: String) -> Result<(), AppError> {
    // The connection held in `state` must be released before the file is
    // overwritten on disk, otherwise the copy could race with SQLite's own
    // file handles. Locking and dropping the guard first keeps this safe.
    drop(state.db.lock().unwrap());
    backup_service::import_database(&state.db_path, &PathBuf::from(src_path))
}
