import clsx from "clsx";
import type { InvoiceStatus } from "@/types";
import { STATUS_LABELS } from "@/utils/format";

const statusClasses: Record<InvoiceStatus, string> = {
  paid: "bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300",
  partially_paid: "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300",
  unpaid: "bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-300",
};

export function StatusBadge({ status }: { status: InvoiceStatus }) {
  return (
    <span
      className={clsx(
        "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium",
        statusClasses[status]
      )}
    >
      {STATUS_LABELS[status]}
    </span>
  );
}
