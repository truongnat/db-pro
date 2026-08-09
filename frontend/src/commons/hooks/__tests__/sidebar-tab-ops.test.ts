import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useSidebarTabOps } from "@/commons/hooks/use-sidebar-tab-ops";

function resetStores() {
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
}

function useOps() {
  return renderHook(() => useSidebarTabOps());
}

describe("useSidebarTabOps — openTableData (double-click behavior)", () => {
  beforeEach(resetStores);

  it("creates a permanent Data tab when no tab exists", () => {
    const { result } = useOps();
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });

    const tabs = useWorkspaceStore.getState().tabs;
    expect(tabs).toHaveLength(1);
    expect(tabs[0].kind).toBe("db-object");
    expect(tabs[0].preview).toBe(false);
    if (tabs[0].kind === "db-object") {
      expect(tabs[0].data.activeSection).toBe("data");
      expect(tabs[0].data.objectName).toBe("users");
      expect(tabs[0].data.objectType).toBe("table");
    }
  });

  it("reuses existing preview tab and switches to Data", () => {
    const { result } = useOps();

    // Single-click creates preview
    act(() => {
      result.current.openSchemaPreview("conn-1", "public", "users", "table");
    });
    expect(useWorkspaceStore.getState().tabs).toHaveLength(1);
    expect(useWorkspaceStore.getState().tabs[0].preview).toBe(true);

    // Double-click promotes and switches to Data
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });
    const tabs = useWorkspaceStore.getState().tabs;
    expect(tabs).toHaveLength(1); // no duplicate
    expect(tabs[0].preview).toBe(false);
    if (tabs[0].kind === "db-object") {
      expect(tabs[0].data.activeSection).toBe("data");
    }
  });

  it("reuses existing permanent Columns tab and switches to Data", () => {
    const { result } = useOps();

    // Create a permanent Data tab then switch back to columns
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });
    useWorkspaceStore
      .getState()
      .setDbObjectSection(useWorkspaceStore.getState().tabs[0].id, "columns");

    // Double-click again → should reuse and switch to Data
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });
    const tabs = useWorkspaceStore.getState().tabs;
    expect(tabs).toHaveLength(1); // still one tab
    if (tabs[0].kind === "db-object") {
      expect(tabs[0].data.activeSection).toBe("data");
    }
  });

  it("repeated double-click does not duplicate tabs", () => {
    const { result } = useOps();
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });
    act(() => {
      result.current.openTableData("conn-1", "public", "users", "table");
    });

    expect(useWorkspaceStore.getState().tabs).toHaveLength(1);
  });

  it("opens view as Data with objectType=view", () => {
    const { result } = useOps();
    act(() => {
      result.current.openTableData("conn-1", "public", "active_users", "view");
    });

    const tabs = useWorkspaceStore.getState().tabs;
    expect(tabs).toHaveLength(1);
    if (tabs[0].kind === "db-object") {
      expect(tabs[0].data.activeSection).toBe("data");
      expect(tabs[0].data.objectType).toBe("view");
    }
  });
});
