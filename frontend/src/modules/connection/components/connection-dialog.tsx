import { useEffect, useState } from "react";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IConnectionService } from "@/commons/di/registry";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useSnackbar } from "@/app/providers/snackbar.provider";
import { useRecentStore } from "@/commons/stores/recent.store";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

import { ConnectionEditor } from "./connection-editor";
import {
  useConnect,
  useCreateConnection,
  useTestConnection,
  useUpdateConnection,
} from "../queries/connection.queries";
import type { Connection, ConnectionFormData } from "../types/connection.types";

export function ConnectionDialog() {
  const { t } = useTranslation();
  const snackbar = useSnackbar();

  const open = useRecentStore((s) => s.connectionDialogOpen);
  const editConnectionId = useRecentStore((s) => s.connectionDialogEditId);
  const closeConnectionDialog = useRecentStore((s) => s.closeConnectionDialog);

  const [connection, setConnection] = useState<Connection | null>(null);
  const [loadingConnection, setLoadingConnection] = useState(false);

  const createMutation = useCreateConnection();
  const updateMutation = useUpdateConnection();
  const testMutation = useTestConnection();
  const connectMutation = useConnect();

  useEffect(() => {
    if (!open || !editConnectionId) {
      setConnection(null);
      return;
    }
    setLoadingConnection(true);
    const service = container.resolve<IConnectionService>(SERVICE_NAMES.CONNECTION_SERVICE);
    service
      .get(editConnectionId)
      .then((conn: unknown) => setConnection(conn as Connection | null))
      .finally(() => setLoadingConnection(false));
  }, [open, editConnectionId]);

  useEffect(() => {
    if (!open) {
      setConnection(null);
      setLoadingConnection(false);
    }
  }, [open]);

  const connectAfterSave = (connectionId: string) => {
    connectMutation.mutate(connectionId, {
      onError: (err: unknown) =>
        snackbar.error((err as { userMessage?: string }).userMessage ?? t("connection.connectFailed")),
    });
  };

  const handleSubmit = (data: ConnectionFormData, password: string) => {
    const config = { ...data, sshTunnel: data.sshTunnel };

    if (editConnectionId) {
      updateMutation.mutate(
        { id: editConnectionId, config, password: password || undefined },
        {
          onSuccess: () => {
            snackbar.success(t("connection.updateSuccess"));
            closeConnectionDialog();
            connectAfterSave(editConnectionId);
          },
          onError: (err: unknown) =>
            snackbar.error((err as { userMessage?: string }).userMessage ?? t("connection.updateFailed")),
        },
      );
    } else {
      createMutation.mutate(
        { config, password },
        {
          onSuccess: (created) => {
            snackbar.success(t("connection.createSuccess"));
            closeConnectionDialog();
            connectAfterSave((created as Connection).id);
          },
          onError: (err: unknown) =>
            snackbar.error((err as { userMessage?: string }).userMessage ?? t("connection.createFailed")),
        },
      );
    }
  };

  const handleTest = (data: ConnectionFormData, password: string) => {
    testMutation.mutate(
      {
        config: { ...data, sshTunnel: data.sshTunnel },
        password,
        connectionId: editConnectionId ?? undefined,
      },
      {
        onSuccess: () => snackbar.success(t("connection.testSuccess")),
        onError: () => snackbar.error(t("connection.testFailed")),
      },
    );
  };

  const title = editConnectionId ? t("connection.edit") : t("connection.new");

  return (
    <Dialog open={open} onOpenChange={(v) => !v && closeConnectionDialog()}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>

        <div className="max-h-[70vh] overflow-y-auto">
          {loadingConnection ? (
            <div className="flex items-center justify-center py-8">
              <p className="text-muted-foreground">{t("common.states.loading")}</p>
            </div>
          ) : (
            <ConnectionEditor
              isEdit={!!editConnectionId}
              initialData={
                connection
                  ? {
                      name: connection.name,
                      host: connection.host,
                      port: connection.port,
                      database: connection.database,
                      username: connection.username,
                      driver: connection.driver,
                      sslMode: connection.sslMode,
                      sshTunnel: connection.sshTunnel,
                      queryTimeoutMs: connection.queryTimeoutMs ?? 30000,
                      maxRows: connection.maxRows ?? 500,
                    }
                  : undefined
              }
              onSubmit={handleSubmit}
              onTest={handleTest}
              onCancel={closeConnectionDialog}
              isSubmitting={createMutation.isPending || updateMutation.isPending}
              isTesting={testMutation.isPending}
              testResult={
                testMutation.isSuccess ? "success" : testMutation.isError ? "error" : null
              }
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
