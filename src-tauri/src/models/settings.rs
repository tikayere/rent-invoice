use rusqlite::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    pub id: i64,
    pub full_name: String,
    pub company_name: Option<String>,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub city: String,
    pub country: String,
    pub currency: String,
    pub logo_path: Option<String>,
    pub signature_path: Option<String>,
    pub tax_number: Option<String>,
    pub iban: Option<String>,
    pub additional_info: Option<String>,
    pub invoice_prefix: String,
    pub next_invoice_number: i64,
    pub date_format: String,
    pub language: String,
    pub theme: String,
    pub invoice_template: String,
    pub updated_at: String,
}

impl Settings {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Settings {
            id: row.get("id")?,
            full_name: row.get("full_name")?,
            company_name: row.get("company_name")?,
            address: row.get("address")?,
            phone: row.get("phone")?,
            email: row.get("email")?,
            city: row.get("city")?,
            country: row.get("country")?,
            currency: row.get("currency")?,
            logo_path: row.get("logo_path")?,
            signature_path: row.get("signature_path")?,
            tax_number: row.get("tax_number")?,
            iban: row.get("iban")?,
            additional_info: row.get("additional_info")?,
            invoice_prefix: row.get("invoice_prefix")?,
            next_invoice_number: row.get("next_invoice_number")?,
            date_format: row.get("date_format")?,
            language: row.get("language")?,
            theme: row.get("theme")?,
            invoice_template: row.get("invoice_template")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsInput {
    pub full_name: String,
    pub company_name: Option<String>,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub city: String,
    pub country: String,
    pub currency: String,
    pub logo_path: Option<String>,
    pub signature_path: Option<String>,
    pub tax_number: Option<String>,
    pub iban: Option<String>,
    pub additional_info: Option<String>,
    pub invoice_prefix: String,
    pub date_format: String,
    pub language: String,
    pub theme: String,
    pub invoice_template: String,
}
