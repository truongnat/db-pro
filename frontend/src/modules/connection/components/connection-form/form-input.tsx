import type { ChangeEvent } from "react";

import { Input } from "@/components/ui/input";

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
    <div className="flex flex-col gap-1.5">
      <label className="text-xs font-medium text-foreground">
        {label}
        {required && <span className="text-destructive"> *</span>}
      </label>
      <Input
        type={type}
        value={String(value)}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        min={min}
        max={max}
        aria-invalid={!!error}
      />
      {error && (
        <span className="text-xs text-destructive">
          {error}
        </span>
      )}
    </div>
  );
}
