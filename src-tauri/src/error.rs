use serde::Serialize;
use thiserror::Error;

/// Central error type for the whole backend. Every Tauri command returns
/// `Result<T, AppError>`; Tauri serializes the `Err` variant back to the
/// frontend as a plain string via the `Serialize` implementation below, so
/// the React side can display `String(error)` directly.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Erreur de base de donnees : {0}")]
    Database(String),
    #[error("Erreur d'entree/sortie : {0}")]
    Io(String),
    #[error("Erreur de generation PDF : {0}")]
    Pdf(String),
    #[error("Ressource introuvable : {0}")]
    NotFound(String),
    #[error("Donnees invalides : {0}")]
    Validation(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("Element introuvable".into()),
            other => AppError::Database(other.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
