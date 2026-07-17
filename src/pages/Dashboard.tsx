import { useNavigate } from "react-router-dom";
import { FileText, Users, PlusCircle, History as HistoryIcon, AlertCircle, TrendingUp } from "lucide-react";
import { Card, CardBody } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { PageLoader } from "@/components/ui/Loader";
import { StatusBadge } from "@/components/ui/Badge";
import { useDashboard } from "@/hooks/useDashboard";
import { useSettings } from "@/hooks/useSettings";
import { formatCurrency, formatDate, monthLabel } from "@/utils/format";

export function Dashboard() {
  const { stats, loading } = useDashboard();
  const { settings } = useSettings();
  const navigate = useNavigate();
  const currency = settings?.currency ?? "XOF";

  if (loading || !stats) return <PageLoader label="Chargement du tableau de bord..." />;

  const statCards = [
    {
      label: "Factures totales",
      value: stats.total_invoices.toLocaleString("fr-FR"),
      icon: FileText,
      tint: "bg-blue-50 text-blue-600 dark:bg-blue-950 dark:text-blue-400",
    },
    {
      label: "Locataires",
      value: stats.total_tenants.toLocaleString("fr-FR"),
      icon: Users,
      tint: "bg-violet-50 text-violet-600 dark:bg-violet-950 dark:text-violet-400",
    },
    {
      label: "Encaisse ce mois",
      value: formatCurrency(stats.total_collected_this_month, currency),
      icon: TrendingUp,
      tint: "bg-emerald-50 text-emerald-600 dark:bg-emerald-950 dark:text-emerald-400",
    },
    {
      label: "Reste a payer (total)",
      value: formatCurrency(stats.total_outstanding, currency),
      icon: AlertCircle,
      tint: "bg-amber-50 text-amber-600 dark:bg-amber-950 dark:text-amber-400",
    },
  ];

  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {statCards.map((s) => (
          <Card key={s.label}>
            <CardBody className="flex items-center gap-4">
              <div className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-lg ${s.tint}`}>
                <s.icon size={20} />
              </div>
              <div className="min-w-0">
                <p className="text-xs font-medium text-slate-500 dark:text-slate-400">{s.label}</p>
                <p className="truncate text-xl font-semibold text-slate-900 dark:text-slate-100">
                  {s.value}
                </p>
              </div>
            </CardBody>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardBody>
            <h3 className="mb-1 text-base font-semibold text-slate-900 dark:text-slate-100">
              Derniere facture
            </h3>
            {stats.last_invoice ? (
              <div className="mt-3 rounded-lg border border-slate-200 dark:border-slate-800 p-4">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <p className="font-medium text-slate-900 dark:text-slate-100">
                      {stats.last_invoice.invoice_number} &middot; {stats.last_invoice.tenant_name}
                    </p>
                    <p className="text-sm text-slate-500">
                      {monthLabel(stats.last_invoice.billing_month)} {stats.last_invoice.billing_year}{" "}
                      &middot; Emise le {formatDate(stats.last_invoice.issue_date)}
                    </p>
                  </div>
                  <StatusBadge status={stats.last_invoice.status} />
                </div>
                <div className="mt-3 flex items-center justify-between">
                  <span className="text-lg font-semibold text-slate-900 dark:text-slate-100">
                    {formatCurrency(stats.last_invoice.total_amount, currency)}
                  </span>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => navigate(`/invoices/${stats.last_invoice!.id}/edit`)}
                  >
                    Voir la facture
                  </Button>
                </div>
              </div>
            ) : (
              <p className="mt-4 text-sm text-slate-400">Aucune facture creee pour le moment.</p>
            )}
          </CardBody>
        </Card>

        <Card>
          <CardBody className="flex flex-col gap-3">
            <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100">
              Actions rapides
            </h3>
            <Button onClick={() => navigate("/invoices/new")} className="justify-start">
              <PlusCircle size={16} /> Nouvelle facture
            </Button>
            <Button variant="outline" onClick={() => navigate("/history")} className="justify-start">
              <HistoryIcon size={16} /> Historique
            </Button>
            <Button variant="outline" onClick={() => navigate("/tenants")} className="justify-start">
              <Users size={16} /> Locataires
            </Button>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}
