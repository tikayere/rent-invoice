import { useNavigate } from "react-router-dom";
import { useState } from "react";
import { Sun, Moon, Search } from "lucide-react";
import { useTheme } from "@/context/ThemeContext";

export function Header({ title }: { title: string }) {
  const { theme, toggle } = useTheme();
  const [query, setQuery] = useState("");
  const navigate = useNavigate();

  function onSearchSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (query.trim()) navigate(`/history?search=${encodeURIComponent(query.trim())}`);
  }

  return (
    <header className="flex h-16 items-center justify-between border-b border-slate-200 dark:border-slate-800 bg-white/70 dark:bg-slate-900/70 backdrop-blur px-6">
      <h1 className="text-lg font-semibold text-slate-900 dark:text-slate-100">{title}</h1>
      <div className="flex items-center gap-3">
        <form onSubmit={onSearchSubmit} className="relative hidden sm:block">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Rechercher une facture..."
            className="w-64 rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 py-1.5 pl-9 pr-3 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
        </form>
        <button
          onClick={toggle}
          className="rounded-lg border border-slate-200 dark:border-slate-700 p-2 text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800"
          title={theme === "dark" ? "Mode clair" : "Mode sombre"}
        >
          {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
        </button>
      </div>
    </header>
  );
}
