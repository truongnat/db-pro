import { useCallback, useEffect, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import type { ChartConfig } from "../state/data-grid.store";
import type { ColumnMeta } from "../types/data-grid.types";

interface ChartConfigDialogProps {
  open: boolean;
  onClose: () => void;
  columns: ColumnMeta[];
  config: ChartConfig | null;
  onApply: (config: ChartConfig) => void;
}

export function ChartConfigDialog({
  open,
  onClose,
  columns,
  config,
  onApply,
}: ChartConfigDialogProps) {
  const { t } = useTranslation();
  const [type, setType] = useState<ChartConfig["type"]>("bar");
  const [xColumn, setXColumn] = useState("");
  const [yColumn, setYColumn] = useState("");

  useEffect(() => {
    if (open) {
      setType(config?.type ?? "bar");
      setXColumn(config?.xColumn ?? columns[0]?.name ?? "");
      setYColumn(config?.yColumn ?? columns[1]?.name ?? columns[0]?.name ?? "");
    }
  }, [open, config, columns]);

  const handleApply = useCallback(() => {
    if (!xColumn || !yColumn) return;
    onApply({ type, xColumn, yColumn });
    onClose();
  }, [type, xColumn, yColumn, onApply, onClose]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
      onClick={onClose}
    >
      <div
        className="w-[400px] rounded-[var(--radius-md)] p-4 shadow-lg"
        style={{
          backgroundColor: "var(--color-bg-secondary, #1e293b)",
          border: "1px solid var(--color-border)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="mb-3 text-sm font-semibold" style={{ color: "var(--color-text)" }}>
          {t("dataGrid.chartConfig")}
        </h3>

        <div className="space-y-3">
          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("dataGrid.chartType")}
            </label>
            <select
              className="w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
              value={type}
              onChange={(e) => setType(e.target.value as ChartConfig["type"])}
            >
              <option value="bar">{t("dataGrid.chartBar")}</option>
              <option value="line">{t("dataGrid.chartLine")}</option>
              <option value="pie">{t("dataGrid.chartPie")}</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("dataGrid.chartXAxis")}
            </label>
            <select
              className="w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
              value={xColumn}
              onChange={(e) => setXColumn(e.target.value)}
            >
              {columns.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("dataGrid.chartYAxis")}
            </label>
            <select
              className="w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
              value={yColumn}
              onChange={(e) => setYColumn(e.target.value)}
            >
              {columns.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-[var(--radius-sm)] px-3 py-1.5 text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("common.actions.cancel")}
          </button>
          <button
            type="button"
            onClick={handleApply}
            disabled={!xColumn || !yColumn}
            className="rounded-[var(--radius-sm)] px-3 py-1.5 text-sm text-white disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary, #3b82f6)" }}
          >
            {t("common.actions.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}
