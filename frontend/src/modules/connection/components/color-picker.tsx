import { useState } from "react";

import { cn } from "@/lib/utils";
import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";

const PRESET_COLORS = [
  "#ef4444",
  "#f97316",
  "#eab308",
  "#22c55e",
  "#06b6d4",
  "#3b82f6",
  "#8b5cf6",
  "#ec4899",
  "#6b7280",
  "#78716c",
];

interface ColorPickerProps {
  value?: string;
  onChange: (color: string | undefined) => void;
}

export function ColorPicker({ value, onChange }: ColorPickerProps) {
  const { t } = useTranslation();
  const [showCustom, setShowCustom] = useState(false);
  const [customValue, setCustomValue] = useState(value ?? "");

  const handleSelect = (color: string) => {
    onChange(value === color ? undefined : color);
  };

  const handleCustomSubmit = () => {
    const hex = customValue.trim();
    if (/^#[0-9a-fA-F]{6}$/.test(hex)) {
      onChange(hex);
      setShowCustom(false);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <label className="text-sm font-medium text-foreground">
        {t("connection.color")}
      </label>
      <div className="flex flex-wrap items-center gap-1.5">
        {PRESET_COLORS.map((color) => (
          <Button
            key={color}
            type="button"
            variant="ghost"
            className={cn(
              "h-6 w-6 cursor-pointer rounded-full border-2 p-0 transition-transform hover:scale-110",
              value === color ? "border-foreground" : "border-transparent",
            )}
            style={{ backgroundColor: color }}
            onClick={() => handleSelect(color)}
            title={color}
          />
        ))}
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-2 py-1 text-xs"
          onClick={() => setShowCustom(!showCustom)}
        >
          {t("connection.customColor")}
        </Button>
        {value && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-auto px-2 py-1 text-xs text-destructive"
            onClick={() => onChange(undefined)}
          >
            {t("common.actions.clear")}
          </Button>
        )}
      </div>
      {showCustom && (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={customValue}
            onChange={(e) => setCustomValue(e.target.value)}
            placeholder="#3b82f6"
            className="h-7 rounded-sm border border-border bg-background px-2 text-sm text-foreground outline-none focus:border-ring"
            onKeyDown={(e) => {
              if (e.key === "Enter") handleCustomSubmit();
            }}
          />
          <Button
            type="button"
            size="sm"
            className="h-7 px-2 text-xs"
            onClick={handleCustomSubmit}
          >
            {t("common.actions.confirm")}
          </Button>
        </div>
      )}
    </div>
  );
}
