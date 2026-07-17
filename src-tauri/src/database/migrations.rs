use chrono::Utc;
use rusqlite::{Connection, Result};

/// Creates every table used by the application if it does not already exist,
/// and seeds a single default settings row. Safe to call on every startup.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            full_name TEXT NOT NULL DEFAULT '',
            company_name TEXT,
            address TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            city TEXT NOT NULL DEFAULT '',
            country TEXT NOT NULL DEFAULT '',
            currency TEXT NOT NULL DEFAULT 'XOF',
            logo_path TEXT,
            signature_path TEXT,
            tax_number TEXT,
            iban TEXT,
            additional_info TEXT,
            invoice_prefix TEXT NOT NULL DEFAULT 'LOY',
            next_invoice_number INTEGER NOT NULL DEFAULT 1,
            date_format TEXT NOT NULL DEFAULT 'DD/MM/YYYY',
            language TEXT NOT NULL DEFAULT 'fr',
            theme TEXT NOT NULL DEFAULT 'light',
            invoice_template TEXT NOT NULL DEFAULT 'classic',
            updated_at TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS tenants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            phone TEXT NOT NULL,
            email TEXT,
            address TEXT NOT NULL,
            id_number TEXT,
            profession TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS invoices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            invoice_number TEXT NOT NULL UNIQUE,
            tenant_id INTEGER NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
            property_address TEXT NOT NULL,
            description TEXT,
            billing_month INTEGER NOT NULL,
            billing_year INTEGER NOT NULL,
            issue_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            rent_amount REAL NOT NULL DEFAULT 0,
            water_charge REAL NOT NULL DEFAULT 0,
            electricity_charge REAL NOT NULL DEFAULT 0,
            other_charges REAL NOT NULL DEFAULT 0,
            discount REAL NOT NULL DEFAULT 0,
            total_amount REAL NOT NULL DEFAULT 0,
            amount_paid REAL NOT NULL DEFAULT 0,
            balance_due REAL NOT NULL DEFAULT 0,
            payment_method TEXT NOT NULL DEFAULT 'cash',
            status TEXT NOT NULL DEFAULT 'unpaid',
            observations TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS invoice_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            invoice_id INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
            label TEXT NOT NULL,
            amount REAL NOT NULL
        );

        CREATE TABLE IF NOT EXISTS payments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            invoice_id INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
            amount REAL NOT NULL,
            payment_method TEXT NOT NULL,
            paid_at TEXT NOT NULL,
            notes TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_invoices_tenant ON invoices(tenant_id);
        CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
        CREATE INDEX IF NOT EXISTS idx_invoices_year_month ON invoices(billing_year, billing_month);
        CREATE INDEX IF NOT EXISTS idx_tenants_name ON tenants(last_name, first_name);
        "#,
    )?;

    // `settings` predates the `invoice_template` column; add it in place for
    // databases created before this column existed (CREATE TABLE IF NOT
    // EXISTS above only applies to brand new databases).
    let has_template_column: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('settings') WHERE name = 'invoice_template'")?
        .exists([])?;
    if !has_template_column {
        conn.execute(
            "ALTER TABLE settings ADD COLUMN invoice_template TEXT NOT NULL DEFAULT 'classic'",
            [],
        )?;
    }

    // Seed the single settings row on first launch.
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?;
    if count == 0 {
        conn.execute(
            "INSERT INTO settings (id, updated_at) VALUES (1, ?1)",
            [Utc::now().to_rfc3339()],
        )?;
    }

    Ok(())
}
