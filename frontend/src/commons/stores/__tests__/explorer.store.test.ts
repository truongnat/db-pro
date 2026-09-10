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

  describe("setFilter", () => {
    it("sets the filter string", () => {
      useExplorerStore.getState().setFilter("users");
      expect(useExplorerStore.getState().filter).toBe("users");
    });

    it("overwrites previous filter", () => {
      useExplorerStore.getState().setFilter("users");
      useExplorerStore.getState().setFilter("orders");
      expect(useExplorerStore.getState().filter).toBe("orders");
    });

    it("clears filter with empty string", () => {
      useExplorerStore.getState().setFilter("users");
      useExplorerStore.getState().setFilter("");
      expect(useExplorerStore.getState().filter).toBe("");
    });
  });

  describe("collapseAll", () => {
    it("clears all expanded nodes", () => {
      useExplorerStore.getState().expandNode("conn:conn-1");
      useExplorerStore.getState().expandNode("schema:conn-1:public");
      useExplorerStore.getState().collapseAll();
      expect(useExplorerStore.getState().expandedNodes).toEqual([]);
    });

    it("is a no-op when already empty", () => {
      useExplorerStore.getState().collapseAll();
      expect(useExplorerStore.getState().expandedNodes).toEqual([]);
    });
  });
});
