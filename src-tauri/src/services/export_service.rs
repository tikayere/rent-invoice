use std::path::Path;

use crate::models::Invoice;
use crate::AppError;

/// Writes a list of invoices to a CSV file, using French column headers so
/// the export is directly usable in local spreadsheet tools.
pub fn export_invoices_csv(invoices: &[Invoice], dest_path: &Path) -> Result<(), AppError> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(dest_path)
        .map_err(|e| AppError::Io(e.to_string()))?;

    writer
        .write_record([
            "Numero",
            "Locataire",
            "Adresse du bien",
            "Mois",
            "Annee",
            "Date d'emission",
            "Date d'echeance",
            "Loyer",
            "Eau",
            "Electricite",
            "Autres frais",
            "Remise",
            "Total",
            "Montant paye",
            "Reste a payer",
            "Mode de paiement",
            "Statut",
        ])
        .map_err(|e| AppError::Io(e.to_string()))?;

    for inv in invoices {
        writer
            .write_record([
                inv.invoice_number.clone(),
                inv.tenant_name.clone(),
                inv.property_address.clone(),
                inv.billing_month.to_string(),
                inv.billing_year.to_string(),
                inv.issue_date.clone(),
                inv.due_date.clone(),
                format!("{:.2}", inv.rent_amount),
                format!("{:.2}", inv.water_charge),
                format!("{:.2}", inv.electricity_charge),
                format!("{:.2}", inv.other_charges),
                format!("{:.2}", inv.discount),
                format!("{:.2}", inv.total_amount),
                format!("{:.2}", inv.amount_paid),
                format!("{:.2}", inv.balance_due),
                inv.payment_method.clone(),
                inv.status.clone(),
            ])
            .map_err(|e| AppError::Io(e.to_string()))?;
    }

    writer.flush().map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}
