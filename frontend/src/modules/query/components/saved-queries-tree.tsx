import { useCallback, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTranslation } from "@/commons/locales/useTranslation";

import {
  useCreateFolder,
  useDeleteFolder,
  useDeleteSavedQuery,
  useListFolders,
  useListSavedQueries,
} from "../queries/query.queries";
import type { SavedQuery, SavedQueryFolder } from "../types/query.types";

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

  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [newFolderName, setNewFolderName] = useState("");
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);

  const folders: SavedQueryFolder[] = foldersQuery.data ?? [];
  const queries: SavedQuery[] = savedQueriesQuery.data ?? [];

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

  const rootQueries = queries.filter((q) => !q.folder);
  const queriesByFolder = new Map<string, SavedQuery[]>();
  for (const q of queries) {
    if (q.folder) {
      const list = queriesByFolder.get(q.folder) ?? [];
      list.push(q);
      queriesByFolder.set(q.folder, list);
    }
  }

  return (
    <div className="flex h-full flex-col overflow-auto p-2 text-sm">
      <div className="mb-2 flex items-center justify-between">
        <span className="font-medium text-foreground">
          {t("query.savedQueries")}
        </span>
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
        <div className="mb-2 flex gap-1">
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

      {rootQueries.length > 0 && (
        <div className="mb-2">
          {rootQueries.map((q) => (
            <div
              key={q.id}
              className="group flex items-center justify-between rounded-sm px-2 py-1 transition-colors hover:bg-background"
            >
              <Button
                type="button"
                variant="ghost"
                className="flex-1 justify-start truncate text-left text-xs text-foreground"
                onClick={() => onSelectQuery(q.sql)}
                title={q.name}
              >
                {q.name}
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="ml-1 text-destructive opacity-0 transition-opacity group-hover:opacity-100"
                onClick={() => handleDeleteQuery(q.id)}
                title={t("common.actions.delete")}
              >
                ×
              </Button>
            </div>
          ))}
        </div>
      )}

      {folders.map((folder) => {
        const isExpanded = expandedFolders.has(folder.id);
        const folderQueries = queriesByFolder.get(folder.name) ?? [];

        return (
          <div key={folder.id} className="mb-1">
            <div className="group flex items-center justify-between rounded-sm px-2 py-1 transition-colors hover:bg-background">
              <Button
                type="button"
                variant="ghost"
                className="flex flex-1 items-center justify-start gap-1 rounded-none border-0 text-left text-xs font-medium text-foreground"
                onClick={() => toggleFolder(folder.id)}
              >
                <span>{isExpanded ? "▼" : "▶"}</span>
                <span className="truncate">{folder.name}</span>
                <span className="ml-auto text-xs text-muted-foreground">
                  ({folderQueries.length})
                </span>
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="ml-1 text-destructive opacity-0 transition-opacity group-hover:opacity-100"
                onClick={() => handleDeleteFolder(folder.id)}
                title={t("common.actions.delete")}
              >
                ×
              </Button>
            </div>

            {isExpanded && folderQueries.length > 0 && (
              <div className="ml-4">
                {folderQueries.map((q) => (
                  <div
                    key={q.id}
                    className="group flex items-center justify-between rounded-sm px-2 py-1 transition-colors hover:bg-background"
                  >
                    <Button
                      type="button"
                      variant="ghost"
                      className="flex-1 justify-start truncate text-left text-xs text-foreground"
                      onClick={() => onSelectQuery(q.sql)}
                      title={q.name}
                    >
                      {q.name}
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="ml-1 text-destructive opacity-0 transition-opacity group-hover:opacity-100"
                      onClick={() => handleDeleteQuery(q.id)}
                      title={t("common.actions.delete")}
                    >
                      ×
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>
        );
      })}

      {queries.length === 0 && folders.length === 0 && (
        <div className="py-4 text-center text-xs italic text-muted-foreground">
          {t("query.noSavedQueries")}
        </div>
      )}
    </div>
  );
}
