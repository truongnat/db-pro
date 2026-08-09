import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

export function requestCloseTab(id: string, opts?: { skipDirtyCheck?: boolean }): void {
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === id);
  if (!tab) return;

  if (opts?.skipDirtyCheck || !tab.dirty) {
    useWorkspaceStore.getState().closeTab(id);
    return;
  }

  useCloseGuardStore.getState().openDialog([id], 1);
}
