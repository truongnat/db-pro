import { type FormEvent, useCallback, useEffect, useMemo, useState } from "react";

import {
  deleteConnection,
  getConnection,
  listConnections,
  upsertConnection,
} from "@/lib/tauri";
import type { ConnectionListItem } from "@/types";

import {
  loadCachedDraft,
  loadCachedSelectedConnectionId,
  resetCachedDraft,
  saveCachedDraft,
  saveCachedSelectedConnectionId,
} from "./cache";
import {
  defaultConnectionDraft,
  fromConnectionInput,
  toConnectionInput,
  type ConnectionDraft,
} from "./draft";

type UseConnectionControllerOptions = {
  onSetBusyAction: (value: string | null) => void;
  onStatus: (message: string) => void;
  onClearError: () => void;
  onError: (error: unknown) => void;
};

export type ConnectionController = ReturnType<typeof useConnectionController>;

export function useConnectionController({
  onSetBusyAction,
  onStatus,
  onClearError,
  onError,
}: UseConnectionControllerOptions) {
  const [connections, setConnections] = useState<ConnectionListItem[]>([]);
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(() =>
    loadCachedSelectedConnectionId(),
  );

  const [draft, setDraft] = useState<ConnectionDraft>(defaultConnectionDraft);
  const [connectionModalOpen, setConnectionModalOpen] = useState(false);
  const [connectionModalMode, setConnectionModalMode] = useState<"create" | "edit">(
    "create",
  );
  const [editingConnectionId, setEditingConnectionId] = useState<string | null>(null);
  const [editBaselineDraft, setEditBaselineDraft] = useState<ConnectionDraft | null>(null);

  const selectedConnection = useMemo(
    () => connections.find((item) => item.id === selectedConnectionId) ?? null,
    [connections, selectedConnectionId],
  );

  const refreshConnections = useCallback(
    async (preferredId?: string) => {
      try {
        const items = await listConnections();
        setConnections(items);
        onClearError();

        setSelectedConnectionId((previous) => {
          if (preferredId && items.some((item) => item.id === preferredId)) {
            return preferredId;
          }
          if (previous && items.some((item) => item.id === previous)) {
            return previous;
          }
          return items[0]?.id ?? null;
        });
      } catch (error) {
        setConnections([]);
        setSelectedConnectionId(null);
        onError(error);
      }
    },
    [onClearError, onError],
  );

  useEffect(() => {
    void refreshConnections();
  }, [refreshConnections]);

  useEffect(() => {
    saveCachedSelectedConnectionId(selectedConnectionId);
  }, [selectedConnectionId]);

  useEffect(() => {
    if (connectionModalMode === "create") {
      saveCachedDraft(draft);
    }
  }, [connectionModalMode, draft]);

  const openCreateModal = useCallback(() => {
    setConnectionModalMode("create");
    setEditingConnectionId(null);
    setEditBaselineDraft(null);
    setDraft(loadCachedDraft(defaultConnectionDraft));
    setConnectionModalOpen(true);
  }, []);

  const openEditModal = useCallback(
    async (connectionId: string) => {
      try {
        onSetBusyAction("load-connection");
        const input = await getConnection(connectionId);
        const nextDraft = fromConnectionInput(input);

        setDraft(nextDraft);
        setEditBaselineDraft(nextDraft);
        setEditingConnectionId(connectionId);
        setConnectionModalMode("edit");
        setConnectionModalOpen(true);
        onStatus(`Editing connection '${input.name}'.`);
        onClearError();
      } catch (error) {
        onError(error);
      } finally {
        onSetBusyAction(null);
      }
    },
    [onClearError, onError, onSetBusyAction, onStatus],
  );

  const closeConnectionModal = useCallback(() => {
    setConnectionModalOpen(false);
  }, []);

  const resetConnectionDraft = useCallback(() => {
    if (connectionModalMode === "edit" && editBaselineDraft) {
      setDraft(editBaselineDraft);
      return;
    }

    setDraft(defaultConnectionDraft);
    resetCachedDraft();
  }, [connectionModalMode, editBaselineDraft]);

  const saveConnection = useCallback(async () => {
    try {
      onSetBusyAction("save-connection");

      if (connectionModalMode === "edit" && !editingConnectionId) {
        throw new Error("Missing connection id for edit mode.");
      }

      const connectionId =
        connectionModalMode === "edit" ? editingConnectionId ?? undefined : undefined;
      const saved = await upsertConnection(toConnectionInput(draft, connectionId));

      onStatus(
        connectionModalMode === "create"
          ? `Saved connection '${saved.name}' (persisted).`
          : `Updated connection '${saved.name}'.`,
      );
      onClearError();

      await refreshConnections(saved.id);
      setConnectionModalOpen(false);
      setConnectionModalMode("create");
      setEditingConnectionId(null);
      setEditBaselineDraft(null);

      if (connectionModalMode === "create") {
        setDraft(defaultConnectionDraft);
        resetCachedDraft();
      }
    } catch (error) {
      onError(error);
    } finally {
      onSetBusyAction(null);
    }
  }, [
    connectionModalMode,
    draft,
    editingConnectionId,
    onClearError,
    onError,
    onSetBusyAction,
    onStatus,
    refreshConnections,
  ]);

  const submitConnectionForm = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      await saveConnection();
    },
    [saveConnection],
  );

  const removeConnection = useCallback(
    async (connectionId: string) => {
      const item = connections.find((entry) => entry.id === connectionId);
      const confirmed = window.confirm(
        `Delete connection '${item?.name ?? connectionId}'? This action cannot be undone.`,
      );
      if (!confirmed) {
        return;
      }

      try {
        onSetBusyAction("delete-connection");
        await deleteConnection(connectionId);
        onStatus("Connection removed.");
        onClearError();
        await refreshConnections();
      } catch (error) {
        onError(error);
      } finally {
        onSetBusyAction(null);
      }
    },
    [connections, onClearError, onError, onSetBusyAction, onStatus, refreshConnections],
  );

  const applyResetConnectionState = useCallback(
    (items: ConnectionListItem[], nextSelectedId: string | null) => {
      setConnections(items);
      setSelectedConnectionId(nextSelectedId);
      setConnectionModalOpen(false);
      setConnectionModalMode("create");
      setEditingConnectionId(null);
      setEditBaselineDraft(null);
      setDraft(defaultConnectionDraft);
      resetCachedDraft();
    },
    [],
  );

  return {
    connections,
    selectedConnection,
    selectedConnectionId,
    setSelectedConnectionId,
    draft,
    setDraft,
    connectionModalOpen,
    connectionModalMode,
    openCreateModal,
    openEditModal,
    closeConnectionModal,
    resetConnectionDraft,
    submitConnectionForm,
    removeConnection,
    applyResetConnectionState,
  };
}
