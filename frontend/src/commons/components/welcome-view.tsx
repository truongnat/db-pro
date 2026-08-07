import { useTranslation } from "@/commons/locales/useTranslation";
import { useCommandStore } from "@/commons/stores/command.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useConnectionList, useConnect, useDeleteConnection } from "@/modules/connection/queries/connection.queries";
import { ConnectionStatusBadge } from "@/modules/connection/components/connection-status";
import { ConnectionDialog } from "@/modules/connection/components/connection-dialog";
import { Button } from "@/components/ui/button";
import { Plus, Command, Pencil, Trash2, Database } from "lucide-react";
import type { Connection } from "@/modules/connection/types/connection.types";

function getConnectionStatus(
  connectionId: string,
  statuses: Record<string, string>,
): "connected" | "disconnected" | "connecting" | "error" {
  return (statuses[connectionId] as "connected" | "disconnected" | "connecting" | "error") ?? "disconnected";
}

export function WelcomeView() {
  const { t } = useTranslation();
  const { data: connections, isLoading } = useConnectionList();
  const connectMutation = useConnect();
  const deleteMutation = useDeleteConnection();
  const statuses = useConnectionModuleStore((s) => s.statuses);

  const recentConnections = useRecentStore((s) => s.recentConnections);
  const addRecentConnection = useRecentStore((s) => s.addRecentConnection);
  const removeRecentConnection = useRecentStore((s) => s.removeRecentConnection);
  const connectionDialogOpen = useRecentStore((s) => s.connectionDialogOpen);
  const connectionDialogEditId = useRecentStore((s) => s.connectionDialogEditId);
  const openConnectionDialog = useRecentStore((s) => s.openConnectionDialog);
  const closeConnectionDialog = useRecentStore((s) => s.closeConnectionDialog);

  const connectionMap = new Map<string, Connection>();
  if (connections) {
    for (const conn of connections) {
      connectionMap.set(conn.id, conn);
    }
  }

  const recentWithDetails = recentConnections
    .map((rc) => ({
      ...rc,
      connection: connectionMap.get(rc.connectionId),
    }))
    .filter((item) => item.connection != null);

  const handleConnect = (connectionId: string) => {
    connectMutation.mutate(connectionId, {
      onSuccess: () => addRecentConnection(connectionId),
    });
  };

  const handleDelete = (connectionId: string) => {
    if (confirm(t("connection.confirmDelete"))) {
      deleteMutation.mutate(connectionId, {
        onSuccess: () => removeRecentConnection(connectionId),
      });
    }
  };

  const hasConnections = connections != null && connections.length > 0;

  return (
    <div className="flex flex-1 flex-col items-center overflow-y-auto px-6 py-12">
      <div className="w-full max-w-xl space-y-8">
        {/* Header */}
        <div className="text-center">
          <h1 className="text-2xl font-bold text-foreground">{t("welcome.title")}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{t("welcome.subtitle")}</p>
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
            <p className="text-sm text-muted-foreground">{t("common.states.loading")}</p>
          ) : !hasConnections ? (
            <div className="flex flex-col items-center gap-3 rounded-lg border border-dashed border-border py-8">
              <Database className="h-8 w-8 text-muted-foreground" />
              <div className="text-center">
                <p className="text-sm text-muted-foreground">{t("welcome.noConnections")}</p>
                <p className="mt-1 text-xs text-muted-foreground">{t("welcome.createFirstConnection")}</p>
              </div>
            </div>
          ) : recentWithDetails.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("welcome.noRecentConnections")}</p>
          ) : (
            <div className="space-y-1">
              {recentWithDetails.map((item) => {
                const conn = item.connection!;
                const status = getConnectionStatus(conn.id, statuses);
                return (
                  <div
                    key={conn.id}
                    className="group flex items-center gap-3 rounded-lg border border-transparent px-3 py-2 hover:border-border hover:bg-muted/50"
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
                          <span className="truncate text-sm font-medium text-foreground">{conn.name}</span>
                          <ConnectionStatusBadge status={status} />
                        </div>
                        <p className="truncate text-xs text-muted-foreground">
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
                        onClick={() => handleDelete(conn.id)}
                      >
                        <Trash2 className="h-3.5 w-3.5 text-destructive" />
                      </Button>
                    </div>
                  </div>
                );
              })}
              <p className="pt-1 text-xs text-muted-foreground">{t("welcome.connectHint")}</p>
            </div>
          )}
        </div>
      </div>

      {/* Connection Dialog */}
      <ConnectionDialog
        open={connectionDialogOpen}
        onClose={closeConnectionDialog}
        editConnectionId={connectionDialogEditId ?? undefined}
      />
    </div>
  );
}
