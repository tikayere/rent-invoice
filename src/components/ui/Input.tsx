import { forwardRef, type InputHTMLAttributes } from "react";
import clsx from "clsx";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, hint, className, id, ...props }, ref) => {
    const inputId = id ?? props.name;
    return (
      <div className="flex flex-col gap-1">
        {label && (
          <label htmlFor={inputId} className="text-sm font-medium text-slate-700 dark:text-slate-300">
            {label}
          </label>
        )}
        <input
          id={inputId}
          ref={ref}
          className={clsx(
            "rounded-lg border px-3 py-2 text-sm bg-white transition-colors",
            "focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-brand-500",
            "dark:bg-slate-950 dark:text-slate-100",
            error
              ? "border-red-400 focus:ring-red-400 focus:border-red-400"
              : "border-slate-300 dark:border-slate-700",
            className
          )}
          {...props}
        />
        {hint && !error && <span className="text-xs text-slate-400">{hint}</span>}
        {error && <span className="text-xs text-red-500">{error}</span>}
      </div>
    );
  }
);
Input.displayName = "Input";
