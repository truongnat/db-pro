import type { ChangeEvent } from "react";

import { cn } from "@/lib/utils";

interface FormSelectProps {
  label: string;
  value: string;
  onChange: (e: ChangeEvent<HTMLSelectElement>) => void;
  options: { value: string; label: string }[];
  error?: string;
  required?: boolean;
  disabled?: boolean;
}

export function FormSelect({
  label,
  value,
  onChange,
  options,
  error,
  required,
  disabled,
}: FormSelectProps) {
  return (
    <div className="flex flex-col gap-1">
      <label className="text-sm font-medium text-foreground">
        {label}
        {required && <span className="text-destructive"> *</span>}
      </label>
      <select
        value={value}
        onChange={onChange}
        disabled={disabled}
        className={cn(
          "h-9 rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-primary",
          error && "border-destructive",
        )}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      {error && (
        <span className="text-xs text-destructive">
          {error}
        </span>
      )}
    </div>
  );
}
