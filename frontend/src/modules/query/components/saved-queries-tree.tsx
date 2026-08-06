import { useCallback, useState } from "react";

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
        <span className="font-medium" style={{ color: "var(--color-text)" }}>
          {t("query.savedQueries")}
        </span>
        <button
          type="button"
          onClick={() => setShowNewFolderInput(!showNewFolderInput)}
          className="rounded-[var(--radius-sm)] px-2 py-0.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
          style={{ color: "var(--color-primary, #3b82f6)" }}
        >
          + {t("query.newFolder")}
        </button>
      </div>

      {showNewFolderInput && (
        <div className="mb-2 flex gap-1">
          <input
            type="text"
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
            placeholder={t("query.folderName")}
            className="flex-1 rounded-[var(--radius-sm)] border px-2 py-1 text-xs outline-none focus:border-[var(--color-primary,#3b82f6)]"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-bg)",
              color: "var(--color-text)",
            }}
            onKeyDown={(e) => e.key === "Enter" && handleCreateFolder()}
          />
          <button
            type="button"
            onClick={handleCreateFolder}
            className="rounded-[var(--radius-sm)] px-2 py-1 text-xs"
            style={{ color: "var(--color-primary, #3b82f6)" }}
          >
            {t("common.actions.save")}
          </button>
        </div>
      )}

      {rootQueries.length > 0 && (
        <div className="mb-2">
          {rootQueries.map((q) => (
            <div
              key={q.id}
              className="group flex items-center justify-between rounded-[var(--radius-sm)] px-2 py-1 transition-colors hover:bg-[var(--color-bg)]"
            >
              <button
                type="button"
                className="flex-1 truncate text-left text-xs"
                style={{ color: "var(--color-text)" }}
                onClick={() => onSelectQuery(q.sql)}
                title={q.name}
              >
                {q.name}
              </button>
              <button
                type="button"
                className="ml-1 opacity-0 transition-opacity group-hover:opacity-100"
                style={{ color: "var(--color-error, #ef4444)" }}
                onClick={() => handleDeleteQuery(q.id)}
                title={t("common.actions.delete")}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}

      {folders.map((folder) => {
        const isExpanded = expandedFolders.has(folder.id);
        const folderQueries = queriesByFolder.get(folder.name) ?? [];

        return (
          <div key={folder.id} className="mb-1">
            <div className="group flex items-center justify-between rounded-[var(--radius-sm)] px-2 py-1 transition-colors hover:bg-[var(--color-bg)]">
              <button
                type="button"
                className="flex flex-1 items-center gap-1 text-left text-xs font-medium"
                style={{ color: "var(--color-text)" }}
                onClick={() => toggleFolder(folder.id)}
              >
                <span>{isExpanded ? "▼" : "▶"}</span>
                <span className="truncate">{folder.name}</span>
                <span
                  className="ml-auto text-xs"
                  style={{ color: "var(--color-text-secondary)" }}
                >
                  ({folderQueries.length})
                </span>
              </button>
              <button
                type="button"
                className="ml-1 opacity-0 transition-opacity group-hover:opacity-100"
                style={{ color: "var(--color-error, #ef4444)" }}
                onClick={() => handleDeleteFolder(folder.id)}
                title={t("common.actions.delete")}
              >
                ×
              </button>
            </div>

            {isExpanded && folderQueries.length > 0 && (
              <div className="ml-4">
                {folderQueries.map((q) => (
                  <div
                    key={q.id}
                    className="group flex items-center justify-between rounded-[var(--radius-sm)] px-2 py-1 transition-colors hover:bg-[var(--color-bg)]"
                  >
                    <button
                      type="button"
                      className="flex-1 truncate text-left text-xs"
                      style={{ color: "var(--color-text)" }}
                      onClick={() => onSelectQuery(q.sql)}
                      title={q.name}
                    >
                      {q.name}
                    </button>
                    <button
                      type="button"
                      className="ml-1 opacity-0 transition-opacity group-hover:opacity-100"
                      style={{ color: "var(--color-error, #ef4444)" }}
                      onClick={() => handleDeleteQuery(q.id)}
                      title={t("common.actions.delete")}
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        );
      })}

      {queries.length === 0 && folders.length === 0 && (
        <div
          className="py-4 text-center text-xs italic"
          style={{ color: "var(--color-text-secondary)" }}
        >
          {t("query.noSavedQueries")}
        </div>
      )}
    </div>
  );
}
