import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useRecentStore } from "@/commons/stores/recent.store";

function resetStore() {
  useRecentStore.setState({
    recentConnections: [],
    recentResources: [],
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

  describe("addRecentResource", () => {
    it("prepends a new resource entry", () => {
      useRecentStore.getState().addRecentResource({
        resourceKey: "dbobj:public.users:conn-1",
        kind: "db-object",
        connectionId: "conn-1",
        schema: "public",
        objectName: "users",
      });

      const { recentResources } = useRecentStore.getState();
      expect(recentResources).toHaveLength(1);
      expect(recentResources[0].resourceKey).toBe("dbobj:public.users:conn-1");
      expect(recentResources[0].openedAt).toBeTruthy();
    });

    it("moves existing resource to front without duplicating", () => {
      useRecentStore.getState().addRecentResource({
        resourceKey: "dbobj:public.users:conn-1",
        kind: "db-object",
        connectionId: "conn-1",
        schema: "public",
        objectName: "users",
      });
      useRecentStore.getState().addRecentResource({
        resourceKey: "dbobj:public.orders:conn-1",
        kind: "db-object",
        connectionId: "conn-1",
        schema: "public",
        objectName: "orders",
      });
      useRecentStore.getState().addRecentResource({
        resourceKey: "dbobj:public.users:conn-1",
        kind: "db-object",
        connectionId: "conn-1",
        schema: "public",
        objectName: "users",
      });

      const { recentResources } = useRecentStore.getState();
      expect(recentResources).toHaveLength(2);
      expect(recentResources[0].resourceKey).toBe("dbobj:public.users:conn-1");
      expect(recentResources[1].resourceKey).toBe("dbobj:public.orders:conn-1");
    });

    it("trims to MAX_RECENT_RESOURCES", () => {
      for (let i = 1; i <= 25; i++) {
        useRecentStore.getState().addRecentResource({
          resourceKey: `dbobj:public.t${i}:conn-1`,
          kind: "db-object",
          connectionId: "conn-1",
          schema: "public",
          objectName: `t${i}`,
        });
      }

      const { recentResources } = useRecentStore.getState();
      expect(recentResources).toHaveLength(20);
      expect(recentResources[0].objectName).toBe("t25");
    });
  });

  describe("removeRecentResource", () => {
    it("removes by resourceKey", () => {
      useRecentStore.getState().addRecentResource({
        resourceKey: "dbobj:public.users:conn-1",
        kind: "db-object",
        connectionId: "conn-1",
        schema: "public",
        objectName: "users",
      });

      useRecentStore.getState().removeRecentResource("dbobj:public.users:conn-1");

      expect(useRecentStore.getState().recentResources).toHaveLength(0);
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
