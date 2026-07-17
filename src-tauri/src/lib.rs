pub mod commands;
pub mod database;
pub mod error;
pub mod models;
pub mod pdf;
pub mod services;

pub use database::AppState;
pub use error::AppError;

use tauri::Manager;

/// Builds and runs the Tauri application. Called from `main.rs`; split out
/// into the library crate so the app can also be exercised from integration
/// tests if needed later.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            let state = AppState::new(&app_data_dir).expect("failed to initialize database");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings_commands::get_settings,
            commands::settings_commands::update_settings,
            commands::tenant_commands::list_tenants,
            commands::tenant_commands::get_tenant,
            commands::tenant_commands::create_tenant,
            commands::tenant_commands::update_tenant,
            commands::tenant_commands::delete_tenant,
            commands::invoice_commands::list_invoices,
            commands::invoice_commands::get_invoice,
            commands::invoice_commands::create_invoice,
            commands::invoice_commands::update_invoice,
            commands::invoice_commands::delete_invoice,
            commands::invoice_commands::preview_next_invoice_number,
            commands::pdf_commands::generate_invoice_pdf,
            commands::pdf_commands::preview_invoice_pdf,
            commands::dashboard_commands::get_dashboard_stats,
            commands::backup_commands::export_database,
            commands::backup_commands::import_database,
            commands::export_commands::export_invoices_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}
