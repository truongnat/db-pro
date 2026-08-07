import { useCallback } from "react";

import { requestCloseTab } from "@/commons/services/request-close-tab";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

export function useTabCloseGuard() {
  const open = useCloseGuardStore((s) => s.open);
  const dirtyCount = useCloseGuardStore((s) => s.dirtyCount);
  const tabIds = useCloseGuardStore((s) => s.tabIds);
  const closeDialog = useCloseGuardStore((s) => s.closeDialog);
  const openDialog = useCloseGuardStore((s) => s.openDialog);

  const onConfirm = useCallback(() => {
    useWorkspaceStore.getState().closeTabs(tabIds);
    closeDialog();
  }, [tabIds, closeDialog]);

  const onCancel = useCallback(() => {
    closeDialog();
  }, [closeDialog]);

  const requestCloseMany = useCallback(
    (ids: string[]) => {
      const { tabs } = useWorkspaceStore.getState();
      const dirtyIds = ids.filter((id) => tabs.find((t) => t.id === id)?.dirty);

      if (dirtyIds.length === 0) {
        useWorkspaceStore.getState().closeTabs(ids);
        return;
      }

      openDialog(ids, dirtyIds.length);
    },
    [openDialog],
  );

  return {
    dialogOpen: open,
    dirtyCount,
    onConfirm,
    onCancel,
    requestClose: requestCloseTab,
    requestCloseMany,
  };
}
