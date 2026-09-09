import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useSnackbar } from "@/app/providers/snackbar.provider";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import { useBackupDatabase, useRestoreDatabase } from "../queries/backup.queries";
import type { BackupFormat } from "../types/backup.types";
import type { DriverType } from "@/modules/connection/types/connection.types";

export type BackupDialogMode = "backup" | "restore";

interface BackupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  connectionId: string;
  driver: DriverType;
  mode: BackupDialogMode;
}

function getFileName(mode: BackupDialogMode, driver: DriverType, format: BackupFormat) {
  if (mode === "restore") return driver === "sqlite" ? "database.sqlite" : "database.sql";
  if (driver === "sqlite") return "database.sqlite";
  return format === "custom" ? "database.dump" : "database.sql";
}

function getErrorMessage(error: unknown): string {
  if (error && typeof error === "object" && "userMessage" in error) {
    const userMessage = (error as { userMessage?: unknown }).userMessage;
    if (typeof userMessage === "string" && userMessage) return userMessage;
  }
  return error instanceof Error ? error.message : String(error);
}

export function BackupDialog({
  open: isOpen,
  onOpenChange,
  connectionId,
  driver,
  mode,
}: BackupDialogProps) {
  const { t } = useTranslation();
  const snackbar = useSnackbar();
  const backupMutation = useBackupDatabase();
  const restoreMutation = useRestoreDatabase();
  const [format, setFormat] = useState<BackupFormat>("plain");
  const [path, setPath] = useState("");
  const [confirmRestore, setConfirmRestore] = useState(false);
  const isPending = backupMutation.isPending || restoreMutation.isPending;

  useEffect(() => {
    if (driver === "sqlite") setFormat("plain");
    setPath("");
    setConfirmRestore(false);
  }, [driver, mode, isOpen]);

  const choosePath = async () => {
    try {
      const defaultPath = getFileName(mode, driver, format);
      const extension = driver === "sqlite" ? "sqlite" : format === "custom" ? "dump" : "sql";
      const selected =
        mode === "backup"
          ? await save({
              defaultPath,
              filters: [{ name: "Database backup", extensions: [extension] }],
            })
          : await open({
              defaultPath,
              filters: [{ name: "Database backup", extensions: [extension] }],
              multiple: false,
            });
      if (typeof selected === "string") setPath(selected);
    } catch (error) {
      snackbar.error(getErrorMessage(error));
    }
  };

  const runBackup = async () => {
    if (!path) return;
    try {
      const result = await backupMutation.mutateAsync({
        connectionId,
        outputPath: path,
        format,
      });
      snackbar.success(t("backup.backupSuccess", { size: result.sizeBytes.toLocaleString() }));
      onOpenChange(false);
    } catch (error) {
      snackbar.error(getErrorMessage(error));
    }
  };

  const runRestore = async () => {
    if (!path) return;
    try {
      await restoreMutation.mutateAsync({
        connectionId,
        inputPath: path,
        format,
      });
      snackbar.success(t("backup.restoreSuccess"));
      onOpenChange(false);
    } catch (error) {
      snackbar.error(getErrorMessage(error));
    }
  };

  return (
    <>
      <Dialog open={isOpen} onOpenChange={(nextOpen) => !isPending && onOpenChange(nextOpen)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {t(mode === "backup" ? "backup.title" : "backup.restoreTitle")}
            </DialogTitle>
            <DialogDescription>{t("backup.connectFirst")}</DialogDescription>
          </DialogHeader>

          <div className="grid gap-3">
            <div className="grid gap-1.5">
              <Label htmlFor="backup-format">{t("backup.format")}</Label>
              <Select value={format} onValueChange={(value) => setFormat(value as BackupFormat)}>
                <SelectTrigger id="backup-format" className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="plain">{t("backup.formatPlain")}</SelectItem>
                  <SelectItem value="custom" disabled={driver === "sqlite"}>
                    {t("backup.formatCustom")}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-1.5">
              <Label htmlFor="backup-path">
                {t(mode === "backup" ? "backup.outputPath" : "backup.inputPath")}
              </Label>
              <div className="flex gap-2">
                <Input
                  id="backup-path"
                  value={path}
                  readOnly
                  placeholder={getFileName(mode, driver, format)}
                />
                <Button type="button" variant="outline" onClick={choosePath} disabled={isPending}>
                  {t("backup.chooseFile")}
                </Button>
              </div>
            </div>
            {isPending && (
              <p className="text-sm text-[var(--text-secondary)]">
                {t(mode === "backup" ? "backup.backupInProgress" : "backup.restoreInProgress")}
              </p>
            )}
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isPending}
            >
              {t("common.actions.cancel")}
            </Button>
            <Button
              type="button"
              onClick={() => (mode === "backup" ? void runBackup() : setConfirmRestore(true))}
              disabled={!path || isPending}
              loading={isPending}
            >
              {t(mode === "backup" ? "backup.startBackup" : "backup.startRestore")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={confirmRestore} onOpenChange={setConfirmRestore}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("backup.restoreTitle")}</AlertDialogTitle>
            <AlertDialogDescription>{t("backup.restoreWarning")}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.actions.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void runRestore()}>
              {t("backup.startRestore")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
