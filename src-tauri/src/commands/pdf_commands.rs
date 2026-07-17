use tauri::State;

use crate::database::AppState;
use crate::models::{Invoice, Settings, Tenant};
use crate::pdf::invoice_pdf::{render_invoice_pdf, InvoicePdfData};
use crate::AppError;

fn load_invoice_bundle(state: &AppState, invoice_id: i64) -> Result<(Settings, Tenant, Invoice), AppError> {
    let conn = state.db.lock().unwrap();

    let settings = conn.query_row("SELECT * FROM settings WHERE id = 1", [], |row| Settings::from_row(row))?;

    let invoice = conn.query_row(
        "SELECT i.*, (t.first_name || ' ' || t.last_name) AS tenant_name
         FROM invoices i JOIN tenants t ON t.id = i.tenant_id WHERE i.id = ?1",
        [invoice_id],
        |row| Invoice::from_row(row),
    )?;

    let tenant = conn.query_row(
        "SELECT t.*, (SELECT COUNT(*) FROM invoices i WHERE i.tenant_id = t.id) AS invoice_count
         FROM tenants t WHERE t.id = ?1",
        [invoice.tenant_id],
        |row| Tenant::from_row(row),
    )?;

    Ok((settings, tenant, invoice))
}

#[tauri::command]
pub fn generate_invoice_pdf(state: State<AppState>, invoice_id: i64, dest_path: String) -> Result<(), AppError> {
    let (settings, tenant, invoice) = load_invoice_bundle(&state, invoice_id)?;
    let data = InvoicePdfData { settings: &settings, tenant: &tenant, invoice: &invoice };
    let bytes = render_invoice_pdf(&data).map_err(AppError::Pdf)?;
    std::fs::write(dest_path, bytes)?;
    Ok(())
}

#[tauri::command]
pub fn preview_invoice_pdf(state: State<AppState>, invoice_id: i64) -> Result<Vec<u8>, AppError> {
    let (settings, tenant, invoice) = load_invoice_bundle(&state, invoice_id)?;
    let data = InvoicePdfData { settings: &settings, tenant: &tenant, invoice: &invoice };
    let bytes = render_invoice_pdf(&data).map_err(AppError::Pdf)?;
    Ok(bytes)
}
