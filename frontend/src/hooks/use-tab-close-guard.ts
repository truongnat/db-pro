import { useCallback, useState } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";

interface CloseGuardState {
  open: boolean;
  ids: string[];
  count: number;
}

const INITIAL_STATE: CloseGuardState = { open: false, ids: [], count: 0 };

export function useTabCloseGuard() {
  const [state, setState] = useState<CloseGuardState>(INITIAL_STATE);

  const requestClose = useCallback(
    (id: string, opts?: { skipDirtyCheck?: boolean }) => {
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === id);
      if (!tab) return;

      if (opts?.skipDirtyCheck || !tab.dirty) {
        useWorkspaceStore.getState().closeTab(id);
        return;
      }

      setState({ open: true, ids: [id], count: 1 });
    },
    [],
  );

  const requestCloseMany = useCallback(
    (ids: string[]) => {
      const { tabs } = useWorkspaceStore.getState();
      const dirtyIds = ids.filter((id) => tabs.find((t) => t.id === id)?.dirty);

      if (dirtyIds.length === 0) {
        useWorkspaceStore.getState().closeTabs(ids);
        return;
      }

      setState({ open: true, ids, count: dirtyIds.length });
    },
    [],
  );

  const onConfirm = useCallback(() => {
    useWorkspaceStore.getState().closeTabs(state.ids);
    setState(INITIAL_STATE);
  }, [state.ids]);

  const onCancel = useCallback(() => {
    setState(INITIAL_STATE);
  }, []);

  return {
    dialogOpen: state.open,
    dirtyCount: state.count,
    onConfirm,
    onCancel,
    requestClose,
    requestCloseMany,
  };
}
