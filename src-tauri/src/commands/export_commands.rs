use rusqlite::ToSql;
use std::path::PathBuf;
use tauri::State;

use crate::database::AppState;
use crate::models::{Invoice, InvoiceFilters};
use crate::services::export_service;
use crate::AppError;

#[tauri::command]
pub fn export_invoices_csv(
    state: State<AppState>,
    dest_path: String,
    filters: InvoiceFilters,
) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(search) = filters.search.as_ref().filter(|s| !s.trim().is_empty()) {
        conditions.push(
            "(lower(i.invoice_number) LIKE ?1 OR lower(t.first_name) LIKE ?1 OR lower(t.last_name) LIKE ?1 OR lower(i.property_address) LIKE ?1)"
                .to_string(),
        );
        params.push(Box::new(format!("%{}%", search.trim().to_lowercase())));
    }
    if let Some(status) = filters.status.as_ref().filter(|s| !s.is_empty()) {
        conditions.push(format!("i.status = ?{}", params.len() + 1));
        params.push(Box::new(status.clone()));
    }
    if let Some(year) = filters.year {
        conditions.push(format!("i.billing_year = ?{}", params.len() + 1));
        params.push(Box::new(year));
    }
    if let Some(month) = filters.month {
        conditions.push(format!("i.billing_month = ?{}", params.len() + 1));
        params.push(Box::new(month));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT i.*, (t.first_name || ' ' || t.last_name) AS tenant_name
         FROM invoices i JOIN tenants t ON t.id = i.tenant_id{}
         ORDER BY i.issue_date DESC",
        where_clause
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
        Invoice::from_row(row)
    })?;
    let invoices = rows.collect::<Result<Vec<_>, _>>()?;

    export_service::export_invoices_csv(&invoices, &PathBuf::from(dest_path))
}
