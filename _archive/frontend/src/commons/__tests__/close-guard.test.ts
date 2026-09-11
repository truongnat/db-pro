import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useCloseGuardStore } from "../stores/close-guard.store";

function resetStore() {
  useCloseGuardStore.setState({ open: false, tabIds: [], dirtyCount: 0 });
}

describe("CloseGuardStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  it("starts closed", () => {
    const state = useCloseGuardStore.getState();
    expect(state.open).toBe(false);
    expect(state.tabIds).toEqual([]);
    expect(state.dirtyCount).toBe(0);
  });

  it("opens dialog with tab ids and dirty count", () => {
    useCloseGuardStore.getState().openDialog(["tab-1", "tab-2"], 2);
    const state = useCloseGuardStore.getState();
    expect(state.open).toBe(true);
    expect(state.tabIds).toEqual(["tab-1", "tab-2"]);
    expect(state.dirtyCount).toBe(2);
  });

  it("closes dialog and resets state", () => {
    useCloseGuardStore.getState().openDialog(["tab-1"], 1);
    useCloseGuardStore.getState().closeDialog();
    const state = useCloseGuardStore.getState();
    expect(state.open).toBe(false);
    expect(state.tabIds).toEqual([]);
    expect(state.dirtyCount).toBe(0);
  });
});
