import { useCallback, useMemo, useState } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

import {
  useConnect,
  useConnectionList,
  useDeleteConnection,
  useDisconnect,
  useDuplicateConnection,
  useRenameConnection,
  useToggleFavorite,
  useToggleReadonly,
} from "../queries/connection.queries";
import {
  useConnectionModuleStore,
  type ConnectionSortField,
} from "../state/connection.store";
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
  const duplicateMutation = useDuplicateConnection();
  const renameMutation = useRenameConnection();
  const toggleFavoriteMutation = useToggleFavorite();
  const toggleReadonlyMutation = useToggleReadonly();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const connectionErrors = useConnectionModuleStore((s) => s.connectionErrors);
  const favorites = useConnectionModuleStore((s) => s.favorites);
  const sortField = useConnectionModuleStore((s) => s.sortField);
  const sortDirection = useConnectionModuleStore((s) => s.sortDirection);
  const filterTag = useConnectionModuleStore((s) => s.filterTag);
  const filterGroup = useConnectionModuleStore((s) => s.filterGroup);
  const setSortField = useConnectionModuleStore((s) => s.setSortField);
  const setSortDirection = useConnectionModuleStore((s) => s.setSortDirection);
  const setFilterTag = useConnectionModuleStore((s) => s.setFilterTag);
  const setFilterGroup = useConnectionModuleStore((s) => s.setFilterGroup);
  const clearFilters = useConnectionModuleStore((s) => s.clearFilters);

  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const uniqueTags = useMemo(() => {
    if (!connections) return [];
    const tagSet = new Set<string>();
    for (const conn of connections) {
      for (const tag of conn.tags ?? []) tagSet.add(tag);
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

  const filteredAndSorted = useMemo(() => {
    if (!connections) return [];
    const result = connections.filter((conn) => {
      if (filterTag && !(conn.tags ?? []).includes(filterTag)) return false;
      if (filterGroup && conn.group !== filterGroup) return false;
      return true;
    });

    result.sort((a, b) => {
      // Favorites always first
      const aFav = favorites[a.id] ?? a.favorite ?? false;
      const bFav = favorites[b.id] ?? b.favorite ?? false;
      if (aFav && !bFav) return -1;
      if (!aFav && bFav) return 1;

      const dir = sortDirection === "asc" ? 1 : -1;
      switch (sortField) {
        case "name":
          return a.name.localeCompare(b.name) * dir;
        case "driver":
          return a.driver.localeCompare(b.driver) * dir;
        case "group":
          return (a.group ?? "").localeCompare(b.group ?? "") * dir;
        default:
          return 0;
      }
    });

    return result;
  }, [connections, filterTag, filterGroup, favorites, sortField, sortDirection]);

  const handleSortToggle = useCallback(
    (field: ConnectionSortField) => {
      if (sortField === field) {
        setSortDirection(sortDirection === "asc" ? "desc" : "asc");
      } else {
        setSortField(field);
        setSortDirection("asc");
      }
    },
    [sortField, sortDirection, setSortField, setSortDirection],
  );

  const handleStartRename = useCallback((conn: Connection) => {
    setRenamingId(conn.id);
    setRenameValue(conn.name);
  }, []);

  const handleCommitRename = useCallback(
    (id: string) => {
      const trimmed = renameValue.trim();
      if (trimmed) {
        renameMutation.mutate({ id, name: trimmed });
      }
      setRenamingId(null);
    },
    [renameValue, renameMutation],
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--app-text-muted)]">{t("common.states.loading")}</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center gap-2 py-12">
        <p className="text-destructive">{t("common.states.error")}</p>
        <p className="text-sm text-[var(--app-text-muted)]">
          {(error as { userMessage?: string }).userMessage ?? (error as Error).message}
        </p>
      </div>
    );
  }

  if (!connections?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--app-text-muted)]">{t("common.states.empty")}</p>
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
      {/* Sort + filter toolbar */}
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-xs text-[var(--app-text-muted)]">{t("common.actions.sort")}:</span>
        {(["name", "driver", "group"] as ConnectionSortField[]).map((field) => (
          <Button
            key={field}
            type="button"
            variant="ghost"
            size="sm"
            className={cn(
              "h-auto rounded-full px-2 py-0.5 text-xs",
              sortField === field
                ? "bg-primary text-white hover:bg-primary/90"
                : "bg-background text-[var(--app-text-muted)] hover:bg-muted",
            )}
            onClick={() => handleSortToggle(field)}
          >
            {t(`connection.sort.${field}`)}
            {sortField === field && (sortDirection === "asc" ? " ↑" : " ↓")}
          </Button>
        ))}

        {uniqueGroups.length > 0 && (
          <div className="flex items-center gap-1">
            <span className="text-xs text-[var(--app-text-muted)]">
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
                    : "bg-background text-[var(--app-text-muted)] hover:bg-muted",
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
            <span className="text-xs text-[var(--app-text-muted)]">
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
                    : "border-[var(--app-border-subtle)] bg-transparent text-[var(--app-text-muted)]",
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
            className="h-auto px-2 py-0.5 text-xs text-[var(--app-text-muted)]"
            onClick={clearFilters}
          >
            {t("common.actions.clear")}
          </Button>
        )}
      </div>

      {filteredAndSorted.length === 0 ? (
        <div className="flex items-center justify-center py-12">
          <p className="text-[var(--app-text-muted)]">{t("common.states.empty")}</p>
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border border-[var(--app-border)]">
          <Table className="w-full text-sm">
            <TableHeader>
              <TableRow className="bg-muted hover:bg-muted">
                <TableHead className="text-xs font-medium text-[var(--app-text-muted)]">
                  {t("common.labels.name")}
                </TableHead>
                <TableHead className="text-xs font-medium text-[var(--app-text-muted)]">
                  {t("common.labels.host")}
                </TableHead>
                <TableHead className="text-xs font-medium text-[var(--app-text-muted)]">
                  {t("common.labels.database")}
                </TableHead>
                <TableHead className="text-xs font-medium text-[var(--app-text-muted)]">
                  {t("common.labels.driver")}
                </TableHead>
                <TableHead className="text-xs font-medium text-[var(--app-text-muted)]">
                  {t("common.states.status", "Status")}
                </TableHead>
                <TableHead className="text-right text-xs font-medium text-[var(--app-text-muted)]">
                  {t("common.actions.actions", "Actions")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredAndSorted.map((conn) => {
                const status = getStatus(conn);
                const isFav = favorites[conn.id] ?? conn.favorite ?? false;
                return (
                  <ContextMenu key={conn.id}>
                    <ContextMenuTrigger asChild>
                      <>
                        <TableRow
                          className={cn(
                            "cursor-pointer transition-colors hover:bg-muted",
                            explorerConnectionId === conn.id && "bg-muted",
                          )}
                          onClick={() => renamingId !== conn.id && onEdit(conn.id)}
                        >
                          <TableCell className="px-4 py-3">
                            <div className="flex items-center gap-2">
                              <button
                                type="button"
                                className={cn(
                                  "shrink-0 text-sm transition-colors",
                                  isFav
                                    ? "text-warning"
                                    : "text-[var(--app-text-dim)] hover:text-warning",
                                )}
                                onClick={(e) => {
                                  e.stopPropagation();
                                  toggleFavoriteMutation.mutate({
                                    id: conn.id,
                                    favorite: !isFav,
                                  });
                                }}
                                aria-label={t("connection.toggleFavorite")}
                              >
                                {isFav ? "★" : "☆"}
                              </button>
                              {conn.color && (
                                <span
                                  className="inline-block h-3 w-3 shrink-0 rounded-full"
                                  style={{ backgroundColor: conn.color }}
                                />
                              )}
                              {renamingId === conn.id ? (
                                <input
                                  className="h-6 w-full min-w-0 rounded border border-[var(--app-border)] bg-background px-1 text-sm text-foreground outline-none focus:border-primary"
                                  value={renameValue}
                                  onChange={(e) => setRenameValue(e.target.value)}
                                  onBlur={() => handleCommitRename(conn.id)}
                                  onKeyDown={(e) => {
                                    if (e.key === "Enter") handleCommitRename(conn.id);
                                    if (e.key === "Escape") setRenamingId(null);
                                  }}
                                  onClick={(e) => e.stopPropagation()}
                                  autoFocus
                                />
                              ) : (
                                <span className="font-medium text-foreground">
                                  {conn.name}
                                </span>
                              )}
                              {conn.readonly && (
                                <Badge variant="outline" className="text-[11px]">
                                  {t("connection.readonly")}
                                </Badge>
                              )}
                              {(conn.tags ?? []).map((tag) => (
                                <Badge key={tag} variant="outline" className="text-[11px]">
                                  {tag}
                                </Badge>
                              ))}
                            </div>
                          </TableCell>
                          <TableCell className="text-[var(--app-text-muted)]">
                            {conn.driver === "sqlite" ? conn.database : `${conn.host}:${conn.port}`}
                          </TableCell>
                          <TableCell className="text-[var(--app-text-muted)]">
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
                                      className="h-auto px-2 py-1 text-xs text-[var(--app-text-muted)]"
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
                                  disabled={status === "connecting" || status === "reconnecting"}
                                >
                                  {t("common.actions.connect")}
                                </Button>
                              )}
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                className="h-auto px-2 py-1 text-xs text-[var(--app-text-muted)]"
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
                          <TableRow className="border-t border-[var(--app-border-subtle)] bg-destructive/5">
                            <TableCell colSpan={6} className="px-4 py-2 text-xs text-destructive">
                              {connectionErrors[conn.id]}
                            </TableCell>
                          </TableRow>
                        )}
                      </>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                      <ContextMenuItem onClick={() => toggleFavoriteMutation.mutate({ id: conn.id, favorite: !isFav })}>
                        {isFav ? t("connection.unfavorite") : t("connection.favorite")}
                      </ContextMenuItem>
                      <ContextMenuItem onClick={() => handleStartRename(conn)}>
                        {t("connection.rename")}
                      </ContextMenuItem>
                      <ContextMenuItem onClick={() => duplicateMutation.mutate(conn.id)}>
                        {t("connection.duplicate")}
                      </ContextMenuItem>
                      <ContextMenuItem
                        onClick={() =>
                          toggleReadonlyMutation.mutate({ id: conn.id, readonly: !conn.readonly })
                        }
                      >
                        {conn.readonly
                          ? t("connection.disableReadonly")
                          : t("connection.enableReadonly")}
                      </ContextMenuItem>
                      <ContextMenuSeparator />
                      <ContextMenuItem onClick={() => onEdit(conn.id)}>
                        {t("connection.edit")}
                      </ContextMenuItem>
                      <ContextMenuSeparator />
                      <ContextMenuItem
                        variant="destructive"
                        onClick={() => {
                          if (confirm(t("connection.confirmDelete"))) {
                            deleteMutation.mutate(conn.id);
                          }
                        }}
                      >
                        {t("common.actions.delete")}
                      </ContextMenuItem>
                    </ContextMenuContent>
                  </ContextMenu>
                );
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
