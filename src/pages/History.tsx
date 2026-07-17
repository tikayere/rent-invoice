import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useSearchParams } from "react-router-dom";
import { Search, FileDown, Trash2, Eye, FileSpreadsheet, ChevronLeft, ChevronRight, History as HistoryIcon } from "lucide-react";
import { Card } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { PageLoader } from "@/components/ui/Loader";
import { EmptyState } from "@/components/ui/EmptyState";
import { StatusBadge } from "@/components/ui/Badge";
import { useToast } from "@/context/ToastContext";
import { useSettings } from "@/hooks/useSettings";
import { invoicesApi, pdfApi, exportApi } from "@/services/api";
import { pickPdfSaveTarget, pickCsvSaveTarget, confirmDestructive, notify } from "@/services/tauri";
import { formatCurrency, formatDate, monthLabel } from "@/utils/format";
import type { Invoice, InvoiceFilters, PagedResult, InvoiceStatus } from "@/types";

const PER_PAGE = 10;

export function HistoryPage() {
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const toast = useToast();
  const { settings } = useSettings();
  const currency = settings?.currency ?? "XOF";

  const [search, setSearch] = useState(params.get("search") ?? "");
  const [status, setStatus] = useState<InvoiceStatus | "">("");
  const [year, setYear] = useState<string>("");
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<PagedResult<Invoice> | null>(null);
  const [loading, setLoading] = useState(true);

  const filters: InvoiceFilters = {
    search: search || null,
    status: (status || null) as InvoiceStatus | null,
    year: year ? Number(year) : null,
    month: null,
    page,
    per_page: PER_PAGE,
    sort_by: "issue_date",
    sort_dir: "desc",
  };

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoicesApi.list(filters);
      setResult(data);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [search, status, year, page]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => setPage(1), [search, status, year]);

  async function handleView(inv: Invoice) {
    navigate(`/invoices/${inv.id}/edit`);
  }

  async function handleReprint(inv: Invoice) {
    const dest = await pickPdfSaveTarget(`${inv.invoice_number}.pdf`);
    if (!dest) return;
    try {
      await pdfApi.generateToPath(inv.id, dest);
      await notify("PDF regenere avec succes.");
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleDelete(inv: Invoice) {
    const confirmed = await confirmDestructive(`Supprimer la facture ${inv.invoice_number} ? Cette action est irreversible.`);
    if (!confirmed) return;
    try {
      await invoicesApi.remove(inv.id);
      toast.success("Facture supprimee");
      load();
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleExportCsv() {
    const dest = await pickCsvSaveTarget(`factures-${Date.now()}.csv`);
    if (!dest) return;
    try {
      await exportApi.csv(dest, { ...filters, page: 1, per_page: 100000 });
      toast.success("Export CSV termine");
    } catch (e) {
      toast.error(String(e));
    }
  }

  const years = Array.from({ length: 6 }, (_, i) => new Date().getFullYear() - 2 + i);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-2">
          <div className="relative">
            <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Rechercher (numero, locataire)..."
              className="w-64 rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 py-2 pl-9 pr-3 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
            />
          </div>
          <Select value={status} onChange={(e) => setStatus(e.target.value as InvoiceStatus | "")} className="w-44">
            <option value="">Tous les statuts</option>
            <option value="paid">Paye</option>
            <option value="partially_paid">Partiellement paye</option>
            <option value="unpaid">Non paye</option>
          </Select>
          <Select value={year} onChange={(e) => setYear(e.target.value)} className="w-32">
            <option value="">Toutes annees</option>
            {years.map((y) => (
              <option key={y} value={y}>
                {y}
              </option>
            ))}
          </Select>
        </div>
        <Button variant="outline" onClick={handleExportCsv}>
          <FileSpreadsheet size={16} /> Exporter en CSV
        </Button>
      </div>

      {loading ? (
        <PageLoader />
      ) : !result || result.items.length === 0 ? (
        <EmptyState icon={<HistoryIcon size={40} />} title="Aucune facture trouvee" description="Ajustez vos filtres ou creez une nouvelle facture." />
      ) : (
        <>
          <Card className="overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-slate-50 dark:bg-slate-800/50 text-left text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-4 py-3">Numero</th>
                  <th className="px-4 py-3">Locataire</th>
                  <th className="px-4 py-3">Periode</th>
                  <th className="px-4 py-3">Emise le</th>
                  <th className="px-4 py-3">Total</th>
                  <th className="px-4 py-3">Statut</th>
                  <th className="px-4 py-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {result.items.map((inv) => (
                  <tr key={inv.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/30">
                    <td className="px-4 py-3 font-medium text-slate-900 dark:text-slate-100">{inv.invoice_number}</td>
                    <td className="px-4 py-3 text-slate-600 dark:text-slate-400">{inv.tenant_name}</td>
                    <td className="px-4 py-3 text-slate-600 dark:text-slate-400">
                      {monthLabel(inv.billing_month)} {inv.billing_year}
                    </td>
                    <td className="px-4 py-3 text-slate-600 dark:text-slate-400">{formatDate(inv.issue_date)}</td>
                    <td className="px-4 py-3 text-slate-900 dark:text-slate-100">{formatCurrency(inv.total_amount, currency)}</td>
                    <td className="px-4 py-3">
                      <StatusBadge status={inv.status} />
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex justify-end gap-1">
                        <button onClick={() => handleView(inv)} className="rounded-md p-1.5 text-slate-400 hover:bg-slate-100 hover:text-brand-600 dark:hover:bg-slate-800" title="Voir">
                          <Eye size={16} />
                        </button>
                        <button onClick={() => handleReprint(inv)} className="rounded-md p-1.5 text-slate-400 hover:bg-slate-100 hover:text-brand-600 dark:hover:bg-slate-800" title="Reimprimer">
                          <FileDown size={16} />
                        </button>
                        <button onClick={() => handleDelete(inv)} className="rounded-md p-1.5 text-slate-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950" title="Supprimer">
                          <Trash2 size={16} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </Card>

          <div className="flex items-center justify-between text-sm text-slate-500">
            <span>
              {result.total} facture{result.total > 1 ? "s" : ""} &middot; page {result.page} / {result.total_pages || 1}
            </span>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage((p) => p - 1)}>
                <ChevronLeft size={16} /> Precedent
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={page >= (result.total_pages || 1)}
                onClick={() => setPage((p) => p + 1)}
              >
                Suivant <ChevronRight size={16} />
              </Button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
