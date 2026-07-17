use chrono::{Datelike, Utc};
use rusqlite::ToSql;
use tauri::State;

use crate::database::AppState;
use crate::models::{Invoice, InvoiceFilters, InvoiceInput, PagedResult};
use crate::services::invoice_numbering::format_invoice_number;
use crate::AppError;

const INVOICE_SELECT: &str = "SELECT i.*, (t.first_name || ' ' || t.last_name) AS tenant_name
     FROM invoices i JOIN tenants t ON t.id = i.tenant_id";

fn validate(input: &InvoiceInput) -> Result<(), AppError> {
    if input.tenant_id <= 0 {
        return Err(AppError::Validation("Selectionnez un locataire".into()));
    }
    if input.property_address.trim().is_empty() {
        return Err(AppError::Validation("L'adresse du bien est requise".into()));
    }
    if !(1..=12).contains(&input.billing_month) {
        return Err(AppError::Validation("Mois de facturation invalide".into()));
    }
    if input.rent_amount < 0.0
        || input.water_charge < 0.0
        || input.electricity_charge < 0.0
        || input.other_charges < 0.0
        || input.discount < 0.0
        || input.amount_paid < 0.0
    {
        return Err(AppError::Validation("Les montants ne peuvent pas etre negatifs".into()));
    }
    Ok(())
}

#[tauri::command]
pub fn list_invoices(state: State<AppState>, filters: InvoiceFilters) -> Result<PagedResult<Invoice>, AppError> {
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

    let sort_col = match filters.sort_by.as_deref() {
        Some("total_amount") => "i.total_amount",
        Some("tenant_name") => "tenant_name",
        Some("invoice_number") => "i.invoice_number",
        _ => "i.issue_date",
    };
    let sort_dir = match filters.sort_dir.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM invoices i JOIN tenants t ON t.id = i.tenant_id{}",
        where_clause
    );
    let total: i64 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |r| r.get(0),
    )?;

    let per_page = filters.per_page.max(1);
    let page = filters.page.max(1);
    let offset = (page - 1) * per_page;

    let data_sql = format!(
        "{}{} ORDER BY {} {} LIMIT ?{} OFFSET ?{}",
        INVOICE_SELECT,
        where_clause,
        sort_col,
        sort_dir,
        params.len() + 1,
        params.len() + 2
    );
    params.push(Box::new(per_page));
    params.push(Box::new(offset));

    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
        Invoice::from_row(row)
    })?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;

    let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };

    Ok(PagedResult { items, total, page, per_page, total_pages })
}

#[tauri::command]
pub fn get_invoice(state: State<AppState>, id: i64) -> Result<Invoice, AppError> {
    let conn = state.db.lock().unwrap();
    let sql = format!("{} WHERE i.id = ?1", INVOICE_SELECT);
    let invoice = conn.query_row(&sql, [id], |row| Invoice::from_row(row))?;
    Ok(invoice)
}

