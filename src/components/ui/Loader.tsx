import { Loader2 } from "lucide-react";

export function PageLoader({ label = "Chargement..." }: { label?: string }) {
  return (
    <div className="flex h-full min-h-[300px] flex-col items-center justify-center gap-3 text-slate-400">
      <Loader2 size={28} className="animate-spin" />
      <span className="text-sm">{label}</span>
    </div>
  );
}
