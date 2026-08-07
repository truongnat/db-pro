import type { QueryTabData, WorkspaceTab } from "@/commons/types/workspace.types";

export function migratePersistedWorkspace(tabs: WorkspaceTab[]): WorkspaceTab[] {
  let changed = false;
  const migrated: WorkspaceTab[] = tabs.map((tab): WorkspaceTab => {
    if (tab.kind === "db-object") {
      const activeSection = tab.data.activeSection as string;
      if (activeSection === "structure") {
        changed = true;
        return { ...tab, data: { ...tab.data, activeSection: "columns" } };
      }
      return tab;
    }
    const data = tab.data as QueryTabData;
    if (!data.context) {
      changed = true;
      return {
        ...tab,
        data: { ...data, context: { database: null, schema: null } },
      };
    }
    return tab;
  });
  return changed ? migrated : tabs;
}
