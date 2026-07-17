import { invoke } from "@tauri-apps/api/core";
import type {
  Settings,
  SettingsInput,
  Tenant,
  TenantInput,
  Invoice,
  InvoiceInput,
  InvoiceFilters,
  PagedResult,
  DashboardStats,
} from "@/types";

/**
 * Thin typed wrapper around Tauri's `invoke`. Every Rust command lives in
 * src-tauri/src/commands and returns `Result<T, String>`, so failures surface
 * here as rejected promises with a human-readable message.
 */

// ---------- Settings ----------
export const settingsApi = {
  get: () => invoke<Settings>("get_settings"),
  update: (input: SettingsInput) => invoke<Settings>("update_settings", { input }),
};

// ---------- Tenants ----------
export const tenantsApi = {
  list: (search?: string) => invoke<Tenant[]>("list_tenants", { search: search ?? null }),
  get: (id: number) => invoke<Tenant>("get_tenant", { id }),
  create: (input: TenantInput) => invoke<Tenant>("create_tenant", { input }),
  update: (id: number, input: TenantInput) => invoke<Tenant>("update_tenant", { id, input }),
  remove: (id: number) => invoke<void>("delete_tenant", { id }),
};

// ---------- Invoices ----------
export const invoicesApi = {
  list: (filters: InvoiceFilters) => invoke<PagedResult<Invoice>>("list_invoices", { filters }),
  get: (id: number) => invoke<Invoice>("get_invoice", { id }),
  create: (input: InvoiceInput) => invoke<Invoice>("create_invoice", { input }),
  update: (id: number, input: InvoiceInput) => invoke<Invoice>("update_invoice", { id, input }),
  remove: (id: number) => invoke<void>("delete_invoice", { id }),
  nextNumber: () => invoke<string>("preview_next_invoice_number"),
};

// ---------- PDF ----------
export const pdfApi = {
  /** Generates the PDF and saves it to the given absolute path. */
  generateToPath: (invoiceId: number, destPath: string) =>
    invoke<void>("generate_invoice_pdf", { invoiceId, destPath }),
  /** Generates the PDF and returns raw bytes (base64) for in-app preview. */
  generatePreview: (invoiceId: number) => invoke<number[]>("preview_invoice_pdf", { invoiceId }),
};

// ---------- Dashboard ----------
export const dashboardApi = {
  stats: () => invoke<DashboardStats>("get_dashboard_stats"),
};

// ---------- Backup ----------
export const backupApi = {
  exportTo: (destPath: string) => invoke<void>("export_database", { destPath }),
  importFrom: (srcPath: string) => invoke<void>("import_database", { srcPath }),
};

// ---------- Export ----------
export const exportApi = {
  csv: (destPath: string, filters: InvoiceFilters) =>
    invoke<void>("export_invoices_csv", { destPath, filters }),
};
