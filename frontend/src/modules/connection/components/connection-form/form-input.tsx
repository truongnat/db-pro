import type { ChangeEvent } from "react";

import { cn } from "@/ui/lib/utils";

interface FormInputProps {
  label: string;
  value: string | number;
  onChange: (e: ChangeEvent<HTMLInputElement>) => void;
  type?: string;
  placeholder?: string;
  error?: string;
  required?: boolean;
  disabled?: boolean;
  min?: number;
  max?: number;
}

export function FormInput({
  label,
  value,
  onChange,
  type = "text",
  placeholder,
  error,
  required,
  disabled,
  min,
  max,
}: FormInputProps) {
  return (
    <div className="flex flex-col gap-1">
      <label className="text-sm font-medium" style={{ color: "var(--color-text)" }}>
        {label}
        {required && <span style={{ color: "var(--color-error)" }}> *</span>}
      </label>
      <input
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        min={min}
        max={max}
        className={cn(
          "rounded-[var(--radius-sm)] border px-3 py-2 text-sm outline-none transition-colors focus:border-[var(--color-primary,#3b82f6)]",
          error && "border-[var(--color-error)]",
        )}
        style={{
          borderColor: error ? "var(--color-error)" : "var(--color-border)",
          backgroundColor: "var(--color-bg)",
          color: "var(--color-text)",
          height: "var(--input-height, 36px)",
        }}
      />
      {error && (
        <span className="text-xs" style={{ color: "var(--color-error)" }}>
          {error}
        </span>
      )}
    </div>
  );
}
