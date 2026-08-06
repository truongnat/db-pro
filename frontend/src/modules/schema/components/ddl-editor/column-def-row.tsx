import type { ChangeEvent } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import type { ColumnDef } from "../../services/ddl-builder";

interface ColumnDefRowProps {
  column: ColumnDef;
  index: number;
  onChange: (index: number, field: keyof ColumnDef, value: string | boolean) => void;
  onRemove: (index: number) => void;
  canRemove: boolean;
}

export function ColumnDefRow({ column, index, onChange, onRemove, canRemove }: ColumnDefRowProps) {
  const { t } = useTranslation();

  const handleField = (field: keyof ColumnDef) => (e: ChangeEvent<HTMLInputElement>) => {
    onChange(index, field, e.target.value);
  };

  return (
    <div className="grid grid-cols-[1fr_1fr_80px_1fr_40px_32px] items-end gap-2">
      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
            {t("schema.columnName")}
          </label>
        )}
        <input
          type="text"
          value={column.name}
          onChange={handleField("name")}
          placeholder="column_name"
          className="rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm outline-none focus:border-[var(--color-primary,#3b82f6)]"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-bg)",
            color: "var(--color-text)",
          }}
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
            {t("schema.columnDataType")}
          </label>
        )}
        <input
          type="text"
          value={column.dataType}
          onChange={handleField("dataType")}
          placeholder="TEXT"
          className="rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm outline-none focus:border-[var(--color-primary,#3b82f6)]"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-bg)",
            color: "var(--color-text)",
          }}
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
            {t("schema.columnNullable")}
          </label>
        )}
        <label className="flex items-center justify-center">
          <input
            type="checkbox"
            checked={column.nullable}
            onChange={(e) => onChange(index, "nullable", e.target.checked)}
            className="h-4 w-4 rounded border accent-[var(--color-primary,#3b82f6)]"
            style={{ borderColor: "var(--color-border)" }}
          />
        </label>
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
            {t("schema.columnDefault")}
          </label>
        )}
        <input
          type="text"
          value={column.defaultValue}
          onChange={handleField("defaultValue")}
          placeholder="DEFAULT"
          className="rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm outline-none focus:border-[var(--color-primary,#3b82f6)]"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-bg)",
            color: "var(--color-text)",
          }}
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
            {t("schema.columnPk")}
          </label>
        )}
        <label className="flex items-center justify-center">
          <input
            type="checkbox"
            checked={column.isPk}
            onChange={(e) => onChange(index, "isPk", e.target.checked)}
            className="h-4 w-4 rounded border accent-[var(--color-primary,#3b82f6)]"
            style={{ borderColor: "var(--color-border)" }}
          />
        </label>
      </div>

      <button
        type="button"
        onClick={() => onRemove(index)}
        disabled={!canRemove}
        className="flex items-center justify-center rounded-[var(--radius-sm)] text-sm transition-colors hover:bg-[var(--color-bg)]"
        style={{
          color: canRemove ? "var(--color-error, #ef4444)" : "var(--color-text-secondary)",
          opacity: canRemove ? 1 : 0.3,
          height: "var(--input-height, 36px)",
        }}
        title={t("common.actions.delete")}
      >
        ×
      </button>
    </div>
  );
}
