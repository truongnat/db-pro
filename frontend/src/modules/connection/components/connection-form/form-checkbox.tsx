interface FormCheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function FormCheckbox({ label, checked, onChange, disabled }: FormCheckboxProps) {
  return (
    <label
      className="flex cursor-pointer items-center gap-2 text-sm"
      style={{ color: "var(--color-text)" }}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        disabled={disabled}
        className="h-4 w-4 rounded border accent-[var(--color-primary,#3b82f6)]"
        style={{ borderColor: "var(--color-border)" }}
      />
      {label}
    </label>
  );
}
