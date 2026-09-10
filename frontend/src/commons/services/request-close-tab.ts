import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useStagedChangesStore } from "@/modules/data-grid/state/staged-changes.store";

function hasUnsavedWork(id: string): boolean {
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === id);
  if (!tab) return false;
  if (tab.dirty) return true;
  return useStagedChangesStore.getState().getCount(id) > 0;
}

export function requestCloseTab(id: string, opts?: { skipDirtyCheck?: boolean }): void {
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === id);
  if (!tab) return;

  if (opts?.skipDirtyCheck || !hasUnsavedWork(id)) {
    useStagedChangesStore.getState().clearTab(id);
    useWorkspaceStore.getState().closeTab(id);
    return;
  }

  useCloseGuardStore.getState().openDialog([id], 1);
}
