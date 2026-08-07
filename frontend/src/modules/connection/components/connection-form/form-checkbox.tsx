import { Checkbox } from "@/components/ui/checkbox";

interface FormCheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function FormCheckbox({ label, checked, onChange, disabled }: FormCheckboxProps) {
  return (
    <label className="flex cursor-pointer items-center gap-2 text-sm text-foreground">
      <Checkbox
        checked={checked}
        onCheckedChange={(val) => onChange(val === true)}
        disabled={disabled}
      />
      {label}
    </label>
  );
}
