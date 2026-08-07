import { useEffect, useState } from "react";
import { useNavigate, useSearch } from "@tanstack/react-router";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IConnectionService } from "@/commons/di/registry";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useSnackbar } from "@/app/providers/snackbar.provider";
import { Button } from "@/components/ui/button";

import { ConnectionEditor } from "../components/connection-editor";
import {
  useCreateConnection,
  useTestConnection,
  useUpdateConnection,
} from "../queries/connection.queries";
import type { Connection, ConnectionFormData } from "../types/connection.types";

export function ConnectionEditPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const snackbar = useSnackbar();
  const search = useSearch({ strict: false }) as { id?: string };
  const editId = search?.id;

  const [connection, setConnection] = useState<Connection | null>(null);
  const [loadingConnection, setLoadingConnection] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  const createMutation = useCreateConnection();
  const updateMutation = useUpdateConnection();
  const testMutation = useTestConnection();

  useEffect(() => {
    if (!editId) return;
    setLoadingConnection(true);
    setLoadError(null);
    const service = container.resolve<IConnectionService>(SERVICE_NAMES.CONNECTION_SERVICE);
    service
      .get(editId)
      .then((conn: unknown) => setConnection(conn as Connection | null))
      .catch((err: unknown) => {
        setLoadError((err as { userMessage?: string }).userMessage ?? "Failed to load connection");
      })
      .finally(() => setLoadingConnection(false));
  }, [editId]);

  const handleSubmit = (data: ConnectionFormData, password: string) => {
    const config = { ...data, sshTunnel: data.sshTunnel };

    if (editId) {
      updateMutation.mutate(
        { id: editId, config, password: password || undefined },
        {
          onSuccess: () => {
            snackbar.success("Connection updated");
            navigate({ to: "/connections" });
          },
          onError: (err: unknown) =>
            snackbar.error((err as { userMessage?: string }).userMessage ?? "Failed to update"),
        },
      );
    } else {
      createMutation.mutate(
        { config, password },
        {
          onSuccess: () => {
            snackbar.success("Connection created");
            navigate({ to: "/connections" });
          },
          onError: (err: unknown) =>
            snackbar.error((err as { userMessage?: string }).userMessage ?? "Failed to create"),
        },
      );
    }
  };

  const handleTest = (data: ConnectionFormData, password: string) => {
    testMutation.mutate(
      { config: { ...data, sshTunnel: data.sshTunnel }, password, connectionId: editId ?? undefined },
      {
        onSuccess: () => snackbar.success(t("connection.testSuccess")),
        onError: () => snackbar.error(t("connection.testFailed")),
      },
    );
  };

  if (loadingConnection) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">{t("common.states.loading")}</p>
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="flex flex-col items-center gap-2 py-12">
        <p className="text-destructive">{loadError}</p>
        <Button
          variant="outline"
          onClick={() => navigate({ to: "/connections" })}
        >
          {t("common.actions.close")}
        </Button>
      </div>
    );
  }

  if (editId && !connection) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">{t("error.not_found")}</p>
      </div>
    );
  }

  return (
    <div className="mx-auto h-full w-full max-w-2xl overflow-y-auto p-5">
      <h1 className="mb-5 text-lg font-semibold text-foreground">
        {editId ? t("connection.edit") : t("connection.new")}
      </h1>

      <ConnectionEditor
        isEdit={!!editId}
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
        onCancel={() => navigate({ to: "/connections" })}
        isSubmitting={createMutation.isPending || updateMutation.isPending}
        isTesting={testMutation.isPending}
        testResult={
          testMutation.isSuccess ? "success" : testMutation.isError ? "error" : null
        }
      />
    </div>
  );
}
