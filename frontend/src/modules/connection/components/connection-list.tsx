import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/ui/lib/utils";

import { useConnectionList, useConnect, useDeleteConnection, useDisconnect } from "../queries/connection.queries";
import { useConnectionModuleStore } from "../state/connection.store";
import type { Connection } from "../types/connection.types";
import { ConnectionStatusBadge } from "./connection-status";

export function ConnectionList({ onEdit }: { onEdit: (id: string) => void }) {
  const { t } = useTranslation();
  const { data: connections, isLoading, error } = useConnectionList();
  const connectMutation = useConnect();
  const disconnectMutation = useDisconnect();
  const deleteMutation = useDeleteConnection();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);
  const statuses = useConnectionModuleStore((s) => s.statuses);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>{t("common.states.loading")}</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center gap-2 py-12">
        <p style={{ color: "var(--color-error)" }}>{t("common.states.error")}</p>
        <p className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
          {(error as { userMessage?: string }).userMessage ?? (error as Error).message}
        </p>
      </div>
    );
  }

  if (!connections?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>{t("common.states.empty")}</p>
      </div>
    );
  }

  const getStatus = (conn: Connection) => {
    if (statuses[conn.id]) return statuses[conn.id];
    if (activeConnectionId === conn.id) return "connected" as const;
    return "disconnected" as const;
  };

  return (
    <div
      className="overflow-hidden rounded-[var(--radius-md)] border"
      style={{ borderColor: "var(--color-border)" }}
    >
      <table className="w-full text-sm">
        <thead>
          <tr style={{ backgroundColor: "var(--color-surface)" }}>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.labels.name")}
            </th>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.labels.host")}
            </th>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.labels.database")}
            </th>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.labels.driver")}
            </th>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.states.status", "Status")}
            </th>
            <th className="px-4 py-3 text-right font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.actions.actions", "Actions")}
            </th>
          </tr>
        </thead>
        <tbody>
          {connections.map((conn) => {
            const status = getStatus(conn);
            return (
              <tr
                key={conn.id}
                className={cn(
                  "cursor-pointer border-t transition-colors hover:bg-[var(--color-surface)]",
                  activeConnectionId === conn.id && "bg-[var(--color-surface)]",
                )}
                style={{ borderColor: "var(--color-border)" }}
                onClick={() => onEdit(conn.id)}
              >
                <td className="px-4 py-3 font-medium" style={{ color: "var(--color-text)" }}>
                  {conn.name}
                </td>
                <td style={{ color: "var(--color-text-secondary)" }}>
                  {conn.host}:{conn.port}
                </td>
                <td style={{ color: "var(--color-text-secondary)" }}>{conn.database}</td>
                <td>
                  <span
                    className="rounded px-1.5 py-0.5 text-xs"
                    style={{
                      backgroundColor: "var(--color-bg)",
                      color: "var(--color-text-secondary)",
                    }}
                  >
                    {conn.driver}
                  </span>
                </td>
                <td>
                  <ConnectionStatusBadge status={status} />
                </td>
                <td className="px-4 py-3 text-right">
                  <div className="flex justify-end gap-1" onClick={(e) => e.stopPropagation()}>
                    {status === "connected" ? (
                      <button
                        className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                        style={{ color: "var(--color-error)" }}
                        onClick={() => disconnectMutation.mutate(conn.id)}
                        disabled={disconnectMutation.isPending}
                      >
                        {t("common.actions.disconnect")}
                      </button>
                    ) : (
                      <button
                        className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                        style={{ color: "var(--color-primary,#3b82f6)" }}
                        onClick={() => connectMutation.mutate(conn.id)}
                        disabled={status === "connecting"}
                      >
                        {t("common.actions.connect")}
                      </button>
                    )}
                    <button
                      className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                      style={{ color: "var(--color-text-secondary)" }}
                      onClick={() => onEdit(conn.id)}
                    >
                      {t("connection.edit")}
                    </button>
                    <button
                      className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                      style={{ color: "var(--color-error)" }}
                      onClick={() => {
                        if (confirm("Delete this connection?")) {
                          deleteMutation.mutate(conn.id);
                        }
                      }}
                    >
                      {t("common.actions.delete")}
                    </button>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
