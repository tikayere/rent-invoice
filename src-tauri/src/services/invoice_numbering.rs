/// Formats an invoice number using the bailleur's configured prefix, the
/// calendar year of issuance, and a zero-padded sequential counter.
/// Example: prefix "LOY", year 2026, sequence 1 -> "LOY-2026-000001".
pub fn format_invoice_number(prefix: &str, year: i32, sequence: i64) -> String {
    format!("{}-{}-{:06}", prefix, year, sequence)
}
