interface FormCheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function FormCheckbox({ label, checked, onChange, disabled }: FormCheckboxProps) {
  return (
    <label
      className="flex cursor-pointer items-center gap-2 text-sm text-foreground"
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        disabled={disabled}
        className="h-4 w-4 rounded border-border accent-primary"
      />
      {label}
    </label>
  );
}
