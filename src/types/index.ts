// Domain types shared between the React frontend and the Rust backend.
// Field names use snake_case to match the JSON produced by serde on the Rust side.

export type PaymentMethod = "cash" | "bank_transfer" | "mobile_money" | "check" | "other";
export type InvoiceStatus = "paid" | "partially_paid" | "unpaid";
export type Theme = "light" | "dark";
export type InvoiceTemplateKey = "classic" | "modern" | "minimal";

export interface Settings {
  id: number;
  full_name: string;
  company_name: string | null;
  address: string;
  phone: string;
  email: string;
  city: string;
  country: string;
  currency: string;
  logo_path: string | null;
  signature_path: string | null;
  tax_number: string | null;
  iban: string | null;
  additional_info: string | null;
  invoice_prefix: string;
  next_invoice_number: number;
  date_format: string;
  language: string;
  theme: Theme;
  invoice_template: InvoiceTemplateKey;
  updated_at: string;
}

export interface SettingsInput {
  full_name: string;
  company_name?: string | null;
  address: string;
  phone: string;
  email: string;
  city: string;
  country: string;
  currency: string;
  logo_path?: string | null;
  signature_path?: string | null;
  tax_number?: string | null;
  iban?: string | null;
  additional_info?: string | null;
  invoice_prefix: string;
  date_format: string;
  language: string;
  theme: Theme;
  invoice_template: InvoiceTemplateKey;
}

export interface Tenant {
  id: number;
  first_name: string;
  last_name: string;
  phone: string;
  email: string | null;
  address: string;
  id_number: string | null;
  profession: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  invoice_count?: number;
}

export interface TenantInput {
  first_name: string;
  last_name: string;
  phone: string;
  email?: string | null;
  address: string;
  id_number?: string | null;
  profession?: string | null;
  notes?: string | null;
}

export interface Invoice {
  id: number;
  invoice_number: string;
  tenant_id: number;
  tenant_name: string;
  property_address: string;
  description: string | null;
  billing_month: number;
  billing_year: number;
  issue_date: string;
  due_date: string;
  rent_amount: number;
  water_charge: number;
  electricity_charge: number;
  other_charges: number;
  discount: number;
  total_amount: number;
  amount_paid: number;
  balance_due: number;
  payment_method: PaymentMethod;
  status: InvoiceStatus;
  observations: string | null;
  created_at: string;
  updated_at: string;
}

export interface InvoiceInput {
  tenant_id: number;
  property_address: string;
  description?: string | null;
  billing_month: number;
  billing_year: number;
  issue_date: string;
  due_date: string;
  rent_amount: number;
  water_charge: number;
  electricity_charge: number;
  other_charges: number;
  discount: number;
  amount_paid: number;
  payment_method: PaymentMethod;
  observations?: string | null;
}

export interface InvoiceFilters {
  search?: string | null;
  status?: InvoiceStatus | null;
  year?: number | null;
  month?: number | null;
  page: number;
  per_page: number;
  sort_by?: string | null;
  sort_dir?: "asc" | "desc" | null;
}

export interface PagedResult<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface DashboardStats {
  total_invoices: number;
  total_tenants: number;
  last_invoice: Invoice | null;
  total_outstanding: number;
  total_collected_this_month: number;
}
