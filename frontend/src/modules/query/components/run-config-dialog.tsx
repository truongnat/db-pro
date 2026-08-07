import { useCallback, useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useSaveRunConfig } from "../queries/query.queries";

interface RunConfigDialogProps {
  open: boolean;
  onClose: () => void;
  connectionId: string;
  defaultName?: string;
  defaultSql?: string;
}

export function RunConfigDialog({
  open,
  onClose,
  connectionId,
  defaultName = "",
  defaultSql = "",
}: RunConfigDialogProps) {
  const { t } = useTranslation();
  const saveMutation = useSaveRunConfig();

  const [name, setName] = useState(defaultName);
  const [sql, setSql] = useState(defaultSql);
  const [timeoutMs, setTimeoutMs] = useState(30000);
  const [maxRows, setMaxRows] = useState(1000);

  useEffect(() => {
    if (open) {
      setName(defaultName);
      setSql(defaultSql);
      setTimeoutMs(30000);
      setMaxRows(1000);
    }
  }, [open, defaultName, defaultSql]);

  const handleSave = useCallback(() => {
    if (!name.trim() || !sql.trim()) return;
    saveMutation.mutate(
      {
        connectionId,
        name: name.trim(),
        sql: sql.trim(),
        timeoutMs,
        maxRows,
      },
      { onSuccess: () => onClose() },
    );
  }, [connectionId, name, sql, timeoutMs, maxRows, saveMutation, onClose]);

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-[480px]">
        <DialogHeader>
          <DialogTitle>{t("query.newRunConfig")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="run-config-name">{t("query.configName")}</Label>
            <Input
              id="run-config-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="run-config-sql">SQL</Label>
            <Textarea
              id="run-config-sql"
              value={sql}
              onChange={(e) => setSql(e.target.value)}
              rows={4}
              className="font-mono text-xs"
            />
          </div>

          <div className="flex gap-3">
            <div className="flex-1 space-y-1.5">
              <Label htmlFor="run-config-timeout">{t("query.timeoutMs")}</Label>
              <Input
                id="run-config-timeout"
                type="number"
                value={timeoutMs}
                onChange={(e) => setTimeoutMs(Number(e.target.value))}
                min={1000}
                step={1000}
              />
            </div>
            <div className="flex-1 space-y-1.5">
              <Label htmlFor="run-config-maxrows">{t("query.maxRows")}</Label>
              <Input
                id="run-config-maxrows"
                type="number"
                value={maxRows}
                onChange={(e) => setMaxRows(Number(e.target.value))}
                min={1}
              />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            {t("common.actions.cancel")}
          </Button>
          <Button
            onClick={handleSave}
            disabled={!name.trim() || !sql.trim() || saveMutation.isPending}
          >
            {t("common.actions.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
