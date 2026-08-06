import { useMemo, useState } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/ui/lib/utils";

import { useConnectionList, useConnect, useDeleteConnection, useDisconnect } from "../queries/connection.queries";
import { useConnectionModuleStore } from "../state/connection.store";
import type { Connection } from "../types/connection.types";
import { ConnectionStatusBadge } from "./connection-status";

interface ConnectionListProps {
  onEdit: (id: string) => void;
  onBackup?: (id: string) => void;
  onRestore?: (id: string) => void;
}

export function ConnectionList({ onEdit, onBackup, onRestore }: ConnectionListProps) {
  const { t } = useTranslation();
  const { data: connections, isLoading, error } = useConnectionList();
  const connectMutation = useConnect();
  const disconnectMutation = useDisconnect();
  const deleteMutation = useDeleteConnection();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const [filterTag, setFilterTag] = useState<string | null>(null);
  const [filterGroup, setFilterGroup] = useState<string | null>(null);

  const uniqueTags = useMemo(() => {
    if (!connections) return [];
    const tagSet = new Set<string>();
    for (const conn of connections) {
      for (const tag of conn.tags ?? []) {
        tagSet.add(tag);
      }
    }
    return Array.from(tagSet).sort();
  }, [connections]);

  const uniqueGroups = useMemo(() => {
    if (!connections) return [];
    const groupSet = new Set<string>();
    for (const conn of connections) {
      if (conn.group) groupSet.add(conn.group);
    }
    return Array.from(groupSet).sort();
  }, [connections]);

  const filteredConnections = useMemo(() => {
    if (!connections) return [];
    return connections.filter((conn) => {
      if (filterTag && !(conn.tags ?? []).includes(filterTag)) return false;
      if (filterGroup && conn.group !== filterGroup) return false;
      return true;
    });
  }, [connections, filterTag, filterGroup]);

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

  const hasActiveFilters = filterTag || filterGroup;

  return (
    <div className="flex flex-col gap-3">
      {(uniqueTags.length > 0 || uniqueGroups.length > 0) && (
        <div className="flex flex-wrap items-center gap-2">
          {uniqueGroups.length > 0 && (
            <div className="flex items-center gap-1">
              <span className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
                {t("connection.group")}:
              </span>
              {filterGroup && (
                <button
                  className="rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-[var(--color-surface)]"
                  style={{ color: "var(--color-error)" }}
                  onClick={() => setFilterGroup(null)}
                >
                  ×
                </button>
              )}
              {uniqueGroups.map((group) => (
                <button
                  key={group}
                  className={cn(
                    "rounded-full px-2 py-0.5 text-xs transition-colors",
                    filterGroup === group && "ring-1",
                  )}
                  style={{
                    backgroundColor: filterGroup === group ? "var(--color-primary,#3b82f6)" : "var(--color-surface)",
                    color: filterGroup === group ? "white" : "var(--color-text-secondary)",
                  }}
                  onClick={() => setFilterGroup(filterGroup === group ? null : group)}
                >
                  {group}
                </button>
              ))}
            </div>
          )}
          {uniqueTags.length > 0 && (
            <div className="flex items-center gap-1">
              <span className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
                {t("connection.tags")}:
              </span>
              {filterTag && (
                <button
                  className="rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-[var(--color-surface)]"
                  style={{ color: "var(--color-error)" }}
                  onClick={() => setFilterTag(null)}
                >
                  ×
                </button>
              )}
              {uniqueTags.map((tag) => (
                <button
                  key={tag}
                  className={cn(
                    "rounded-full border px-2 py-0.5 text-xs transition-colors",
                    filterTag === tag && "ring-1",
                  )}
                  style={{
                    borderColor: "var(--color-border)",
                    backgroundColor: filterTag === tag ? "var(--color-primary,#3b82f6)" : "transparent",
                    color: filterTag === tag ? "white" : "var(--color-text-secondary)",
                  }}
                  onClick={() => setFilterTag(filterTag === tag ? null : tag)}
                >
                  {tag}
                </button>
              ))}
            </div>
          )}
          {hasActiveFilters && (
            <button
              className="rounded px-2 py-0.5 text-xs transition-colors hover:bg-[var(--color-surface)]"
              style={{ color: "var(--color-text-secondary)" }}
              onClick={() => {
                setFilterTag(null);
                setFilterGroup(null);
              }}
            >
              {t("common.actions.clear")}
            </button>
          )}
        </div>
      )}

      {filteredConnections.length === 0 ? (
        <div className="flex items-center justify-center py-12">
          <p style={{ color: "var(--color-text-secondary)" }}>{t("common.states.empty")}</p>
        </div>
      ) : (
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
              {filteredConnections.map((conn) => {
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
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        {conn.color && (
                          <span
                            className="inline-block h-3 w-3 shrink-0 rounded-full"
                            style={{ backgroundColor: conn.color }}
                          />
                        )}
                        <span className="font-medium" style={{ color: "var(--color-text)" }}>
                          {conn.name}
                        </span>
                        {(conn.tags ?? []).map((tag) => (
                          <span
                            key={tag}
                            className="rounded-full border px-1.5 py-0.5 text-[10px]"
                            style={{
                              borderColor: "var(--color-border)",
                              color: "var(--color-text-secondary)",
                            }}
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td style={{ color: "var(--color-text-secondary)" }}>
                      {conn.driver === "sqlite" ? conn.database : `${conn.host}:${conn.port}`}
                    </td>
                    <td style={{ color: "var(--color-text-secondary)" }}>
                      {conn.driver === "sqlite" ? "\u2014" : conn.database}
                    </td>
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
                          <>
                            <button
                              className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                              style={{ color: "var(--color-error)" }}
                              onClick={() => disconnectMutation.mutate(conn.id)}
                              disabled={disconnectMutation.isPending}
                            >
                              {t("common.actions.disconnect")}
                            </button>
                            {onBackup && (
                              <button
                                className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                                style={{ color: "var(--color-primary,#3b82f6)" }}
                                onClick={() => onBackup(conn.id)}
                              >
                                {t("backup.title")}
                              </button>
                            )}
                            {onRestore && (
                              <button
                                className="rounded px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
                                style={{ color: "var(--color-text-secondary)" }}
                                onClick={() => onRestore(conn.id)}
                              >
                                {t("backup.restoreTitle")}
                              </button>
                            )}
                          </>
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
                            if (confirm(t("connection.confirmDelete"))) {
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
      )}
    </div>
  );
}
