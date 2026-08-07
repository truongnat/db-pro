import { useState } from "react";
import { useTranslation } from "react-i18next";

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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import type { BackupFormat } from "../types/backup.types";

interface BackupDialogProps {
  open: boolean;
  connectionId: string;
  onClose: () => void;
  onBackup: (outputPath: string, format: BackupFormat) => void;
  isPending: boolean;
}

export function BackupDialog({
  open,
  connectionId: _connectionId,
  onClose,
  onBackup,
  isPending,
}: BackupDialogProps) {
  const { t } = useTranslation();
  const [outputPath, setOutputPath] = useState("");
  const [format, setFormat] = useState<BackupFormat>("plain");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!outputPath.trim()) return;
    onBackup(outputPath.trim(), format);
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-[28rem]">
        <DialogHeader>
          <DialogTitle>{t("backup.title")}</DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <Label>{t("backup.outputPath")}</Label>
            <Input
              value={outputPath}
              onChange={(e) => setOutputPath(e.target.value)}
              placeholder="/path/to/backup.sql"
              autoFocus
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <Label>{t("backup.format")}</Label>
            <Select value={format} onValueChange={(v) => setFormat(v as BackupFormat)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="plain">{t("backup.formatPlain")}</SelectItem>
                <SelectItem value="custom">{t("backup.formatCustom")}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={onClose}>
              {t("common.actions.cancel")}
            </Button>
            <Button type="submit" disabled={!outputPath.trim() || isPending}>
              {isPending ? t("backup.backupInProgress") : t("backup.startBackup")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
