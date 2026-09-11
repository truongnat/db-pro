import { useCallback } from "react";

import { requestCloseTab, requestCloseTabs } from "@/commons/services/request-close-tab";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useStagedChangesStore } from "@/modules/data-grid/state/staged-changes.store";

export function useTabCloseGuard() {
  const open = useCloseGuardStore((s) => s.open);
  const dirtyCount = useCloseGuardStore((s) => s.dirtyCount);
  const tabIds = useCloseGuardStore((s) => s.tabIds);
  const closeDialog = useCloseGuardStore((s) => s.closeDialog);

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

  return {
    dialogOpen: open,
    dirtyCount,
    onConfirm,
    onCancel,
    requestClose: requestCloseTab,
    requestCloseMany: useCallback((ids: string[]) => requestCloseTabs(ids), []),
  };
}
