import { NavLink } from "react-router-dom";
import clsx from "clsx";
import { LayoutDashboard, FileText, Users, History, Settings, Building2 } from "lucide-react";

const navItems = [
  { to: "/", label: "Tableau de bord", icon: LayoutDashboard, end: true },
  { to: "/invoices/new", label: "Nouvelle facture", icon: FileText },
  { to: "/tenants", label: "Locataires", icon: Users },
  { to: "/history", label: "Historique", icon: History },
  { to: "/settings", label: "Parametres", icon: Settings },
];

export function Sidebar() {
  return (
    <aside className="flex h-full w-60 flex-col border-r border-slate-200 bg-white dark:bg-slate-900 dark:border-slate-800">
      <div className="flex items-center gap-2 px-5 py-5">
        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-brand-600 text-white">
          <Building2 size={18} />
        </div>
        <div>
          <p className="text-sm font-semibold text-slate-900 dark:text-slate-100 leading-tight">
            Gestion Loyers
          </p>
          <p className="text-xs text-slate-400 leading-tight">Facturation locale</p>
        </div>
      </div>
      <nav className="flex-1 space-y-1 px-3">
        {navItems.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              clsx(
                "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                isActive
                  ? "bg-brand-50 text-brand-700 dark:bg-brand-900/40 dark:text-brand-300"
                  : "text-slate-600 hover:bg-slate-100 dark:text-slate-400 dark:hover:bg-slate-800"
              )
            }
          >
            <Icon size={18} />
            {label}
          </NavLink>
        ))}
      </nav>
      <div className="px-5 py-4 text-xs text-slate-400 border-t border-slate-200 dark:border-slate-800">
        100% local &middot; aucune connexion requise
      </div>
    </aside>
  );
}
