export function formatCurrency(amount: number, currency: string): string {
  try {
    return new Intl.NumberFormat("fr-FR", {
      style: "currency",
      currency,
      currencyDisplay: currency.length === 3 ? "symbol" : "code",
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    }).format(amount);
  } catch {
    // Currency code not recognized by Intl (e.g. custom local currency label)
    return `${amount.toLocaleString("fr-FR", { maximumFractionDigits: 2 })} ${currency}`;
  }
}

export function formatDate(iso: string, format = "DD/MM/YYYY"): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  const dd = String(d.getDate()).padStart(2, "0");
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const yyyy = d.getFullYear();
  switch (format) {
    case "MM/DD/YYYY":
      return `${mm}/${dd}/${yyyy}`;
    case "YYYY-MM-DD":
      return `${yyyy}-${mm}-${dd}`;
    default:
      return `${dd}/${mm}/${yyyy}`;
  }
}

export const MONTH_NAMES_FR = [
  "Janvier", "Fevrier", "Mars", "Avril", "Mai", "Juin",
  "Juillet", "Aout", "Septembre", "Octobre", "Novembre", "Decembre",
];

export function monthLabel(month: number): string {
  return MONTH_NAMES_FR[month - 1] ?? String(month);
}

export function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

export function addDaysIso(iso: string, days: number): string {
  const d = new Date(iso);
  d.setDate(d.getDate() + days);
  return d.toISOString().slice(0, 10);
}

export const STATUS_LABELS: Record<string, string> = {
  paid: "Paye",
  partially_paid: "Partiellement paye",
  unpaid: "Non paye",
};

export const PAYMENT_METHOD_LABELS: Record<string, string> = {
  cash: "Especes",
  bank_transfer: "Virement",
  mobile_money: "Mobile Money",
  check: "Cheque",
  other: "Autre",
};
