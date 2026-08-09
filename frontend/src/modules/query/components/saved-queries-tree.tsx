import { useCallback, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { useTranslation } from "@/commons/locales/useTranslation";

import {
  useCreateFolder,
  useDeleteFolder,
  useDeleteSavedQuery,
  useDuplicateSavedQuery,
  useListFolders,
  useListSavedQueries,
  useRenameSavedQuery,
} from "../queries/query.queries";
import { useSavedQueryMetaStore } from "../stores/saved-query-meta.store";
import type { SavedQuery, SavedQueryFolder } from "../types/query.types";

type SortField = "name" | "created";

interface SavedQueriesTreeProps {
  connectionId: string;
  onSelectQuery: (sql: string) => void;
}

export function SavedQueriesTree({ connectionId, onSelectQuery }: SavedQueriesTreeProps) {
  const { t } = useTranslation();
  const foldersQuery = useListFolders(connectionId);
  const savedQueriesQuery = useListSavedQueries(connectionId);
  const createFolderMutation = useCreateFolder();
  const deleteFolderMutation = useDeleteFolder();
  const deleteSavedMutation = useDeleteSavedQuery();
  const renameSavedMutation = useRenameSavedQuery();
  const duplicateSavedMutation = useDuplicateSavedQuery();

  const toggleFavorite = useSavedQueryMetaStore((s) => s.toggleFavorite);
  const isFavorite = useSavedQueryMetaStore((s) => s.isFavorite);
  const getMeta = useSavedQueryMetaStore((s) => s.getMeta);
  const addTag = useSavedQueryMetaStore((s) => s.addTag);
  const removeTag = useSavedQueryMetaStore((s) => s.removeTag);
  const getAllTags = useSavedQueryMetaStore((s) => s.getAllTags);

  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [newFolderName, setNewFolderName] = useState("");
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);
  const [search, setSearch] = useState("");
  const [sortField, setSortField] = useState<SortField>("name");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [filterTag, setFilterTag] = useState<string | null>(null);
  const [showFavoritesOnly, setShowFavoritesOnly] = useState(false);

  const folders: SavedQueryFolder[] = foldersQuery.data ?? [];
  const queries: SavedQuery[] = savedQueriesQuery.data ?? [];
  const allTags = getAllTags();

  // Enrich queries with frontend meta
  const enrichedQueries = useMemo(() => {
    return queries.map((q) => ({
      ...q,
      favorite: isFavorite(q.id),
      tags: getMeta(q.id).tags,
    }));
  }, [queries, isFavorite, getMeta]);

  // Filter and sort
  const filteredQueries = useMemo(() => {
    let result = enrichedQueries;

    if (search.trim()) {
      const q = search.toLowerCase();
      result = result.filter(
        (sq) =>
          sq.name.toLowerCase().includes(q) ||
          sq.sql.toLowerCase().includes(q) ||
          sq.tags.some((tag) => tag.toLowerCase().includes(q)),
      );
    }

    if (filterTag) {
      result = result.filter((sq) => sq.tags.includes(filterTag));
    }

    if (showFavoritesOnly) {
      result = result.filter((sq) => sq.favorite);
    }

    // Sort: favorites first, then by field
    return [...result].sort((a, b) => {
      if (a.favorite !== b.favorite) return a.favorite ? -1 : 1;
      if (sortField === "name") return a.name.localeCompare(b.name);
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  }, [enrichedQueries, search, filterTag, showFavoritesOnly, sortField]);

  const toggleFolder = useCallback((folderId: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(folderId)) {
        next.delete(folderId);
      } else {
        next.add(folderId);
      }
      return next;
    });
  }, []);

  const handleCreateFolder = useCallback(() => {
    if (!newFolderName.trim()) return;
    createFolderMutation.mutate(
      { connectionId, name: newFolderName.trim() },
      {
        onSuccess: () => {
          setNewFolderName("");
          setShowNewFolderInput(false);
        },
      },
    );
  }, [connectionId, newFolderName, createFolderMutation]);

  const handleDeleteFolder = useCallback(
    (folderId: string) => {
      deleteFolderMutation.mutate({ id: folderId, connectionId });
    },
    [connectionId, deleteFolderMutation],
  );

  const handleDeleteQuery = useCallback(
    (queryId: string) => {
      deleteSavedMutation.mutate({ id: queryId, connectionId });
    },
    [connectionId, deleteSavedMutation],
  );

  const handleRename = useCallback(
    (id: string) => {
      if (!renameValue.trim()) {
        setRenamingId(null);
        return;
      }
      renameSavedMutation.mutate(
        { id, connectionId, newName: renameValue.trim() },
        { onSuccess: () => setRenamingId(null) },
      );
    },
    [connectionId, renameValue, renameSavedMutation],
  );

  const handleDuplicate = useCallback(
    (id: string) => {
      duplicateSavedMutation.mutate({ id, connectionId });
    },
    [connectionId, duplicateSavedMutation],
  );

  const handleToggleFavorite = useCallback(
    (id: string) => {
      toggleFavorite(id);
    },
    [toggleFavorite],
  );

  const handleAddTag = useCallback(
    (id: string, tag: string) => {
      if (tag.trim()) addTag(id, tag.trim());
    },
    [addTag],
  );

  const handleRemoveTag = useCallback(
    (id: string, tag: string) => {
      removeTag(id, tag);
    },
    [removeTag],
  );

  // Group by folder
  const rootQueries = filteredQueries.filter((q) => !q.folder);
  const queriesByFolder = new Map<string, typeof filteredQueries>();
  for (const q of filteredQueries) {
    if (q.folder) {
      const list = queriesByFolder.get(q.folder) ?? [];
      list.push(q);
      queriesByFolder.set(q.folder, list);
    }
  }

  return (
    <div className="flex h-full flex-col overflow-auto text-sm">
      {/* Search & filter toolbar */}
      <div className="border-b border-[var(--app-border-subtle)] p-2">
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder={t("query.searchSavedQueries")}
          className="mb-2 text-xs"
        />
        <div className="flex flex-wrap items-center gap-1">
          <Button
            type="button"
            variant={showFavoritesOnly ? "default" : "ghost"}
            size="sm"
            className="h-6 px-2 text-xs"
            onClick={() => setShowFavoritesOnly(!showFavoritesOnly)}
          >
            {showFavoritesOnly ? "★" : "☆"} {t("query.favorites")}
          </Button>
          <Button
            type="button"
            variant={sortField === "name" ? "default" : "ghost"}
            size="sm"
            className="h-6 px-2 text-xs"
            onClick={() => setSortField(sortField === "name" ? "created" : "name")}
          >
            {t("common.actions.sort")}:{" "}
            {sortField === "name" ? t("query.sort.name") : t("query.sort.created")}
          </Button>
          {allTags.length > 0 && (
            <select
              className="h-6 rounded-sm border border-[var(--app-border)] bg-transparent px-1 text-xs text-foreground"
              value={filterTag ?? ""}
              onChange={(e) => setFilterTag(e.target.value || null)}
            >
              <option value="">{t("query.allTags")}</option>
              {allTags.map((tag) => (
                <option key={tag} value={tag}>
                  {tag}
                </option>
              ))}
            </select>
          )}
        </div>
      </div>

      {/* New folder button */}
      <div className="flex items-center justify-between border-b border-[var(--app-border-subtle)] px-2 py-1">
        <span className="font-medium text-foreground">{t("query.savedQueries")}</span>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => setShowNewFolderInput(!showNewFolderInput)}
          className="rounded-sm px-2 py-0.5 text-xs text-primary"
        >
          + {t("query.newFolder")}
        </Button>
      </div>

      {showNewFolderInput && (
        <div className="flex gap-1 border-b border-[var(--app-border-subtle)] p-2">
          <Input
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
            placeholder={t("query.folderName")}
            className="flex-1 text-xs"
            onKeyDown={(e) => e.key === "Enter" && handleCreateFolder()}
          />
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={handleCreateFolder}
            className="rounded-sm px-2 py-1 text-xs text-primary"
          >
            {t("common.actions.save")}
          </Button>
        </div>
      )}

      <div className="flex-1 overflow-auto p-1">
        {/* Root queries (no folder) */}
        {rootQueries.map((q) => (
          <SavedQueryItem
            key={q.id}
            query={q}
            isRenaming={renamingId === q.id}
            renameValue={renameValue}
            onRenameValueChange={setRenameValue}
            onStartRename={(id, name) => {
              setRenamingId(id);
              setRenameValue(name);
            }}
            onCommitRename={() => handleRename(q.id)}
            onCancelRename={() => setRenamingId(null)}
            onSelect={() => onSelectQuery(q.sql)}
            onDelete={() => handleDeleteQuery(q.id)}
            onToggleFavorite={() => handleToggleFavorite(q.id)}
            onDuplicate={() => handleDuplicate(q.id)}
            onAddTag={(tag) => handleAddTag(q.id, tag)}
            onRemoveTag={(tag) => handleRemoveTag(q.id, tag)}
          />
        ))}

        {/* Folders */}
        {folders.map((folder) => {
          const isExpanded = expandedFolders.has(folder.id);
          const folderQueries = queriesByFolder.get(folder.name) ?? [];

          return (
            <div key={folder.id} className="mb-1">
              <ContextMenu>
                <ContextMenuTrigger asChild>
                  <div className="group flex items-center justify-between rounded-sm px-2 py-1 transition-colors hover:bg-background">
                    <Button
                      type="button"
                      variant="ghost"
                      className="flex flex-1 items-center justify-start gap-1 rounded-none border-0 text-left text-xs font-medium text-foreground"
                      onClick={() => toggleFolder(folder.id)}
                    >
                      <span>{isExpanded ? "▼" : "▶"}</span>
                      <span className="truncate">{folder.name}</span>
                      <span className="ml-auto text-xs text-[var(--app-text-muted)]">
                        ({folderQueries.length})
                      </span>
                    </Button>
                  </div>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem onClick={() => handleDeleteFolder(folder.id)}>
                    {t("common.actions.delete")}
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>

              {isExpanded &&
                folderQueries.map((q) => (
                  <SavedQueryItem
                    key={q.id}
                    query={q}
                    isRenaming={renamingId === q.id}
                    renameValue={renameValue}
                    onRenameValueChange={setRenameValue}
                    onStartRename={(id, name) => {
                      setRenamingId(id);
                      setRenameValue(name);
                    }}
                    onCommitRename={() => handleRename(q.id)}
                    onCancelRename={() => setRenamingId(null)}
                    onSelect={() => onSelectQuery(q.sql)}
                    onDelete={() => handleDeleteQuery(q.id)}
                    onToggleFavorite={() => handleToggleFavorite(q.id)}
                    onDuplicate={() => handleDuplicate(q.id)}
                    onAddTag={(tag) => handleAddTag(q.id, tag)}
                    onRemoveTag={(tag) => handleRemoveTag(q.id, tag)}
                    indent
                  />
                ))}
            </div>
          );
        })}

        {filteredQueries.length === 0 && folders.length === 0 && (
          <div className="py-4 text-center text-xs italic text-[var(--app-text-muted)]">
            {search ? t("query.noMatchingQueries") : t("query.noSavedQueries")}
          </div>
        )}
      </div>
    </div>
  );
}

