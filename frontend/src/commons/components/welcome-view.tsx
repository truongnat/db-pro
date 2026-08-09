import { useEffect, useMemo, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useCommandStore } from "@/commons/stores/command.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useShellStore } from "@/commons/stores/shell.store";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import {
  useConnectionList,
  useConnect,
  useDeleteConnection,
} from "@/modules/connection/queries/connection.queries";
import { ConnectionStatusBadge } from "@/modules/connection/components/connection-status";
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
import { Button } from "@/components/ui/button";
import { useSnackbar } from "@/app/providers/snackbar.provider";
import { Plus, Command, Pencil, Trash2, Database } from "lucide-react";
import type { Connection } from "@/modules/connection/types/connection.types";

function getConnectionStatus(
  connectionId: string,
  statuses: Record<string, string>,
): "connected" | "disconnected" | "connecting" | "error" {
  return (
    (statuses[connectionId] as "connected" | "disconnected" | "connecting" | "error") ??
    "disconnected"
  );
}

export function WelcomeView() {
  const { t } = useTranslation();
  const snackbar = useSnackbar();
  const { data: connections, isLoading } = useConnectionList();
  const connectMutation = useConnect();
  const deleteMutation = useDeleteConnection();
  const statuses = useConnectionModuleStore((s) => s.statuses);

  const recentConnections = useRecentStore((s) => s.recentConnections);
  const removeRecentConnection = useRecentStore((s) => s.removeRecentConnection);
  const openConnectionDialog = useRecentStore((s) => s.openConnectionDialog);

  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  const connectionMap = useMemo(() => {
    const map = new Map<string, Connection>();
    if (connections) {
      for (const conn of connections) {
        map.set(conn.id, conn);
      }
    }
    return map;
  }, [connections]);

  useEffect(() => {
    if (!connections) return;
    const validIds = new Set(connections.map((c) => c.id));
    for (const rc of recentConnections) {
      if (!validIds.has(rc.connectionId)) {
        removeRecentConnection(rc.connectionId);
      }
    }
  }, [connections, recentConnections, removeRecentConnection]);

  const recentWithDetails = recentConnections
    .map((rc) => ({
      ...rc,
      connection: connectionMap.get(rc.connectionId),
    }))
    .filter((item) => item.connection != null);

  const handleConnect = (connectionId: string) => {
    const status = (statuses[connectionId] as string) ?? "disconnected";

    // Already connected → focus in Explorer instead of reconnecting.
    if (status === "connected") {
      useConnectionStore.getState().setExplorerConnection(connectionId);
      useShellStore.getState().setSidebarView("explorer");
      useExplorerStore.getState().expandNode(`conn:${connectionId}`);
      return;
    }

    // Connecting / reconnecting → ignore.
    if (status === "connecting" || status === "reconnecting") return;

    connectMutation.mutate(connectionId, {
      onError: (err: unknown) =>
        snackbar.error(
          (err as { userMessage?: string }).userMessage ?? t("connection.connectFailed"),
        ),
    });
  };

  const handleDeleteConfirmed = () => {
    if (!deleteConfirmId) return;
    const id = deleteConfirmId;
    setDeleteConfirmId(null);
    deleteMutation.mutate(id, {
      onSuccess: () => removeRecentConnection(id),
    });
  };

  const hasConnections = connections != null && connections.length > 0;

  return (
    <div className="flex flex-1 flex-col items-center overflow-y-auto px-6 py-16">
      <div className="w-full max-w-xl space-y-8">
        {/* Header */}
        <div className="text-center">
          <h1 className="text-xl font-semibold text-foreground">{t("welcome.title")}</h1>
          <p className="mt-1.5 text-sm text-[var(--app-text-muted)]">{t("welcome.subtitle")}</p>
        </div>

        {/* Quick Actions */}
        <div className="flex justify-center gap-3">
          <Button onClick={() => openConnectionDialog()}>
            <Plus className="mr-2 h-4 w-4" />
            {t("welcome.newConnection")}
          </Button>
          <Button variant="outline" onClick={() => useCommandStore.getState().open()}>
            <Command className="mr-2 h-4 w-4" />
            {t("welcome.openCommandPalette")}
          </Button>
        </div>

        {/* Recent Connections */}
        <div className="space-y-3">
          <h2 className="text-sm font-medium text-foreground">{t("welcome.recentConnections")}</h2>

          {isLoading ? (
            <p className="text-sm text-[var(--app-text-muted)]">{t("common.states.loading")}</p>
          ) : !hasConnections ? (
            <div className="flex flex-col items-center gap-3 rounded-lg border border-dashed border-[var(--app-border-strong)] py-10">
              <Database className="h-8 w-8 text-[var(--app-text-dim)]" />
              <div className="text-center">
                <p className="text-sm text-[var(--app-text-muted)]">{t("welcome.noConnections")}</p>
                <p className="mt-1 text-xs text-[var(--app-text-muted)]">
                  {t("welcome.createFirstConnection")}
                </p>
              </div>
            </div>
          ) : recentWithDetails.length === 0 ? (
            <p className="text-sm text-[var(--app-text-muted)]">
              {t("welcome.noRecentConnections")}
            </p>
          ) : (
            <div className="space-y-1">
              {recentWithDetails.map((item) => {
                const conn = item.connection!;
                const status = getConnectionStatus(conn.id, statuses);
                return (
                  <div
                    key={conn.id}
                    className="group flex items-center gap-3 rounded-lg border border-transparent px-3 py-2 transition-colors hover:border-[var(--app-border)] hover:bg-[var(--app-hover)]"
                  >
                    <button
                      type="button"
                      className="flex min-w-0 flex-1 items-center gap-3 text-left"
                      onClick={() => handleConnect(conn.id)}
                    >
                      <div
                        className="h-2.5 w-2.5 shrink-0 rounded-full"
                        style={{ backgroundColor: conn.color ?? "var(--foreground)" }}
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <span className="truncate text-sm font-medium text-foreground">
                            {conn.name}
                          </span>
                          <ConnectionStatusBadge status={status} />
                        </div>
                        <p className="truncate text-xs text-[var(--app-text-muted)]">
                          {conn.host}:{conn.port} / {conn.database}
                        </p>
                      </div>
                    </button>
                    <div className="flex shrink-0 items-center gap-1 opacity-0 group-hover:opacity-100">
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        onClick={() => openConnectionDialog(conn.id)}
                      >
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        onClick={() => setDeleteConfirmId(conn.id)}
                      >
                        <Trash2 className="h-3.5 w-3.5 text-destructive" />
                      </Button>
                    </div>
                  </div>
                );
              })}
              <p className="pt-1 text-xs text-[var(--app-text-muted)]">
                {t("welcome.connectHint")}
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Delete Confirmation Dialog */}
      <AlertDialog
        open={deleteConfirmId != null}
        onOpenChange={(open) => !open && setDeleteConfirmId(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("connection.confirmDelete")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("connection.confirmDeleteDescription")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.actions.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-white hover:bg-destructive/90"
              onClick={handleDeleteConfirmed}
            >
              {t("common.actions.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
