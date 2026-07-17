use chrono::Utc;
use tauri::State;

use crate::database::AppState;
use crate::models::{Settings, SettingsInput};
use crate::AppError;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, AppError> {
    let conn = state.db.lock().unwrap();
    let settings = conn.query_row("SELECT * FROM settings WHERE id = 1", [], |row| {
        Settings::from_row(row)
    })?;
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, input: SettingsInput) -> Result<Settings, AppError> {
    if input.full_name.trim().is_empty() {
        return Err(AppError::Validation("Le nom complet est requis".into()));
    }
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE settings SET
            full_name = ?1, company_name = ?2, address = ?3, phone = ?4, email = ?5,
            city = ?6, country = ?7, currency = ?8, logo_path = ?9, signature_path = ?10,
            tax_number = ?11, iban = ?12, additional_info = ?13, invoice_prefix = ?14,
            date_format = ?15, language = ?16, theme = ?17, invoice_template = ?18, updated_at = ?19
         WHERE id = 1",
        rusqlite::params![
            input.full_name,
            input.company_name,
            input.address,
            input.phone,
            input.email,
            input.city,
            input.country,
            input.currency,
            input.logo_path,
            input.signature_path,
            input.tax_number,
            input.iban,
            input.additional_info,
            input.invoice_prefix,
            input.date_format,
            input.language,
            input.theme,
            input.invoice_template,
            Utc::now().to_rfc3339(),
        ],
    )?;

    let settings = conn.query_row("SELECT * FROM settings WHERE id = 1", [], |row| {
        Settings::from_row(row)
    })?;
    Ok(settings)
}
