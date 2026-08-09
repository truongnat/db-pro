import { beforeEach, describe, expect, it } from "vitest";

import { useConnectionStore } from "../connection.store";

function resetStore() {
  useConnectionStore.getState().reset();
}

describe("ConnectionStore — activeConnectionIds", () => {
  beforeEach(resetStore);

  it("starts with no active connections", () => {
    expect(useConnectionStore.getState().activeConnectionIds).toEqual([]);
  });

  it("adds an active connection", () => {
    useConnectionStore.getState().setActiveConnection("c1");
    expect(useConnectionStore.getState().activeConnectionIds).toEqual(["c1"]);
  });

  it("does not duplicate active connections", () => {
    useConnectionStore.getState().setActiveConnection("c1");
    useConnectionStore.getState().setActiveConnection("c1");
    expect(useConnectionStore.getState().activeConnectionIds).toEqual(["c1"]);
  });

  it("tracks multiple active connections", () => {
    useConnectionStore.getState().setActiveConnection("c1");
    useConnectionStore.getState().setActiveConnection("c2");
    expect(useConnectionStore.getState().activeConnectionIds).toEqual(["c1", "c2"]);
  });

  it("removes an active connection", () => {
    useConnectionStore.getState().setActiveConnection("c1");
    useConnectionStore.getState().setActiveConnection("c2");
    useConnectionStore.getState().removeActiveConnection("c1");
    expect(useConnectionStore.getState().activeConnectionIds).toEqual(["c2"]);
  });

  it("removing a connection also removes it from activeConnectionIds", () => {
    useConnectionStore.getState().setConnections([
      { id: "c1", name: "A" } as never,
      { id: "c2", name: "B" } as never,
    ]);
    useConnectionStore.getState().setActiveConnection("c1");
    useConnectionStore.getState().setActiveConnection("c2");
    useConnectionStore.getState().removeConnection("c1");
    expect(useConnectionStore.getState().activeConnectionIds).toEqual(["c2"]);
  });

  it("removing explorer connection reference on delete still works", () => {
    useConnectionStore.getState().setConnections([{ id: "c1", name: "A" } as never]);
    useConnectionStore.getState().setExplorerConnection("c1");
    useConnectionStore.getState().setActiveConnection("c1");
    useConnectionStore.getState().removeConnection("c1");
    const state = useConnectionStore.getState();
    expect(state.explorerConnectionId).toBeNull();
    expect(state.activeConnectionIds).toEqual([]);
  });

  it("reset clears activeConnectionIds", () => {
    useConnectionStore.getState().setActiveConnection("c1");
    useConnectionStore.getState().reset();
    expect(useConnectionStore.getState().activeConnectionIds).toEqual([]);
  });
});
