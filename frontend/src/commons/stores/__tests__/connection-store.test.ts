import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useConnectionStore } from "../connection.store";

function resetStore() {
  useConnectionStore.getState().reset();
}

describe("ConnectionStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("initial state", () => {
    it("starts with empty connections and no explorer connection", () => {
      const state = useConnectionStore.getState();
      expect(state.connections).toEqual([]);
      expect(state.explorerConnectionId).toBeNull();
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe("setConnections", () => {
    it("replaces all connections", () => {
      const conns = [
        { id: "c1", name: "DB 1" },
        { id: "c2", name: "DB 2" },
      ];
      useConnectionStore.getState().setConnections(conns);
      expect(useConnectionStore.getState().connections).toEqual(conns);
    });

    it("replaces previous connections", () => {
      useConnectionStore.getState().setConnections([{ id: "c1", name: "A" }]);
      useConnectionStore.getState().setConnections([{ id: "c2", name: "B" }]);
      expect(useConnectionStore.getState().connections).toHaveLength(1);
      expect((useConnectionStore.getState().connections[0] as { id: string }).id).toBe("c2");
    });
  });

  describe("setExplorerConnection", () => {
    it("sets the explorer connection id", () => {
      useConnectionStore.getState().setExplorerConnection("conn-1");
      expect(useConnectionStore.getState().explorerConnectionId).toBe("conn-1");
    });

    it("clears with null", () => {
      useConnectionStore.getState().setExplorerConnection("conn-1");
      useConnectionStore.getState().setExplorerConnection(null);
      expect(useConnectionStore.getState().explorerConnectionId).toBeNull();
    });
  });

  describe("setLoading", () => {
    it("sets loading flag", () => {
      useConnectionStore.getState().setLoading(true);
      expect(useConnectionStore.getState().isLoading).toBe(true);
      useConnectionStore.getState().setLoading(false);
      expect(useConnectionStore.getState().isLoading).toBe(false);
    });
  });

  describe("setError", () => {
    it("sets error", () => {
      const error = { code: "CONNECTION_FAILED" as const, userMessage: "Failed", technicalMessage: "timeout", messageId: "err.conn" };
      useConnectionStore.getState().setError(error);
      expect(useConnectionStore.getState().error).toEqual(error);
    });

    it("clears error with null", () => {
      useConnectionStore.getState().setError({ code: "CONNECTION_FAILED" as const, userMessage: "x", technicalMessage: "y", messageId: "z" });
      useConnectionStore.getState().setError(null);
      expect(useConnectionStore.getState().error).toBeNull();
    });
  });

  describe("addConnection", () => {
    it("appends a connection", () => {
      useConnectionStore.getState().addConnection({ id: "c1", name: "A" });
      expect(useConnectionStore.getState().connections).toHaveLength(1);
      useConnectionStore.getState().addConnection({ id: "c2", name: "B" });
      expect(useConnectionStore.getState().connections).toHaveLength(2);
    });
  });

  describe("updateConnection", () => {
    it("updates connection by id", () => {
      useConnectionStore.getState().setConnections([
        { id: "c1", name: "Old" },
        { id: "c2", name: "Keep" },
      ]);
      useConnectionStore.getState().updateConnection("c1", { id: "c1", name: "New" });
      const conns = useConnectionStore.getState().connections as { id: string; name: string }[];
      expect(conns.find((c) => c.id === "c1")!.name).toBe("New");
      expect(conns.find((c) => c.id === "c2")!.name).toBe("Keep");
    });

    it("does nothing if id not found", () => {
      useConnectionStore.getState().setConnections([{ id: "c1", name: "A" }]);
      useConnectionStore.getState().updateConnection("nonexistent", { id: "x", name: "B" });
      expect(useConnectionStore.getState().connections).toHaveLength(1);
    });
  });

  describe("removeConnection", () => {
    it("removes connection by id", () => {
      useConnectionStore.getState().setConnections([
        { id: "c1", name: "A" },
        { id: "c2", name: "B" },
      ]);
      useConnectionStore.getState().removeConnection("c1");
      expect(useConnectionStore.getState().connections).toHaveLength(1);
      expect((useConnectionStore.getState().connections[0] as { id: string }).id).toBe("c2");
    });

    it("clears explorerConnectionId if removed connection was active", () => {
      useConnectionStore.getState().setConnections([{ id: "c1", name: "A" }]);
      useConnectionStore.getState().setExplorerConnection("c1");
      useConnectionStore.getState().removeConnection("c1");
      expect(useConnectionStore.getState().explorerConnectionId).toBeNull();
    });

    it("preserves explorerConnectionId if different connection removed", () => {
      useConnectionStore.getState().setConnections([
        { id: "c1", name: "A" },
        { id: "c2", name: "B" },
      ]);
      useConnectionStore.getState().setExplorerConnection("c1");
      useConnectionStore.getState().removeConnection("c2");
      expect(useConnectionStore.getState().explorerConnectionId).toBe("c1");
    });
  });

  describe("reset", () => {
    it("returns to initial state", () => {
      const store = useConnectionStore.getState();
      store.setConnections([{ id: "c1", name: "A" }]);
      store.setExplorerConnection("c1");
      store.setLoading(true);
      store.setError({ code: "CONNECTION_FAILED" as const, userMessage: "x", technicalMessage: "y", messageId: "z" });

      store.reset();

      const state = useConnectionStore.getState();
      expect(state.connections).toEqual([]);
      expect(state.explorerConnectionId).toBeNull();
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });
  });
});
