import { useCallback, useEffect, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";

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
        className="w-[400px] rounded-md border border-border bg-muted p-4 shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="mb-3 text-sm font-semibold text-foreground">
          {t("dataGrid.chartConfig")}
        </h3>

        <div className="space-y-3">
          <div>
            <label className="mb-1 block text-xs text-muted-foreground">
              {t("dataGrid.chartType")}
            </label>
            <select
              className="w-full rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground"
              value={type}
              onChange={(e) => setType(e.target.value as ChartConfig["type"])}
            >
              <option value="bar">{t("dataGrid.chartBar")}</option>
              <option value="line">{t("dataGrid.chartLine")}</option>
              <option value="pie">{t("dataGrid.chartPie")}</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-xs text-muted-foreground">
              {t("dataGrid.chartXAxis")}
            </label>
            <select
              className="w-full rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground"
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
            <label className="mb-1 block text-xs text-muted-foreground">
              {t("dataGrid.chartYAxis")}
            </label>
            <select
              className="w-full rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground"
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
          <Button
            type="button"
            variant="ghost"
            onClick={onClose}
            className="text-muted-foreground"
          >
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="button"
            onClick={handleApply}
            disabled={!xColumn || !yColumn}
          >
            {t("common.actions.confirm")}
          </Button>
        </div>
      </div>
    </div>
  );
}
