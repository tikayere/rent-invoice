use chrono::Utc;
use tauri::State;

use crate::database::AppState;
use crate::models::{Tenant, TenantInput};
use crate::AppError;

fn validate(input: &TenantInput) -> Result<(), AppError> {
    if input.first_name.trim().is_empty() || input.last_name.trim().is_empty() {
        return Err(AppError::Validation("Le nom et le prenom sont requis".into()));
    }
    if input.phone.trim().is_empty() {
        return Err(AppError::Validation("Le telephone est requis".into()));
    }
    if input.address.trim().is_empty() {
        return Err(AppError::Validation("L'adresse est requise".into()));
    }
    Ok(())
}

#[tauri::command]
pub fn list_tenants(state: State<AppState>, search: Option<String>) -> Result<Vec<Tenant>, AppError> {
    let conn = state.db.lock().unwrap();
    let like = search
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("%{}%", s.trim().to_lowercase()));

    let sql = "SELECT t.*, (SELECT COUNT(*) FROM invoices i WHERE i.tenant_id = t.id) AS invoice_count
               FROM tenants t
               WHERE (?1 IS NULL
                   OR lower(t.first_name) LIKE ?1
                   OR lower(t.last_name) LIKE ?1
                   OR lower(t.phone) LIKE ?1
                   OR lower(t.email) LIKE ?1)
               ORDER BY t.last_name, t.first_name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([&like], |row| Tenant::from_row(row))?;
    let tenants = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(tenants)
}

#[tauri::command]
pub fn get_tenant(state: State<AppState>, id: i64) -> Result<Tenant, AppError> {
    let conn = state.db.lock().unwrap();
    let tenant = conn.query_row(
        "SELECT t.*, (SELECT COUNT(*) FROM invoices i WHERE i.tenant_id = t.id) AS invoice_count
         FROM tenants t WHERE t.id = ?1",
        [id],
        |row| Tenant::from_row(row),
    )?;
    Ok(tenant)
}

#[tauri::command]
pub fn create_tenant(state: State<AppState>, input: TenantInput) -> Result<Tenant, AppError> {
    validate(&input)?;
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO tenants (first_name, last_name, phone, email, address, id_number, profession, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        rusqlite::params![
            input.first_name,
            input.last_name,
            input.phone,
            input.email,
            input.address,
            input.id_number,
            input.profession,
            input.notes,
            now,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let tenant = conn.query_row(
        "SELECT t.*, 0 AS invoice_count FROM tenants t WHERE t.id = ?1",
        [id],
        |row| Tenant::from_row(row),
    )?;
    Ok(tenant)
}

#[tauri::command]
pub fn update_tenant(state: State<AppState>, id: i64, input: TenantInput) -> Result<Tenant, AppError> {
    validate(&input)?;
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE tenants SET first_name = ?1, last_name = ?2, phone = ?3, email = ?4, address = ?5,
            id_number = ?6, profession = ?7, notes = ?8, updated_at = ?9
         WHERE id = ?10",
        rusqlite::params![
            input.first_name,
            input.last_name,
            input.phone,
            input.email,
            input.address,
            input.id_number,
            input.profession,
            input.notes,
            now,
            id,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound("Locataire introuvable".into()));
    }
    let tenant = conn.query_row(
        "SELECT t.*, (SELECT COUNT(*) FROM invoices i WHERE i.tenant_id = t.id) AS invoice_count
         FROM tenants t WHERE t.id = ?1",
        [id],
        |row| Tenant::from_row(row),
    )?;
    Ok(tenant)
}

#[tauri::command]
pub fn delete_tenant(state: State<AppState>, id: i64) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    let invoice_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM invoices WHERE tenant_id = ?1", [id], |r| r.get(0))?;
    if invoice_count > 0 {
        return Err(AppError::Validation(
            "Impossible de supprimer ce locataire : des factures lui sont associees".into(),
        ));
    }
    let affected = conn.execute("DELETE FROM tenants WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::NotFound("Locataire introuvable".into()));
    }
    Ok(())
}