/* ─── Saved Query Item ──────────────────────────────────────────── */

interface EnrichedSavedQuery extends SavedQuery {
  favorite: boolean;
  tags: string[];
}

interface SavedQueryItemProps {
  query: EnrichedSavedQuery;
  isRenaming: boolean;
  renameValue: string;
  onRenameValueChange: (v: string) => void;
  onStartRename: (id: string, name: string) => void;
  onCommitRename: () => void;
  onCancelRename: () => void;
  onSelect: () => void;
  onDelete: () => void;
  onToggleFavorite: () => void;
  onDuplicate: () => void;
  onAddTag: (tag: string) => void;
  onRemoveTag: (tag: string) => void;
  indent?: boolean;
}

function SavedQueryItem({
  query,
  isRenaming,
  renameValue,
  onRenameValueChange,
  onStartRename,
  onCommitRename,
  onCancelRename,
  onSelect,
  onDelete,
  onToggleFavorite,
  onDuplicate,
  onAddTag,
  onRemoveTag,
  indent,
}: SavedQueryItemProps) {
  const { t } = useTranslation();
  const [showTagInput, setShowTagInput] = useState(false);
  const [tagValue, setTagValue] = useState("");

  const handleAddTag = useCallback(() => {
    if (tagValue.trim()) {
      onAddTag(tagValue.trim());
      setTagValue("");
      setShowTagInput(false);
    }
  }, [tagValue, onAddTag]);

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          className={`group flex flex-col rounded-sm px-2 py-1 transition-colors hover:bg-background ${indent ? "ml-4" : ""}`}
        >
          <div className="flex items-center gap-1">
            {/* Favorite star */}
            <button
              type="button"
              className="shrink-0 text-xs text-[var(--app-text-muted)] hover:text-warning"
              onClick={(e) => {
                e.stopPropagation();
                onToggleFavorite();
              }}
              title={query.favorite ? t("query.unfavorite") : t("query.favorite")}
            >
              {query.favorite ? "★" : "☆"}
            </button>

            {/* Name / rename input */}
            {isRenaming ? (
              <input
                className="flex-1 rounded-sm border border-[var(--app-border)] bg-background px-1 text-xs text-foreground outline-none"
                value={renameValue}
                onChange={(e) => onRenameValueChange(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") onCommitRename();
                  if (e.key === "Escape") onCancelRename();
                }}
                onBlur={onCommitRename}
                autoFocus
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <Button
                type="button"
                variant="ghost"
                className="flex-1 justify-start truncate text-left text-xs text-foreground"
                onClick={onSelect}
                title={query.name}
              >
                {query.name}
              </Button>
            )}
          </div>

          {/* Tags */}
          {query.tags.length > 0 && (
            <div className="ml-5 mt-0.5 flex flex-wrap gap-1">
              {query.tags.map((tag) => (
                <Badge
                  key={tag}
                  variant="secondary"
                  className="cursor-pointer px-1.5 py-0 text-[11px]"
                  onClick={(e) => {
                    e.stopPropagation();
                    onRemoveTag(tag);
                  }}
                  title={t("query.removeTag")}
                >
                  {tag} ×
                </Badge>
              ))}
            </div>
          )}

          {/* Inline tag input */}
          {showTagInput && (
            <div className="ml-5 mt-0.5 flex gap-1">
              <Input
                className="h-5 flex-1 text-[11px]"
                value={tagValue}
                onChange={(e) => setTagValue(e.target.value)}
                placeholder={t("query.addTag")}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleAddTag();
                  if (e.key === "Escape") setShowTagInput(false);
                }}
                autoFocus
              />
            </div>
          )}
        </div>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={onToggleFavorite}>
          {query.favorite ? t("query.unfavorite") : t("query.favorite")}
        </ContextMenuItem>
        <ContextMenuItem onClick={() => onStartRename(query.id, query.name)}>
          {t("query.rename")}
        </ContextMenuItem>
        <ContextMenuItem onClick={onDuplicate}>{t("query.duplicate")}</ContextMenuItem>
        <ContextMenuItem onClick={() => setShowTagInput(true)}>{t("query.addTag")}</ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem className="text-destructive" onClick={onDelete}>
          {t("common.actions.delete")}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