#[tauri::command]
pub fn preview_next_invoice_number(state: State<AppState>) -> Result<String, AppError> {
    let conn = state.db.lock().unwrap();
    let (prefix, next_number): (String, i64) = conn.query_row(
        "SELECT invoice_prefix, next_invoice_number FROM settings WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let year = Utc::now().year();
    Ok(format_invoice_number(&prefix, year, next_number))
}

#[tauri::command]
pub fn create_invoice(state: State<AppState>, input: InvoiceInput) -> Result<Invoice, AppError> {
    validate(&input)?;
    let mut conn = state.db.lock().unwrap();
    let tx = conn.transaction()?;
    let now = Utc::now().to_rfc3339();

    let tenant_exists: i64 = tx.query_row("SELECT COUNT(*) FROM tenants WHERE id = ?1", [input.tenant_id], |r| r.get(0))?;
    if tenant_exists == 0 {
        return Err(AppError::NotFound("Locataire introuvable".into()));
    }

    let (prefix, next_number): (String, i64) = tx.query_row(
        "SELECT invoice_prefix, next_invoice_number FROM settings WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let issue_year = input
        .issue_date
        .get(0..4)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or_else(|| Utc::now().year());
    let invoice_number = format_invoice_number(&prefix, issue_year, next_number);

    let total = input.total();
    let balance_due = input.balance_due();
    let status = input.status();

    tx.execute(
        "INSERT INTO invoices (
            invoice_number, tenant_id, property_address, description, billing_month, billing_year,
            issue_date, due_date, rent_amount, water_charge, electricity_charge, other_charges,
            discount, total_amount, amount_paid, balance_due, payment_method, status, observations,
            created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?20)",
        rusqlite::params![
            invoice_number,
            input.tenant_id,
            input.property_address,
            input.description,
            input.billing_month,
            input.billing_year,
            input.issue_date,
            input.due_date,
            input.rent_amount,
            input.water_charge,
            input.electricity_charge,
            input.other_charges,
            input.discount,
            total,
            input.amount_paid,
            balance_due,
            input.payment_method,
            status,
            input.observations,
            now,
        ],
    )?;
    let invoice_id = tx.last_insert_rowid();

    insert_invoice_items(&tx, invoice_id, &input)?;
    if input.amount_paid > 0.0 {
        tx.execute(
            "INSERT INTO payments (invoice_id, amount, payment_method, paid_at, notes) VALUES (?1,?2,?3,?4,NULL)",
            rusqlite::params![invoice_id, input.amount_paid, input.payment_method, input.issue_date],
        )?;
    }

    tx.execute(
        "UPDATE settings SET next_invoice_number = ?1 WHERE id = 1",
        [next_number + 1],
    )?;

    tx.commit()?;

    let sql = format!("{} WHERE i.id = ?1", INVOICE_SELECT);
    let invoice = conn.query_row(&sql, [invoice_id], |row| Invoice::from_row(row))?;
    Ok(invoice)
}

#[tauri::command]
pub fn update_invoice(state: State<AppState>, id: i64, input: InvoiceInput) -> Result<Invoice, AppError> {
    validate(&input)?;
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();

    let total = input.total();
    let balance_due = input.balance_due();
    let status = input.status();

    let affected = conn.execute(
        "UPDATE invoices SET
            tenant_id = ?1, property_address = ?2, description = ?3, billing_month = ?4, billing_year = ?5,
            issue_date = ?6, due_date = ?7, rent_amount = ?8, water_charge = ?9, electricity_charge = ?10,
            other_charges = ?11, discount = ?12, total_amount = ?13, amount_paid = ?14, balance_due = ?15,
            payment_method = ?16, status = ?17, observations = ?18, updated_at = ?19
         WHERE id = ?20",
        rusqlite::params![
            input.tenant_id,
            input.property_address,
            input.description,
            input.billing_month,
            input.billing_year,
            input.issue_date,
            input.due_date,
            input.rent_amount,
            input.water_charge,
            input.electricity_charge,
            input.other_charges,
            input.discount,
            total,
            input.amount_paid,
            balance_due,
            input.payment_method,
            status,
            input.observations,
            now,
            id,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound("Facture introuvable".into()));
    }

    conn.execute("DELETE FROM invoice_items WHERE invoice_id = ?1", [id])?;
    insert_invoice_items(&conn, id, &input)?;

    let sql = format!("{} WHERE i.id = ?1", INVOICE_SELECT);
    let invoice = conn.query_row(&sql, [id], |row| Invoice::from_row(row))?;
    Ok(invoice)
}

#[tauri::command]
pub fn delete_invoice(state: State<AppState>, id: i64) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    let affected = conn.execute("DELETE FROM invoices WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::NotFound("Facture introuvable".into()));
    }
    Ok(())
}

fn insert_invoice_items(
    conn: &rusqlite::Connection,
    invoice_id: i64,
    input: &InvoiceInput,
) -> Result<(), AppError> {
    let mut items: Vec<(&str, f64)> = vec![
        ("Loyer mensuel", input.rent_amount),
        ("Charges d'eau", input.water_charge),
        ("Charges d'electricite", input.electricity_charge),
    ];
    if input.other_charges > 0.0 {
        items.push(("Autres frais", input.other_charges));
    }
    if input.discount > 0.0 {
        items.push(("Remise", -input.discount));
    }
    for (label, amount) in items {
        conn.execute(
            "INSERT INTO invoice_items (invoice_id, label, amount) VALUES (?1, ?2, ?3)",
            rusqlite::params![invoice_id, label, amount],
        )?;
    }
    Ok(())
}
