import { Outlet, useLocation } from "react-router-dom";
import { Sidebar } from "@/components/layout/Sidebar";
import { Header } from "@/components/layout/Header";

const titles: Record<string, string> = {
  "/": "Tableau de bord",
  "/tenants": "Locataires",
  "/invoices/new": "Nouvelle facture",
  "/history": "Historique des factures",
  "/settings": "Parametres",
};

function resolveTitle(pathname: string): string {
  if (titles[pathname]) return titles[pathname];
  if (pathname.startsWith("/invoices/") && pathname.endsWith("/edit")) return "Modifier la facture";
  return "Gestion des loyers";
}

export function MainLayout() {
  const location = useLocation();
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-50 dark:bg-slate-950">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header title={resolveTitle(location.pathname)} />
        <main className="flex-1 overflow-y-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
