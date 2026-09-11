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
import { ArrowRight, Command, Database, Pencil, Plus, Trash2 } from "lucide-react";
import type { Connection } from "@/modules/connection/types/connection.types";
import { isMac } from "@/commons/utils/platform";

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
    return new Map((connections ?? []).map((connection) => [connection.id, connection]));
  }, [connections]);

  useEffect(() => {
    if (!connections) return;
    const validIds = new Set(connections.map((connection) => connection.id));
    for (const recent of recentConnections) {
      if (!validIds.has(recent.connectionId)) removeRecentConnection(recent.connectionId);
    }
  }, [connections, recentConnections, removeRecentConnection]);

  const recentWithDetails = recentConnections
    .map((recent) => ({ ...recent, connection: connectionMap.get(recent.connectionId) }))
    .filter((item): item is typeof item & { connection: Connection } => item.connection != null);

  // A saved connection should never become unreachable merely because it has not
  // been opened recently. Fall back to the full list for first-time/returning users.
  const displayedConnections =
    recentWithDetails.length > 0
      ? recentWithDetails.map((item) => item.connection)
      : (connections ?? []);

  const handleConnect = (connectionId: string) => {
    const status = statuses[connectionId] ?? "disconnected";
    if (status === "connected") {
      useConnectionStore.getState().setExplorerConnection(connectionId);
      useShellStore.getState().setSidebarView("explorer");
      useExplorerStore.getState().expandNode(`conn:${connectionId}`);
      return;
    }
    if (status === "connecting" || status === "reconnecting") return;

    connectMutation.mutate(connectionId, {
      onError: (error: unknown) =>
        snackbar.error(
          (error as { userMessage?: string }).userMessage ?? t("connection.connectFailed"),
        ),
    });
  };

  const handleDeleteConfirmed = () => {
    if (!deleteConfirmId) return;
    const id = deleteConfirmId;
    setDeleteConfirmId(null);
    deleteMutation.mutate(id, { onSuccess: () => removeRecentConnection(id) });
  };

  const hasConnections = connections != null && connections.length > 0;

  return (
    <div className="relative flex flex-1 overflow-y-auto bg-[var(--surface-editor)]">
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-64 opacity-70"
        aria-hidden="true"
        style={{
          background: "radial-gradient(ellipse at 50% -25%, var(--accent-soft), transparent 68%)",
        }}
      />

      <div className="relative mx-auto flex w-full max-w-3xl flex-col px-6 py-10 sm:px-10 sm:py-14">
        <header className="flex flex-col items-center text-center">
          <div className="mb-5 grid h-14 w-14 place-items-center rounded-2xl border border-[var(--border-default)] bg-[var(--surface-floating)] shadow-[var(--elevation-popover)]">
            <img src="/brand/db-pro-logo.svg" alt="" className="h-11 w-11" aria-hidden="true" />
          </div>
          <p className="mb-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-primary">
            {t("welcome.eyebrow")}
          </p>
          <h1 className="text-2xl font-semibold tracking-[-0.025em] text-foreground">
            {t("welcome.title")}
          </h1>
          <p className="mt-2 max-w-md text-sm leading-6 text-[var(--text-secondary)]">
            {t("welcome.subtitle")}
          </p>

          <div className="mt-6 flex flex-wrap items-center justify-center gap-2.5">
            <Button size="lg" className="px-4 shadow-sm" onClick={() => openConnectionDialog()}>
              <Plus className="h-4 w-4" />
              {t("welcome.newConnection")}
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="px-4"
              onClick={() => useCommandStore.getState().open()}
            >
              <Command className="h-4 w-4" />
              {t("welcome.openCommandPalette")}
              <kbd className="ml-1 rounded border border-[var(--border-default)] bg-[var(--surface-hover)] px-1.5 py-0.5 font-sans text-[11px] font-medium text-[var(--text-tertiary)]">
                {isMac ? "⌘K" : "Ctrl K"}
              </kbd>
            </Button>
          </div>
        </header>

        <section className="mt-10" aria-labelledby="welcome-connections-title">
          <div className="mb-3 flex items-end justify-between gap-4">
            <div>
              <h2 id="welcome-connections-title" className="text-sm font-semibold text-foreground">
                {recentWithDetails.length > 0
                  ? t("welcome.recentConnections")
                  : t("welcome.connections")}
              </h2>
              {hasConnections && (
                <p className="mt-0.5 text-xs text-[var(--text-tertiary)]">
                  {t("welcome.connectHint")}
                </p>
              )}
            </div>
            {hasConnections && (
              <span className="rounded-full bg-[var(--surface-hover)] px-2 py-0.5 text-[11px] tabular-nums text-[var(--text-tertiary)]">
                {connections.length}
              </span>
            )}
          </div>

          <div className="overflow-hidden rounded-xl border border-[var(--border-default)] bg-[var(--surface-floating)] shadow-[0_1px_2px_rgba(0,0,0,0.03)]">
            {isLoading ? (
              <div className="space-y-3 p-4" aria-label={t("common.states.loading")}>
                <span className="sr-only">{t("common.states.loading")}</span>
                {[0, 1, 2].map((item) => (
                  <div key={item} className="flex animate-pulse items-center gap-3">
                    <div className="h-9 w-9 rounded-lg bg-[var(--surface-active)]" />
                    <div className="flex-1 space-y-2">
                      <div className="h-3 w-32 rounded bg-[var(--surface-active)]" />
                      <div className="h-2.5 w-48 rounded bg-[var(--surface-hover)]" />
                    </div>
                  </div>
                ))}
              </div>
            ) : !hasConnections ? (
              <div className="flex flex-col items-center px-6 py-9 text-center">
                <div className="mb-3 grid h-10 w-10 place-items-center rounded-xl bg-[var(--accent-soft)] text-primary">
                  <Database className="h-5 w-5" />
                </div>
                <p className="text-sm font-medium text-foreground">{t("welcome.noConnections")}</p>
                <p className="mt-1 max-w-xs text-xs leading-5 text-[var(--text-secondary)]">
                  {t("welcome.createFirstConnection")}
                </p>
              </div>
            ) : (
              <div className="divide-y divide-[var(--border-subtle)]">
                {displayedConnections.map((connection) => {
                  const status = getConnectionStatus(connection.id, statuses);
                  const isConnecting = status === "connecting";
                  return (
                    <div
                      key={connection.id}
                      className="group flex items-center gap-3 px-3 py-2.5 transition-colors hover:bg-[var(--surface-hover)] focus-within:bg-[var(--surface-hover)]"
                    >
                      <button
                        type="button"
                        className="flex min-w-0 flex-1 items-center gap-3 rounded-md text-left focus-visible:outline-offset-4"
                        onClick={() => handleConnect(connection.id)}
                        disabled={isConnecting}
                      >
                        <span
                          className="grid h-9 w-9 shrink-0 place-items-center rounded-lg bg-[var(--surface-hover)]"
                          aria-hidden="true"
                        >
                          <Database
                            className="h-4 w-4"
                            style={{ color: connection.color ?? "var(--text-secondary)" }}
                          />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="flex items-center gap-2">
                            <span className="truncate text-sm font-medium text-foreground">
                              {connection.name}
                            </span>
                            <ConnectionStatusBadge status={status} />
                          </span>
                          <span className="mt-0.5 block truncate text-xs text-[var(--text-tertiary)]">
                            {connection.driver === "sqlite"
                              ? connection.database
                              : `${connection.host}:${connection.port} / ${connection.database}`}
                          </span>
                        </span>
                        <ArrowRight className="h-4 w-4 shrink-0 text-[var(--text-tertiary)] opacity-0 transition-all group-hover:translate-x-0.5 group-hover:opacity-100 group-focus-within:opacity-100" />
                      </button>

                      <div className="flex shrink-0 items-center gap-0.5 opacity-60 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          aria-label={`${t("connection.edit")}: ${connection.name}`}
                          onClick={() => openConnectionDialog(connection.id)}
                        >
                          <Pencil className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          aria-label={`${t("common.actions.delete")}: ${connection.name}`}
                          onClick={() => setDeleteConfirmId(connection.id)}
                        >
                          <Trash2 className="h-3.5 w-3.5 text-destructive" />
                        </Button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </section>
      </div>

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
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
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
