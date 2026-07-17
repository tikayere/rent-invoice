use rusqlite::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Invoice {
    pub id: i64,
    pub invoice_number: String,
    pub tenant_id: i64,
    pub tenant_name: String,
    pub property_address: String,
    pub description: Option<String>,
    pub billing_month: i64,
    pub billing_year: i64,
    pub issue_date: String,
    pub due_date: String,
    pub rent_amount: f64,
    pub water_charge: f64,
    pub electricity_charge: f64,
    pub other_charges: f64,
    pub discount: f64,
    pub total_amount: f64,
    pub amount_paid: f64,
    pub balance_due: f64,
    pub payment_method: String,
    pub status: String,
    pub observations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Invoice {
    /// Expects a query that joins `invoices` with `tenants` and exposes
    /// `tenant_name` as `first_name || ' ' || last_name`.
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Invoice {
            id: row.get("id")?,
            invoice_number: row.get("invoice_number")?,
            tenant_id: row.get("tenant_id")?,
            tenant_name: row.get("tenant_name")?,
            property_address: row.get("property_address")?,
            description: row.get("description")?,
            billing_month: row.get("billing_month")?,
            billing_year: row.get("billing_year")?,
            issue_date: row.get("issue_date")?,
            due_date: row.get("due_date")?,
            rent_amount: row.get("rent_amount")?,
            water_charge: row.get("water_charge")?,
            electricity_charge: row.get("electricity_charge")?,
            other_charges: row.get("other_charges")?,
            discount: row.get("discount")?,
            total_amount: row.get("total_amount")?,
            amount_paid: row.get("amount_paid")?,
            balance_due: row.get("balance_due")?,
            payment_method: row.get("payment_method")?,
            status: row.get("status")?,
            observations: row.get("observations")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceInput {
    pub tenant_id: i64,
    pub property_address: String,
    pub description: Option<String>,
    pub billing_month: i64,
    pub billing_year: i64,
    pub issue_date: String,
    pub due_date: String,
    pub rent_amount: f64,
    pub water_charge: f64,
    pub electricity_charge: f64,
    pub other_charges: f64,
    pub discount: f64,
    pub amount_paid: f64,
    pub payment_method: String,
    pub observations: Option<String>,
}

impl InvoiceInput {
    pub fn total(&self) -> f64 {
        let sum = self.rent_amount + self.water_charge + self.electricity_charge + self.other_charges
            - self.discount;
        (sum.max(0.0) * 100.0).round() / 100.0
    }

    pub fn balance_due(&self) -> f64 {
        ((self.total() - self.amount_paid).max(0.0) * 100.0).round() / 100.0
    }

    pub fn status(&self) -> &'static str {
        let total = self.total();
        if self.amount_paid <= 0.0 {
            "unpaid"
        } else if self.amount_paid + 0.001 >= total && total > 0.0 {
            "paid"
        } else {
            "partially_paid"
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceFilters {
    pub search: Option<String>,
    pub status: Option<String>,
    pub year: Option<i64>,
    pub month: Option<i64>,
    pub page: i64,
    pub per_page: i64,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardStats {
    pub total_invoices: i64,
    pub total_tenants: i64,
    pub last_invoice: Option<Invoice>,
    pub total_outstanding: f64,
    pub total_collected_this_month: f64,
}
