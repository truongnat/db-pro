import { useMemo, useState } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

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
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const connectionErrors = useConnectionModuleStore((s) => s.connectionErrors);
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
        <p className="text-muted-foreground">{t("common.states.loading")}</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center gap-2 py-12">
        <p className="text-destructive">{t("common.states.error")}</p>
        <p className="text-sm text-muted-foreground">
          {(error as { userMessage?: string }).userMessage ?? (error as Error).message}
        </p>
      </div>
    );
  }

  if (!connections?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">{t("common.states.empty")}</p>
      </div>
    );
  }

  const getStatus = (conn: Connection) => {
    if (statuses[conn.id]) return statuses[conn.id];
    if (explorerConnectionId === conn.id) return "connected" as const;
    return "disconnected" as const;
  };

  const hasActiveFilters = filterTag || filterGroup;

  return (
    <div className="flex flex-col gap-3">
      {(uniqueTags.length > 0 || uniqueGroups.length > 0) && (
        <div className="flex flex-wrap items-center gap-2">
          {uniqueGroups.length > 0 && (
            <div className="flex items-center gap-1">
              <span className="text-xs text-muted-foreground">
                {t("connection.group")}:
              </span>
              {filterGroup && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-auto px-1.5 py-0.5 text-xs text-destructive"
                  onClick={() => setFilterGroup(null)}
                >
                  ×
                </Button>
              )}
              {uniqueGroups.map((group) => (
                <Button
                  key={group}
                  type="button"
                  variant="ghost"
                  size="sm"
                  className={cn(
                    "h-auto rounded-full px-2 py-0.5 text-xs",
                    filterGroup === group
                      ? "bg-primary text-white hover:bg-primary/90"
                      : "bg-card text-muted-foreground hover:bg-card",
                  )}
                  onClick={() => setFilterGroup(filterGroup === group ? null : group)}
                >
                  {group}
                </Button>
              ))}
            </div>
          )}
          {uniqueTags.length > 0 && (
            <div className="flex items-center gap-1">
              <span className="text-xs text-muted-foreground">
                {t("connection.tags")}:
              </span>
              {filterTag && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-auto px-1.5 py-0.5 text-xs text-destructive"
                  onClick={() => setFilterTag(null)}
                >
                  ×
                </Button>
              )}
              {uniqueTags.map((tag) => (
                <Button
                  key={tag}
                  type="button"
                  variant="ghost"
                  size="sm"
                  className={cn(
                    "h-auto rounded-full border px-2 py-0.5 text-xs",
                    filterTag === tag
                      ? "border-primary bg-primary text-white hover:bg-primary/90"
                      : "border-border bg-transparent text-muted-foreground",
                  )}
                  onClick={() => setFilterTag(filterTag === tag ? null : tag)}
                >
                  {tag}
                </Button>
              ))}
            </div>
          )}
          {hasActiveFilters && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-auto px-2 py-0.5 text-xs text-muted-foreground"
              onClick={() => {
                setFilterTag(null);
                setFilterGroup(null);
              }}
            >
              {t("common.actions.clear")}
            </Button>
          )}
        </div>
      )}

      {filteredConnections.length === 0 ? (
        <div className="flex items-center justify-center py-12">
          <p className="text-muted-foreground">{t("common.states.empty")}</p>
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border border-border">
          <Table className="w-full text-sm">
            <TableHeader>
              <TableRow className="bg-muted hover:bg-muted">
                <TableHead className="text-xs font-medium text-muted-foreground">
                  {t("common.labels.name")}
                </TableHead>
                <TableHead className="text-xs font-medium text-muted-foreground">
                  {t("common.labels.host")}
                </TableHead>
                <TableHead className="text-xs font-medium text-muted-foreground">
                  {t("common.labels.database")}
                </TableHead>
                <TableHead className="text-xs font-medium text-muted-foreground">
                  {t("common.labels.driver")}
                </TableHead>
                <TableHead className="text-xs font-medium text-muted-foreground">
                  {t("common.states.status", "Status")}
                </TableHead>
                <TableHead className="text-right text-xs font-medium text-muted-foreground">
                  {t("common.actions.actions", "Actions")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredConnections.map((conn) => {
                const status = getStatus(conn);
                return (
                  <>
                  <TableRow
                    key={conn.id}
                    className={cn(
                      "cursor-pointer transition-colors hover:bg-muted",
                      explorerConnectionId === conn.id && "bg-muted",
                    )}
                    onClick={() => onEdit(conn.id)}
                  >
                    <TableCell className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        {conn.color && (
                          <span
                            className="inline-block h-3 w-3 shrink-0 rounded-full"
                            style={{ backgroundColor: conn.color }}
                          />
                        )}
                        <span className="font-medium text-foreground">
                          {conn.name}
                        </span>
                        {(conn.tags ?? []).map((tag) => (
                          <Badge key={tag} variant="outline" className="text-[10px]">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {conn.driver === "sqlite" ? conn.database : `${conn.host}:${conn.port}`}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {conn.driver === "sqlite" ? "\u2014" : conn.database}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">{conn.driver}</Badge>
                    </TableCell>
                    <TableCell>
                      <ConnectionStatusBadge status={status} />
                    </TableCell>
                    <TableCell className="px-4 py-3 text-right">
                      <div className="flex justify-end gap-1" onClick={(e) => e.stopPropagation()}>
                        {status === "connected" ? (
                          <>
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              className="h-auto px-2 py-1 text-xs text-destructive"
                              onClick={() => disconnectMutation.mutate(conn.id)}
                              disabled={disconnectMutation.isPending}
                            >
                              {t("common.actions.disconnect")}
                            </Button>
                            {onBackup && (
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                className="h-auto px-2 py-1 text-xs text-primary"
                                onClick={() => onBackup(conn.id)}
                              >
                                {t("backup.title")}
                              </Button>
                            )}
                            {onRestore && (
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                className="h-auto px-2 py-1 text-xs text-muted-foreground"
                                onClick={() => onRestore(conn.id)}
                              >
                                {t("backup.restoreTitle")}
                              </Button>
                            )}
                          </>
                        ) : (
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="h-auto px-2 py-1 text-xs text-primary"
                            onClick={() => connectMutation.mutate(conn.id)}
                            disabled={status === "connecting"}
                          >
                            {t("common.actions.connect")}
                          </Button>
                        )}
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-auto px-2 py-1 text-xs text-muted-foreground"
                          onClick={() => onEdit(conn.id)}
                        >
                          {t("connection.edit")}
                        </Button>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-auto px-2 py-1 text-xs text-destructive"
                          onClick={() => {
                            if (confirm(t("connection.confirmDelete"))) {
                              deleteMutation.mutate(conn.id);
                            }
                          }}
                        >
                          {t("common.actions.delete")}
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                  {connectionErrors[conn.id] && (
                    <TableRow className="border-t border-border bg-destructive/5">
                      <TableCell colSpan={6} className="px-4 py-2 text-xs text-destructive">
                        {connectionErrors[conn.id]}
                      </TableCell>
                    </TableRow>
                  )}
                  </>
                );
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
