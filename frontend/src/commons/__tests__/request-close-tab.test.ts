import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { requestCloseTab } from "../services/request-close-tab";
import type { WorkspaceTab } from "@/commons/types/workspace.types";

function makeTab(overrides: Partial<WorkspaceTab> = {}): WorkspaceTab {
  return {
    id: "tab-1",
    kind: "query",
    connectionId: "conn-1",
    title: "Query 1",
    resourceKey: "query:1",
    order: 0,
    dirty: false,
    pinned: false,
    preview: false,
    data: {
      sql: "",
      status: "idle",
      error: null,
      result: null,
      timing: null,
      executionStartedAt: null,
      context: { database: null, schema: null },
    },
    ...overrides,
  } as WorkspaceTab;
}

function resetStores() {
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
  useCloseGuardStore.setState({ open: false, tabIds: [], dirtyCount: 0 });
}

describe("requestCloseTab", () => {
  beforeEach(resetStores);
  afterEach(resetStores);

  it("closes a clean tab immediately", () => {
    const tab = makeTab({ dirty: false });
    useWorkspaceStore.getState().openTab(tab);
    expect(useWorkspaceStore.getState().tabs).toHaveLength(1);

    requestCloseTab("tab-1");
    expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
  });

  it("does nothing for non-existent tab", () => {
    requestCloseTab("nonexistent");
    // Should not throw
  });

  it("opens close guard dialog for dirty tab", () => {
    const tab = makeTab({ dirty: true });
    useWorkspaceStore.getState().openTab(tab);

    requestCloseTab("tab-1");
    // Tab should still be open
    expect(useWorkspaceStore.getState().tabs).toHaveLength(1);
    // Dialog should be open
    expect(useCloseGuardStore.getState().open).toBe(true);
    expect(useCloseGuardStore.getState().tabIds).toEqual(["tab-1"]);
  });

  it("closes dirty tab immediately when skipDirtyCheck is set", () => {
    const tab = makeTab({ dirty: true });
    useWorkspaceStore.getState().openTab(tab);

    requestCloseTab("tab-1", { skipDirtyCheck: true });
    expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
    // Dialog should NOT be open
    expect(useCloseGuardStore.getState().open).toBe(false);
  });
});
