import type { ChangeEvent } from "react";

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
      <label className="text-sm font-medium" style={{ color: "var(--color-text)" }}>
        {label}
        {required && <span style={{ color: "var(--color-error)" }}> *</span>}
      </label>
      <select
        value={value}
        onChange={onChange}
        disabled={disabled}
        className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm outline-none transition-colors focus:border-[var(--color-primary,#3b82f6)]"
        style={{
          borderColor: error ? "var(--color-error)" : "var(--color-border)",
          backgroundColor: "var(--color-bg)",
          color: "var(--color-text)",
          height: "var(--input-height, 36px)",
        }}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      {error && (
        <span className="text-xs" style={{ color: "var(--color-error)" }}>
          {error}
        </span>
      )}
    </div>
  );
}
