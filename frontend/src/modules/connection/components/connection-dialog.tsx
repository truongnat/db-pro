import { useCallback, useEffect, useRef, useState } from "react";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IConnectionService } from "@/commons/di/registry";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useSnackbar } from "@/app/providers/snackbar.provider";
import { useRecentStore } from "@/commons/stores/recent.store";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

import { ConnectionEditor, type SaveIntent } from "./connection-editor";
import {
  useConnect,
  useCreateConnection,
  useTestConnection,
  useUpdateConnection,
} from "../queries/connection.queries";
import type { Connection, ConnectionFormData } from "../types/connection.types";

type LoadState = "idle" | "loading" | "ready" | "not-found" | "error";

export function ConnectionDialog() {
  const { t } = useTranslation();
  const snackbar = useSnackbar();

  const open = useRecentStore((s) => s.connectionDialogOpen);
  const editConnectionId = useRecentStore((s) => s.connectionDialogEditId);
  const closeConnectionDialog = useRecentStore((s) => s.closeConnectionDialog);

  const [connection, setConnection] = useState<Connection | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("idle");
  const [connectError, setConnectError] = useState<string | null>(null);
  const [persistedConnectionId, setPersistedConnectionId] = useState<string | null>(null);
  const loadSeq = useRef(0);

  const createMutation = useCreateConnection();
  const updateMutation = useUpdateConnection();
  const testMutation = useTestConnection();
  const connectMutation = useConnect();

  // Once a brand-new connection has been created, remember its id so a
  // subsequent submit (e.g. retry after a connect failure) updates the
  // same record instead of creating a duplicate.
  const effectiveConnectionId = editConnectionId ?? persistedConnectionId;

  const loadConnection = useCallback(async (id: string) => {
    const seq = ++loadSeq.current;
    setLoadState("loading");
    const service = container.resolve<IConnectionService>(SERVICE_NAMES.CONNECTION_SERVICE);
    try {
      const conn = (await service.get(id)) as Connection | null;
      if (seq !== loadSeq.current) return;
      if (!conn) {
        setLoadState("not-found");
        return;
      }
      setConnection(conn);
      setLoadState("ready");
    } catch {
      if (seq === loadSeq.current) setLoadState("error");
    }
  }, []);

  useEffect(() => {
    loadSeq.current++;
    setConnectError(null);
    if (!open) {
      setConnection(null);
      setLoadState("idle");
      setPersistedConnectionId(null);
      return;
    }
    if (!editConnectionId) {
      setConnection(null);
      setLoadState("ready");
      return;
    }
    loadConnection(editConnectionId);
  }, [open, editConnectionId, loadConnection]);

  const persistAndConnect = (connectionId: string, intent: SaveIntent) => {
    if (intent === "save") {
      closeConnectionDialog();
      return;
    }

    setConnectError(null);
    connectMutation.mutate(connectionId, {
      onSuccess: () => {
        closeConnectionDialog();
      },
      onError: (err: unknown) => {
        setConnectError(
          (err as { userMessage?: string }).userMessage ?? t("connection.connectFailed"),
        );
      },
    });
  };

  const handleSubmit = (data: ConnectionFormData, password: string, intent: SaveIntent) => {
    const config = { ...data, sshTunnel: data.sshTunnel };

    if (effectiveConnectionId) {
      updateMutation.mutate(
        { id: effectiveConnectionId, config, password: password || undefined },
        {
          onSuccess: () => {
            snackbar.success(t("connection.updateSuccess"));
            persistAndConnect(effectiveConnectionId, intent);
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
            setPersistedConnectionId(created.id);
            snackbar.success(t("connection.createSuccess"));
            persistAndConnect(created.id, intent);
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
        connectionId: effectiveConnectionId ?? undefined,
      },
      {
        onSuccess: () => snackbar.success(t("connection.testSuccess")),
        onError: () => snackbar.error(t("connection.testFailed")),
      },
    );
  };

  const isSubmitting = createMutation.isPending || updateMutation.isPending;
  const isConnecting = connectMutation.isPending;

  return (
    <Dialog open={open} onOpenChange={(v) => !v && closeConnectionDialog()}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{editConnectionId ? t("connection.edit") : t("connection.new")}</DialogTitle>
        </DialogHeader>

        <div className="max-h-[70vh] overflow-y-auto">
          {loadState === "loading" && (
            <div className="flex items-center justify-center py-8">
              <p className="text-muted-foreground">{t("common.states.loading")}</p>
            </div>
          )}

          {loadState === "error" && (
            <div className="flex flex-col items-center gap-3 py-8">
              <p className="text-destructive">{t("connection.loadFailed")}</p>
              <div className="flex gap-2">
                <Button variant="outline" onClick={() => loadConnection(effectiveConnectionId!)}>
                  {t("common.actions.retry")}
                </Button>
                <Button variant="outline" onClick={closeConnectionDialog}>
                  {t("common.actions.close")}
                </Button>
              </div>
            </div>
          )}

          {loadState === "not-found" && (
            <div className="flex flex-col items-center gap-3 py-8">
              <p className="text-muted-foreground">{t("connection.notFound")}</p>
              <Button variant="outline" onClick={closeConnectionDialog}>
                {t("common.actions.close")}
              </Button>
            </div>
          )}

          {loadState === "ready" && (
            <ConnectionEditor
              isEdit={!!effectiveConnectionId}
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
              isSubmitting={isSubmitting || isConnecting}
              isTesting={testMutation.isPending}
              isConnecting={isConnecting}
              testResult={
                testMutation.isSuccess ? "success" : testMutation.isError ? "error" : null
              }
              connectError={connectError}
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
