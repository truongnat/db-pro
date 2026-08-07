import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useExplorerStore } from "@/commons/stores/explorer.store";

function resetStore() {
  useExplorerStore.setState({ expandedNodes: [], filter: "" });
}

describe("ExplorerStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("toggleNode", () => {
    it("adds a collapsed node", () => {
      useExplorerStore.getState().toggleNode("conn:conn-1");
      expect(useExplorerStore.getState().expandedNodes).toEqual(["conn:conn-1"]);
    });

    it("removes an expanded node", () => {
      useExplorerStore.getState().toggleNode("conn:conn-1");
      useExplorerStore.getState().toggleNode("conn:conn-1");
      expect(useExplorerStore.getState().expandedNodes).toEqual([]);
    });
  });

  describe("expandNode", () => {
    it("adds a collapsed node", () => {
      useExplorerStore.getState().expandNode("conn:conn-1");
      expect(useExplorerStore.getState().expandedNodes).toEqual(["conn:conn-1"]);
    });

    it("keeps an already expanded node expanded", () => {
      useExplorerStore.getState().expandNode("conn:conn-1");
      useExplorerStore.getState().expandNode("conn:conn-1");
      expect(useExplorerStore.getState().expandedNodes).toEqual(["conn:conn-1"]);
    });

    it("does not collapse other nodes", () => {
      useExplorerStore.getState().expandNode("conn:conn-1");
      useExplorerStore.getState().expandNode("schema:conn-1:public");
      expect(useExplorerStore.getState().expandedNodes).toEqual([
        "conn:conn-1",
        "schema:conn-1:public",
      ]);
    });
  });
});
