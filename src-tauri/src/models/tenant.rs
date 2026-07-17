use rusqlite::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Tenant {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: String,
    pub id_number: Option<String>,
    pub profession: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_count: Option<i64>,
}

impl Tenant {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Tenant {
            id: row.get("id")?,
            first_name: row.get("first_name")?,
            last_name: row.get("last_name")?,
            phone: row.get("phone")?,
            email: row.get("email")?,
            address: row.get("address")?,
            id_number: row.get("id_number")?,
            profession: row.get("profession")?,
            notes: row.get("notes")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            invoice_count: row.get("invoice_count").ok(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantInput {
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: String,
    pub id_number: Option<String>,
    pub profession: Option<String>,
    pub notes: Option<String>,
}
