import { useCallback, useEffect, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

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

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>{t("dataGrid.chartConfig")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label className="text-xs">{t("dataGrid.chartType")}</Label>
            <Select value={type} onValueChange={(v) => setType(v as ChartConfig["type"])}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="bar">{t("dataGrid.chartBar")}</SelectItem>
                <SelectItem value="line">{t("dataGrid.chartLine")}</SelectItem>
                <SelectItem value="pie">{t("dataGrid.chartPie")}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label className="text-xs">{t("dataGrid.chartXAxis")}</Label>
            <Select value={xColumn} onValueChange={setXColumn}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {columns.map((c) => (
                  <SelectItem key={c.name} value={c.name}>
                    {c.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label className="text-xs">{t("dataGrid.chartYAxis")}</Label>
            <Select value={yColumn} onValueChange={setYColumn}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {columns.map((c) => (
                  <SelectItem key={c.name} value={c.name}>
                    {c.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={onClose} className="text-muted-foreground">
            {t("common.actions.cancel")}
          </Button>
          <Button onClick={handleApply} disabled={!xColumn || !yColumn}>
            {t("common.actions.confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
