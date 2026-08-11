import { useCallback } from "react";

import { requestCloseTab } from "@/commons/services/request-close-tab";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useStagedChangesStore } from "@/modules/data-grid/state/staged-changes.store";

export function useTabCloseGuard() {
  const open = useCloseGuardStore((s) => s.open);
  const dirtyCount = useCloseGuardStore((s) => s.dirtyCount);
  const tabIds = useCloseGuardStore((s) => s.tabIds);
  const closeDialog = useCloseGuardStore((s) => s.closeDialog);
  const openDialog = useCloseGuardStore((s) => s.openDialog);

  const onConfirm = useCallback(() => {
    for (const id of tabIds) {
      useStagedChangesStore.getState().clearTab(id);
    }
    useWorkspaceStore.getState().closeTabs(tabIds);
    closeDialog();
  }, [tabIds, closeDialog]);

  const onCancel = useCallback(() => {
    closeDialog();
  }, [closeDialog]);

  const requestCloseMany = useCallback(
    (ids: string[]) => {
      const { tabs } = useWorkspaceStore.getState();
      const dirtyIds = ids.filter((id) => {
        const tab = tabs.find((t) => t.id === id);
        if (!tab) return false;
        if (tab.dirty) return true;
        return useStagedChangesStore.getState().getCount(id) > 0;
      });

      if (dirtyIds.length === 0) {
        for (const id of ids) {
          useStagedChangesStore.getState().clearTab(id);
        }
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
