import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useTabGridStateStore } from "../state/tab-grid-state.store";

function resetStore() {
  useTabGridStateStore.setState({ states: {} });
}

describe("TabGridStateStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("getState", () => {
    it("returns default state for unknown tab", () => {
      const state = useTabGridStateStore.getState().getState("tab-1");
      expect(state.page).toBe(1);
      expect(state.pageSize).toBe(50);
      expect(state.filters).toEqual([]);
      expect(state.sorts).toEqual([]);
      expect(state.editingCell).toBeNull();
      expect(state.frozenColumns).toEqual([]);
      expect(state.hiddenColumns).toEqual([]);
      expect(state.chartConfig).toBeNull();
    });

    it("returns stored state for known tab", () => {
      useTabGridStateStore.getState().setState("tab-1", { page: 5 });
      const state = useTabGridStateStore.getState().getState("tab-1");
      expect(state.page).toBe(5);
    });
  });

  describe("setState", () => {
    it("merges partial state", () => {
      const { setState, getState } = useTabGridStateStore.getState();
      setState("tab-1", { page: 3, pageSize: 100 });
      const state = getState("tab-1");
      expect(state.page).toBe(3);
      expect(state.pageSize).toBe(100);
    });

    it("does not affect other tabs", () => {
      const { setState, getState } = useTabGridStateStore.getState();
      setState("tab-1", { page: 3 });
      setState("tab-2", { page: 7 });
      expect(getState("tab-1").page).toBe(3);
      expect(getState("tab-2").page).toBe(7);
    });
  });

  describe("draft filters (addDraftFilter / removeDraftFilter / applyFilters / clearFilters)", () => {
    it("addDraftFilter appends to draftFilters without changing applied filters", () => {
      const { addDraftFilter, getState } = useTabGridStateStore.getState();
      addDraftFilter("tab-1", {
        column: "name",
        op: "like",
        value: { type: "text", value: "John" },
      });
      const state = getState("tab-1");
      expect(state.draftFilters).toHaveLength(1);
      expect(state.draftFilters[0].column).toBe("name");
      expect(state.filters).toHaveLength(0); // applied unchanged
    });

    it("applyFilters copies draft → applied and resets page", () => {
      const { addDraftFilter, setState, applyFilters, getState } = useTabGridStateStore.getState();
      setState("tab-1", { page: 5 });
      addDraftFilter("tab-1", { column: "name", op: "like", value: { type: "text", value: "A" } });
      addDraftFilter("tab-1", { column: "age", op: "gt", value: { type: "int64", value: "18" } });
      applyFilters("tab-1");
      const state = getState("tab-1");
      expect(state.filters).toHaveLength(2);
      expect(state.filters[0].column).toBe("name");
      expect(state.filters[1].column).toBe("age");
      expect(state.page).toBe(1);
    });

    it("removeDraftFilter removes by index from draft list", () => {
      const { addDraftFilter, removeDraftFilter, getState } = useTabGridStateStore.getState();
      addDraftFilter("tab-1", { column: "a", op: "eq", value: { type: "text", value: "1" } });
      addDraftFilter("tab-1", { column: "b", op: "eq", value: { type: "text", value: "2" } });
      removeDraftFilter("tab-1", 0);
      const state = getState("tab-1");
      expect(state.draftFilters).toHaveLength(1);
      expect(state.draftFilters[0].column).toBe("b");
    });

    it("clearFilters clears both draft and applied, resets page", () => {
      const { addDraftFilter, applyFilters, clearFilters, setState, getState } =
        useTabGridStateStore.getState();
      setState("tab-1", { page: 3 });
      addDraftFilter("tab-1", { column: "x", op: "eq", value: { type: "text", value: "y" } });
      applyFilters("tab-1");
      expect(getState("tab-1").filters).toHaveLength(1);
      clearFilters("tab-1");
      const state = getState("tab-1");
      expect(state.filters).toHaveLength(0);
      expect(state.draftFilters).toHaveLength(0);
      expect(state.page).toBe(1);
    });
  });

  describe("draft sorts (addDraftSort / removeDraftSort / applySorts / clearSorts)", () => {
    it("addDraftSort appends to draftSorts without changing applied sorts", () => {
      const { addDraftSort, getState } = useTabGridStateStore.getState();
      addDraftSort("tab-1", { column: "name", direction: "asc" });
      const state = getState("tab-1");
      expect(state.draftSorts).toHaveLength(1);
      expect(state.sorts).toHaveLength(0); // applied unchanged
    });

    it("applySorts copies draft → applied", () => {
      const { addDraftSort, applySorts, getState } = useTabGridStateStore.getState();
      addDraftSort("tab-1", { column: "name", direction: "asc" });
      addDraftSort("tab-1", { column: "age", direction: "desc" });
      applySorts("tab-1");
      const state = getState("tab-1");
      expect(state.sorts).toHaveLength(2);
      expect(state.sorts[0]).toEqual({ column: "name", direction: "asc" });
    });

    it("applySorts resets page to 1", () => {
      const { setPage, addDraftSort, applySorts, getState } = useTabGridStateStore.getState();
      setPage("tab-1", 5);
      addDraftSort("tab-1", { column: "name", direction: "asc" });
      applySorts("tab-1");
      expect(getState("tab-1").page).toBe(1);
    });

    it("removeDraftSort removes by index from draft list", () => {
      const { addDraftSort, removeDraftSort, getState } = useTabGridStateStore.getState();
      addDraftSort("tab-1", { column: "a", direction: "asc" });
      addDraftSort("tab-1", { column: "b", direction: "desc" });
      removeDraftSort("tab-1", 0);
      const state = getState("tab-1");
      expect(state.draftSorts).toHaveLength(1);
      expect(state.draftSorts[0].column).toBe("b");
    });

    it("clearSorts clears both draft and applied", () => {
      const { addDraftSort, applySorts, clearSorts, getState } = useTabGridStateStore.getState();
      addDraftSort("tab-1", { column: "x", direction: "asc" });
      applySorts("tab-1");
      expect(getState("tab-1").sorts).toHaveLength(1);
      clearSorts("tab-1");
      const state = getState("tab-1");
      expect(state.sorts).toHaveLength(0);
      expect(state.draftSorts).toHaveLength(0);
    });

    it("clearSorts resets page to 1", () => {
      const { setPage, addDraftSort, applySorts, clearSorts, getState } =
        useTabGridStateStore.getState();
      setPage("tab-1", 3);
      addDraftSort("tab-1", { column: "x", direction: "asc" });
      applySorts("tab-1");
      clearSorts("tab-1");
      expect(getState("tab-1").page).toBe(1);
    });
  });

  describe("setSorts", () => {
    it("sets sorts for a tab", () => {
      const { setSorts, getState } = useTabGridStateStore.getState();
      setSorts("tab-1", [{ column: "name", direction: "asc" }]);
      expect(getState("tab-1").sorts).toEqual([{ column: "name", direction: "asc" }]);
    });

    it("replaces existing sorts", () => {
      const { setSorts, getState } = useTabGridStateStore.getState();
      setSorts("tab-1", [{ column: "name", direction: "asc" }]);
      setSorts("tab-1", [{ column: "age", direction: "desc" }]);
      expect(getState("tab-1").sorts).toEqual([{ column: "age", direction: "desc" }]);
    });

    it("setSorts resets page to 1", () => {
      const { setPage, setSorts, getState } = useTabGridStateStore.getState();
      setPage("tab-1", 7);
      setSorts("tab-1", [{ column: "name", direction: "asc" }]);
      expect(getState("tab-1").page).toBe(1);
    });
  });

  describe("setPage / setPageSize", () => {
    it("setPage updates page", () => {
      const { setPage, getState } = useTabGridStateStore.getState();
      setPage("tab-1", 10);
      expect(getState("tab-1").page).toBe(10);
    });

    it("setPageSize updates pageSize and resets page to 1", () => {
      const { setPageSize, setPage, getState } = useTabGridStateStore.getState();
      setPage("tab-1", 5);
      setPageSize("tab-1", 200);
      const state = getState("tab-1");
      expect(state.pageSize).toBe(200);
      expect(state.page).toBe(1);
    });
  });

  describe("setEditingCell", () => {
    it("sets editing cell", () => {
      const { setEditingCell, getState } = useTabGridStateStore.getState();
      setEditingCell("tab-1", { row: 3, col: 2 });
      expect(getState("tab-1").editingCell).toEqual({ row: 3, col: 2 });
    });

    it("clears editing cell with null", () => {
      const { setEditingCell, getState } = useTabGridStateStore.getState();
      setEditingCell("tab-1", { row: 3, col: 2 });
      setEditingCell("tab-1", null);
      expect(getState("tab-1").editingCell).toBeNull();
    });
  });

  describe("toggleFrozenColumn", () => {
    it("adds column to frozen list", () => {
      const { toggleFrozenColumn, getState } = useTabGridStateStore.getState();
      toggleFrozenColumn("tab-1", "id");
      expect(getState("tab-1").frozenColumns).toEqual(["id"]);
    });

    it("removes column if already frozen", () => {
      const { toggleFrozenColumn, getState } = useTabGridStateStore.getState();
      toggleFrozenColumn("tab-1", "id");
      toggleFrozenColumn("tab-1", "id");
      expect(getState("tab-1").frozenColumns).toEqual([]);
    });

    it("supports multiple frozen columns", () => {
      const { toggleFrozenColumn, getState } = useTabGridStateStore.getState();
      toggleFrozenColumn("tab-1", "id");
      toggleFrozenColumn("tab-1", "name");
      expect(getState("tab-1").frozenColumns).toEqual(["id", "name"]);
    });
  });

  describe("toggleHiddenColumn / setHiddenColumns", () => {
    it("toggleHiddenColumn adds column", () => {
      const { toggleHiddenColumn, getState } = useTabGridStateStore.getState();
      toggleHiddenColumn("tab-1", "secret_col");
      expect(getState("tab-1").hiddenColumns).toEqual(["secret_col"]);
    });

    it("toggleHiddenColumn removes if already hidden", () => {
      const { toggleHiddenColumn, getState } = useTabGridStateStore.getState();
      toggleHiddenColumn("tab-1", "secret_col");
      toggleHiddenColumn("tab-1", "secret_col");
      expect(getState("tab-1").hiddenColumns).toEqual([]);
    });

    it("setHiddenColumns replaces entire list", () => {
      const { setHiddenColumns, getState } = useTabGridStateStore.getState();
      setHiddenColumns("tab-1", ["a", "b", "c"]);
      expect(getState("tab-1").hiddenColumns).toEqual(["a", "b", "c"]);
    });
  });

  describe("setChartConfig", () => {
    it("sets chart config", () => {
      const { setChartConfig, getState } = useTabGridStateStore.getState();
      const config = { type: "bar" as const, xColumn: "name", yColumn: "count" };
      setChartConfig("tab-1", config);
      expect(getState("tab-1").chartConfig).toEqual(config);
    });

    it("clears chart config with null", () => {
      const { setChartConfig, getState } = useTabGridStateStore.getState();
      setChartConfig("tab-1", { type: "bar" as const, xColumn: "x", yColumn: "y" });
      setChartConfig("tab-1", null);
      expect(getState("tab-1").chartConfig).toBeNull();
    });
  });

  describe("resetTab", () => {
    it("removes tab state entirely", () => {
      const { setState, resetTab } = useTabGridStateStore.getState();
      setState("tab-1", { page: 5 });
      setState("tab-2", { page: 10 });
      resetTab("tab-1");
      const { states } = useTabGridStateStore.getState();
      expect(states["tab-1"]).toBeUndefined();
      expect(states["tab-2"]).toBeDefined();
    });

    it("getState returns defaults after reset", () => {
      const { setState, resetTab, getState } = useTabGridStateStore.getState();
      setState("tab-1", { page: 99 });
      resetTab("tab-1");
      expect(getState("tab-1").page).toBe(1);
    });
  });

  describe("gc (garbage collection)", () => {
    it("removes state for tabs no longer present", () => {
      const { setState, gc } = useTabGridStateStore.getState();
      setState("tab-1", { page: 1 });
      setState("tab-2", { page: 2 });
      setState("tab-3", { page: 3 });
      gc(new Set(["tab-1", "tab-3"]));
      const { states } = useTabGridStateStore.getState();
      expect(states["tab-1"]).toBeDefined();
      expect(states["tab-2"]).toBeUndefined();
      expect(states["tab-3"]).toBeDefined();
    });

    it("handles empty valid set by clearing all", () => {
      const { setState, gc } = useTabGridStateStore.getState();
      setState("tab-1", { page: 1 });
      gc(new Set());
      expect(useTabGridStateStore.getState().states).toEqual({});
    });
  });
});
