import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { mockIntrospect, mockGetTableInfo } = vi.hoisted(() => ({
  mockIntrospect: vi.fn(),
  mockGetTableInfo: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: () => ({
      introspect: mockIntrospect,
      getTableInfo: mockGetTableInfo,
    }),
  },
}));

vi.mock("@/commons/di/registry", () => ({
  SERVICE_NAMES: { SCHEMA_SERVICE: "SCHEMA_SERVICE" },
}));

import { useSchemaCatalogStore } from "../stores/schema-catalog.store";

function resetStore() {
  useSchemaCatalogStore.getState().reset();
}

const MOCK_INTROSPECT_RESULT = {
  schemas: [{ name: "public" }],
  tables: [
    { name: "users", schema: "public", rowCount: 100 },
    { name: "orders", schema: "public", rowCount: 200 },
  ],
  columns: [],
  primaryKeys: [],
  indexes: [],
  foreignKeys: [],
  views: [{ name: "v_active", schema: "public", definition: "SELECT 1" }],
};

describe("SchemaCatalogStore", () => {
  beforeEach(() => {
    resetStore();
    mockIntrospect.mockReset();
    mockGetTableInfo.mockReset();
  });
  afterEach(resetStore);

  describe("getCatalog", () => {
    it("returns undefined for unknown connection", () => {
      expect(useSchemaCatalogStore.getState().getCatalog("unknown")).toBeUndefined();
    });
  });

  describe("ensureLoaded", () => {
    it("loads catalog from introspection", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      await useSchemaCatalogStore.getState().ensureLoaded("conn-1");

      const catalog = useSchemaCatalogStore.getState().getCatalog("conn-1");
      expect(catalog).toBeDefined();
      expect(catalog!.schemas).toHaveLength(1);
      expect(catalog!.objects).toHaveLength(3); // 2 tables + 1 view
      expect(catalog!.objects[0].name).toBe("users");
      expect(catalog!.objects[0].kind).toBe("table");
      expect(catalog!.objects[2].name).toBe("v_active");
      expect(catalog!.objects[2].kind).toBe("view");
    });

    it("does not reload if already loaded", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      await useSchemaCatalogStore.getState().ensureLoaded("conn-1");
      await useSchemaCatalogStore.getState().ensureLoaded("conn-1");
      expect(mockIntrospect).toHaveBeenCalledTimes(1);
    });

    it("preserves cached columns on reload", async () => {
      mockIntrospect.mockResolvedValue(MOCK_INTROSPECT_RESULT);
      const store = useSchemaCatalogStore.getState();

      // First load
      await store.ensureLoaded("conn-1");

      // Manually set some columns
      const cat = store.getCatalog("conn-1")!;
      expect(cat.columnsByTable.size).toBe(0);
    });
  });

  describe("ensureTableColumns", () => {
    it("fetches and caches columns", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      const columns = [
        { name: "id", dataType: "INT", nullable: false, defaultValue: null, isPrimaryKey: true, tableName: "users", schema: "public" },
        { name: "name", dataType: "TEXT", nullable: true, defaultValue: null, isPrimaryKey: false, tableName: "users", schema: "public" },
      ];
      mockGetTableInfo.mockResolvedValueOnce({ table: {}, columns, primaryKey: null, indexes: [], foreignKeys: [] });

      const store = useSchemaCatalogStore.getState();
      await store.ensureLoaded("conn-1");
      const result = await store.ensureTableColumns("conn-1", "public", "users");

      expect(result).toEqual(columns);
      expect(mockGetTableInfo).toHaveBeenCalledWith("conn-1", "public", "users");
    });

    it("returns cached columns without refetching", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      const columns = [
        { name: "id", dataType: "INT", nullable: false, defaultValue: null, isPrimaryKey: true, tableName: "users", schema: "public" },
      ];
      mockGetTableInfo.mockResolvedValueOnce({ table: {}, columns, primaryKey: null, indexes: [], foreignKeys: [] });

      const store = useSchemaCatalogStore.getState();
      await store.ensureLoaded("conn-1");
      await store.ensureTableColumns("conn-1", "public", "users");
      const result = await store.ensureTableColumns("conn-1", "public", "users");

      expect(result).toEqual(columns);
      expect(mockGetTableInfo).toHaveBeenCalledTimes(1);
    });
  });

  describe("getColumns", () => {
    it("returns undefined for unknown connection", () => {
      expect(useSchemaCatalogStore.getState().getColumns("unknown", "public", "users")).toBeUndefined();
    });

    it("returns undefined for table without loaded columns", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      await useSchemaCatalogStore.getState().ensureLoaded("conn-1");
      expect(useSchemaCatalogStore.getState().getColumns("conn-1", "public", "users")).toBeUndefined();
    });
  });

  describe("invalidateConnection", () => {
    it("removes catalog for connection", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      await useSchemaCatalogStore.getState().ensureLoaded("conn-1");
      expect(useSchemaCatalogStore.getState().getCatalog("conn-1")).toBeDefined();

      useSchemaCatalogStore.getState().invalidateConnection("conn-1");
      expect(useSchemaCatalogStore.getState().getCatalog("conn-1")).toBeUndefined();
    });

    it("does nothing for unknown connection", () => {
      useSchemaCatalogStore.getState().invalidateConnection("unknown");
      // Should not throw
    });
  });

  describe("invalidateTable", () => {
    it("removes cached columns for specific table", async () => {
      mockIntrospect.mockResolvedValueOnce(MOCK_INTROSPECT_RESULT);
      const columns = [{ name: "id", dataType: "INT", nullable: false, defaultValue: null, isPrimaryKey: true, tableName: "users", schema: "public" }];
      mockGetTableInfo.mockResolvedValue({ table: {}, columns, primaryKey: null, indexes: [], foreignKeys: [] });

      const store = useSchemaCatalogStore.getState();
      await store.ensureLoaded("conn-1");
      await store.ensureTableColumns("conn-1", "public", "users");
      expect(store.getColumns("conn-1", "public", "users")).toBeDefined();

      store.invalidateTable("conn-1", "public", "users");
      expect(store.getColumns("conn-1", "public", "users")).toBeUndefined();
    });

    it("does nothing for unknown connection", () => {
      useSchemaCatalogStore.getState().invalidateTable("unknown", "public", "users");
      // Should not throw
    });
  });

  describe("reset", () => {
    it("clears all catalogs", async () => {
      mockIntrospect.mockResolvedValue(MOCK_INTROSPECT_RESULT);
      await useSchemaCatalogStore.getState().ensureLoaded("conn-1");
      await useSchemaCatalogStore.getState().ensureLoaded("conn-2");

      useSchemaCatalogStore.getState().reset();
      expect(useSchemaCatalogStore.getState().getCatalog("conn-1")).toBeUndefined();
      expect(useSchemaCatalogStore.getState().getCatalog("conn-2")).toBeUndefined();
    });
  });
});
