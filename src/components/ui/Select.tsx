import { forwardRef, type SelectHTMLAttributes } from "react";
import clsx from "clsx";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, error, className, id, children, ...props }, ref) => {
    const inputId = id ?? props.name;
    return (
      <div className="flex flex-col gap-1">
        {label && (
          <label htmlFor={inputId} className="text-sm font-medium text-slate-700 dark:text-slate-300">
            {label}
          </label>
        )}
        <select
          id={inputId}
          ref={ref}
          className={clsx(
            "rounded-lg border px-3 py-2 text-sm bg-white transition-colors",
            "focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-brand-500",
            "dark:bg-slate-950 dark:text-slate-100",
            error ? "border-red-400" : "border-slate-300 dark:border-slate-700",
            className
          )}
          {...props}
        >
          {children}
        </select>
        {error && <span className="text-xs text-red-500">{error}</span>}
      </div>
    );
  }
);
Select.displayName = "Select";
