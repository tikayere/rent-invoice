use chrono::{Datelike, Utc};
use tauri::State;

use crate::database::AppState;
use crate::models::{DashboardStats, Invoice};
use crate::AppError;

#[tauri::command]
pub fn get_dashboard_stats(state: State<AppState>) -> Result<DashboardStats, AppError> {
    let conn = state.db.lock().unwrap();

    let total_invoices: i64 = conn.query_row("SELECT COUNT(*) FROM invoices", [], |r| r.get(0))?;
    let total_tenants: i64 = conn.query_row("SELECT COUNT(*) FROM tenants", [], |r| r.get(0))?;
    let total_outstanding: f64 =
        conn.query_row("SELECT COALESCE(SUM(balance_due), 0) FROM invoices", [], |r| r.get(0))?;

    let now = Utc::now();
    let total_collected_this_month: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_paid), 0) FROM invoices WHERE billing_month = ?1 AND billing_year = ?2",
        [now.month() as i64, now.year() as i64],
        |r| r.get(0),
    )?;

    let last_invoice: Option<Invoice> = conn
        .query_row(
            "SELECT i.*, (t.first_name || ' ' || t.last_name) AS tenant_name
             FROM invoices i JOIN tenants t ON t.id = i.tenant_id
             ORDER BY i.created_at DESC LIMIT 1",
            [],
            |row| Invoice::from_row(row),
        )
        .ok();

    Ok(DashboardStats {
        total_invoices,
        total_tenants,
        last_invoice,
        total_outstanding,
        total_collected_this_month,
    })
}
