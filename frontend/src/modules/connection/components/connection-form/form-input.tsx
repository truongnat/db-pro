import type { ChangeEvent } from "react";

import { cn } from "@/lib/utils";

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
      <label className="text-sm font-medium text-foreground">
        {label}
        {required && <span className="text-destructive"> *</span>}
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
          "h-9 rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-primary",
          error && "border-destructive",
        )}
      />
      {error && (
        <span className="text-xs text-destructive">
          {error}
        </span>
      )}
    </div>
  );
}
