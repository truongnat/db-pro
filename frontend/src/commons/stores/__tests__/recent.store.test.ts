import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useRecentStore } from "@/commons/stores/recent.store";

function resetStore() {
  useRecentStore.setState({
    recentConnections: [],
    connectionDialogOpen: false,
    connectionDialogEditId: null,
  });
}

describe("RecentStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("addRecentConnection", () => {
    it("prepends a new entry", () => {
      useRecentStore.getState().addRecentConnection("conn-1");

      const { recentConnections } = useRecentStore.getState();
      expect(recentConnections).toHaveLength(1);
      expect(recentConnections[0].connectionId).toBe("conn-1");
      expect(recentConnections[0].connectCount).toBe(1);
    });

    it("upserts existing entry — moves to front, increments count", () => {
      useRecentStore.getState().addRecentConnection("conn-1");
      useRecentStore.getState().addRecentConnection("conn-2");
      useRecentStore.getState().addRecentConnection("conn-1");

      const { recentConnections } = useRecentStore.getState();
      expect(recentConnections).toHaveLength(2);
      expect(recentConnections[0].connectionId).toBe("conn-1");
      expect(recentConnections[0].connectCount).toBe(2);
      expect(recentConnections[1].connectionId).toBe("conn-2");
    });

    it("trims to MAX_RECENT", () => {
      for (let i = 1; i <= 12; i++) {
        useRecentStore.getState().addRecentConnection(`conn-${i}`);
      }

      const { recentConnections } = useRecentStore.getState();
      expect(recentConnections).toHaveLength(10);
      expect(recentConnections[0].connectionId).toBe("conn-12");
    });
  });

  describe("removeRecentConnection", () => {
    it("removes by connectionId", () => {
      useRecentStore.getState().addRecentConnection("conn-1");
      useRecentStore.getState().addRecentConnection("conn-2");

      useRecentStore.getState().removeRecentConnection("conn-1");

      const { recentConnections } = useRecentStore.getState();
      expect(recentConnections).toHaveLength(1);
      expect(recentConnections[0].connectionId).toBe("conn-2");
    });

    it("does nothing if connectionId not found", () => {
      useRecentStore.getState().addRecentConnection("conn-1");

      useRecentStore.getState().removeRecentConnection("nonexistent");

      expect(useRecentStore.getState().recentConnections).toHaveLength(1);
    });
  });

  describe("connection dialog signal", () => {
    it("openConnectionDialog sets open flag", () => {
      useRecentStore.getState().openConnectionDialog();

      const state = useRecentStore.getState();
      expect(state.connectionDialogOpen).toBe(true);
      expect(state.connectionDialogEditId).toBeNull();
    });

    it("openConnectionDialog with editId sets edit target", () => {
      useRecentStore.getState().openConnectionDialog("conn-1");

      const state = useRecentStore.getState();
      expect(state.connectionDialogOpen).toBe(true);
      expect(state.connectionDialogEditId).toBe("conn-1");
    });

    it("closeConnectionDialog clears all dialog state", () => {
      useRecentStore.getState().openConnectionDialog("conn-1");
      useRecentStore.getState().closeConnectionDialog();

      const state = useRecentStore.getState();
      expect(state.connectionDialogOpen).toBe(false);
      expect(state.connectionDialogEditId).toBeNull();
    });
  });
});
