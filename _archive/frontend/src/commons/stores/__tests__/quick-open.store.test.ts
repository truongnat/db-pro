import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useQuickOpenStore } from "@/commons/stores/quick-open.store";

function resetStore() {
  useQuickOpenStore.setState({ isOpen: false, query: "", selectedIndex: 0 });
}

describe("QuickOpenStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  it("starts closed", () => {
    expect(useQuickOpenStore.getState().isOpen).toBe(false);
  });

  it("open() sets isOpen", () => {
    useQuickOpenStore.getState().open();
    expect(useQuickOpenStore.getState().isOpen).toBe(true);
  });

  it("close() clears query and selectedIndex", () => {
    useQuickOpenStore.getState().open();
    useQuickOpenStore.getState().setQuery("client");
    useQuickOpenStore.getState().setSelectedIndex(3);
    useQuickOpenStore.getState().close();
    const state = useQuickOpenStore.getState();
    expect(state.isOpen).toBe(false);
    expect(state.query).toBe("");
    expect(state.selectedIndex).toBe(0);
  });

  it("setQuery updates query and resets selectedIndex", () => {
    useQuickOpenStore.getState().setSelectedIndex(2);
    useQuickOpenStore.getState().setQuery("users");
    const state = useQuickOpenStore.getState();
    expect(state.query).toBe("users");
    expect(state.selectedIndex).toBe(0);
  });

  it("setSelectedIndex updates index", () => {
    useQuickOpenStore.getState().setSelectedIndex(5);
    expect(useQuickOpenStore.getState().selectedIndex).toBe(5);
  });
});
