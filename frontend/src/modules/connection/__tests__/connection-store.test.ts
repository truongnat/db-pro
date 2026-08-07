import { beforeEach, describe, expect, it } from "vitest";

import { useConnectionModuleStore } from "../state/connection.store";

function resetStore() {
  useConnectionModuleStore.getState().reset();
}

describe("Connection module store", () => {
  beforeEach(() => {
    resetStore();
  });

  describe("status management", () => {
    it("starts with empty statuses", () => {
      expect(useConnectionModuleStore.getState().statuses).toEqual({});
    });

    it("sets status for a connection", () => {
      useConnectionModuleStore.getState().setStatus("c1", "connecting");
      expect(useConnectionModuleStore.getState().statuses["c1"]).toBe("connecting");
    });

    it("sets reconnecting status", () => {
      useConnectionModuleStore.getState().setStatus("c1", "reconnecting");
      expect(useConnectionModuleStore.getState().statuses["c1"]).toBe("reconnecting");
    });

    it("sets error for a connection", () => {
      useConnectionModuleStore.getState().setError("c1", "timeout");
      expect(useConnectionModuleStore.getState().connectionErrors["c1"]).toBe("timeout");
    });

    it("clears status and error for a connection", () => {
      useConnectionModuleStore.getState().setStatus("c1", "connected");
      useConnectionModuleStore.getState().setError("c1", "some error");
      useConnectionModuleStore.getState().clearStatus("c1");
      expect(useConnectionModuleStore.getState().statuses["c1"]).toBeUndefined();
      expect(useConnectionModuleStore.getState().connectionErrors["c1"]).toBeUndefined();
    });
  });

  describe("favorites", () => {
    it("starts with no favorites", () => {
      expect(useConnectionModuleStore.getState().favorites).toEqual({});
    });

    it("toggles favorite on", () => {
      useConnectionModuleStore.getState().toggleFavorite("c1");
      expect(useConnectionModuleStore.getState().favorites["c1"]).toBe(true);
    });

    it("toggles favorite off", () => {
      useConnectionModuleStore.getState().toggleFavorite("c1");
      useConnectionModuleStore.getState().toggleFavorite("c1");
      expect(useConnectionModuleStore.getState().favorites["c1"]).toBe(false);
    });

    it("isolates favorites per connection", () => {
      useConnectionModuleStore.getState().toggleFavorite("c1");
      useConnectionModuleStore.getState().toggleFavorite("c2");
      useConnectionModuleStore.getState().toggleFavorite("c1"); // c1 off
      expect(useConnectionModuleStore.getState().favorites["c1"]).toBe(false);
      expect(useConnectionModuleStore.getState().favorites["c2"]).toBe(true);
    });
  });

  describe("sorting", () => {
    it("defaults to name ascending", () => {
      expect(useConnectionModuleStore.getState().sortField).toBe("name");
      expect(useConnectionModuleStore.getState().sortDirection).toBe("asc");
    });

    it("changes sort field", () => {
      useConnectionModuleStore.getState().setSortField("driver");
      expect(useConnectionModuleStore.getState().sortField).toBe("driver");
    });

    it("changes sort direction", () => {
      useConnectionModuleStore.getState().setSortDirection("desc");
      expect(useConnectionModuleStore.getState().sortDirection).toBe("desc");
    });
  });

  describe("filtering", () => {
    it("defaults to no filters", () => {
      expect(useConnectionModuleStore.getState().filterTag).toBeNull();
      expect(useConnectionModuleStore.getState().filterGroup).toBeNull();
    });

    it("sets tag filter", () => {
      useConnectionModuleStore.getState().setFilterTag("production");
      expect(useConnectionModuleStore.getState().filterTag).toBe("production");
    });

    it("sets group filter", () => {
      useConnectionModuleStore.getState().setFilterGroup("staging");
      expect(useConnectionModuleStore.getState().filterGroup).toBe("staging");
    });

    it("clears all filters", () => {
      useConnectionModuleStore.getState().setFilterTag("prod");
      useConnectionModuleStore.getState().setFilterGroup("staging");
      useConnectionModuleStore.getState().clearFilters();
      expect(useConnectionModuleStore.getState().filterTag).toBeNull();
      expect(useConnectionModuleStore.getState().filterGroup).toBeNull();
    });
  });

  describe("reset", () => {
    it("resets all state", () => {
      useConnectionModuleStore.getState().setStatus("c1", "connected");
      useConnectionModuleStore.getState().setError("c1", "err");
      useConnectionModuleStore.getState().toggleFavorite("c1");
      useConnectionModuleStore.getState().setSortField("driver");
      useConnectionModuleStore.getState().setFilterTag("prod");

      useConnectionModuleStore.getState().reset();

      const state = useConnectionModuleStore.getState();
      expect(state.statuses).toEqual({});
      expect(state.connectionErrors).toEqual({});
      expect(state.favorites).toEqual({});
      expect(state.sortField).toBe("name");
      expect(state.sortDirection).toBe("asc");
      expect(state.filterTag).toBeNull();
      expect(state.filterGroup).toBeNull();
    });
  });
});
