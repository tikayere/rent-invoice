use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::AppError;

/// Copies the live SQLite database file to `dest_path`. The source connection
/// uses WAL mode, so we checkpoint it first to make sure everything pending
/// in the write-ahead log is flushed into the main database file before copying.
pub fn export_database(
    conn: &rusqlite::Connection,
    db_path: &Path,
    dest_path: &Path,
) -> Result<(), AppError> {
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")
        .map_err(|e| AppError::Database(e.to_string()))?;
    fs::copy(db_path, dest_path).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}

/// Restores a previously exported database file over the current one.
/// A timestamped safety copy of the current database is made first so the
/// operation can be manually reversed if the chosen backup turns out to be wrong.
/// The caller is responsible for restarting the app afterwards so a fresh
/// connection is opened against the restored file.
pub fn import_database(db_path: &Path, src_path: &Path) -> Result<(), AppError> {
    if !src_path.exists() {
        return Err(AppError::NotFound("Fichier de sauvegarde introuvable".into()));
    }
    if let Some(parent) = db_path.parent() {
        let safety_name = format!(
            "rent_invoices.pre-restore-{}.db",
            Utc::now().format("%Y%m%d%H%M%S")
        );
        let safety_path = parent.join(safety_name);
        if db_path.exists() {
            let _ = fs::copy(db_path, safety_path);
        }
    }
    fs::copy(src_path, db_path).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}
