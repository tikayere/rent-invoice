use serde::Serialize;

/// A single payment record logged against an invoice. Currently written
/// automatically whenever an invoice is created or updated with a nonzero
/// `amount_paid`, and kept for future partial-payment history features.
#[derive(Debug, Clone, Serialize)]
pub struct Payment {
    pub id: i64,
    pub invoice_id: i64,
    pub amount: f64,
    pub payment_method: String,
    pub paid_at: String,
    pub notes: Option<String>,
}
