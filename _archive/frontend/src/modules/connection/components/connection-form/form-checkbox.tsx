import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";

interface FormCheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function FormCheckbox({ label, checked, onChange, disabled }: FormCheckboxProps) {
  return (
    <Label className="flex cursor-pointer items-center gap-2 text-sm font-normal">
      <Checkbox
        checked={checked}
        onCheckedChange={(val) => onChange(val === true)}
        disabled={disabled}
      />
      {label}
    </Label>
  );
}
