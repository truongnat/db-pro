import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

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
      <label className="text-sm font-medium" style={{ color: "var(--color-text)" }}>
        {t("connection.color")}
      </label>
      <div className="flex flex-wrap items-center gap-1.5">
        {PRESET_COLORS.map((color) => (
          <button
            key={color}
            type="button"
            className="h-6 w-6 rounded-full border-2 transition-transform hover:scale-110"
            style={{
              backgroundColor: color,
              borderColor: value === color ? "var(--color-text)" : "transparent",
            }}
            onClick={() => handleSelect(color)}
            title={color}
          />
        ))}
        <button
          type="button"
          className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-surface)]"
          style={{ color: "var(--color-text-secondary)" }}
          onClick={() => setShowCustom(!showCustom)}
        >
          {t("connection.customColor")}
        </button>
        {value && (
          <button
            type="button"
            className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-surface)]"
            style={{ color: "var(--color-error)" }}
            onClick={() => onChange(undefined)}
          >
            {t("common.actions.clear")}
          </button>
        )}
      </div>
      {showCustom && (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={customValue}
            onChange={(e) => setCustomValue(e.target.value)}
            placeholder="#3b82f6"
            className="rounded-[var(--radius-sm)] border px-2 py-1 text-sm"
            style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleCustomSubmit();
            }}
          />
          <button
            type="button"
            className="rounded-[var(--radius-sm)] px-2 py-1 text-xs text-white"
            style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
            onClick={handleCustomSubmit}
          >
            {t("common.actions.confirm")}
          </button>
        </div>
      )}
    </div>
  );
}
