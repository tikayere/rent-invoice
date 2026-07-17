use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Shared application state: a single serialized SQLite connection plus the
/// on-disk locations the app uses for the database file and generated backups.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(app_data_dir: &Path) -> rusqlite::Result<Self> {
        std::fs::create_dir_all(app_data_dir).expect("failed to create app data directory");
        let db_path = app_data_dir.join("rent_invoices.db");
        let conn = Connection::open(&db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        super::migrations::run_migrations(&conn)?;
        Ok(Self {
            db: Mutex::new(conn),
            db_path,
        })
    }
}
