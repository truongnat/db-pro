import { create } from "zustand";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type ISchemaService } from "@/commons/di/registry";
import type {
  IntrospectResult,
  SchemaColumnDto,
  TableInfo,
} from "@/modules/schema/types/schema.types";

function getSchemaService() {
  return container.resolve<ISchemaService>(SERVICE_NAMES.SCHEMA_SERVICE);
}

export interface CatalogTableEntry {
  name: string;
  schema: string;
  rowCount: number | null;
  kind: "table" | "view";
}

export interface ConnectionCatalog {
  schemas: { name: string }[];
  objects: CatalogTableEntry[];
  columnsByTable: Map<string, SchemaColumnDto[]>;
  columnsLoaded: Set<string>;
  columnsLoading: Map<string, Promise<SchemaColumnDto[]>>;
}interface SchemaCatalogState {
  catalogs: Map<string, ConnectionCatalog>;
  ensureLoaded: (connectionId: string) => Promise<void>;
  ensureTableColumns: (connectionId: string, schema: string, table: string) => Promise<SchemaColumnDto[]>;
  getColumns: (connectionId: string, schema: string, table: string) => SchemaColumnDto[] | undefined;
  getCatalog: (connectionId: string) => ConnectionCatalog | undefined;
  invalidateConnection: (connectionId: string) => void;
  invalidateTable: (connectionId: string, schema: string, table: string) => void;
  reset: () => void;
}

function tableKey(schema: string, table: string): string {
  return `${schema}.${table}`;
}

function createEmptyCatalog(): ConnectionCatalog {
  return {
    schemas: [],
    objects: [],
    columnsByTable: new Map(),
    columnsLoaded: new Set(),
    columnsLoading: new Map(),
  };
}

export const useSchemaCatalogStore = create<SchemaCatalogState>()((set, get) => ({
  catalogs: new Map(),

  ensureLoaded: async (connectionId) => {
    const existing = get().catalogs.get(connectionId);
    if (existing && existing.schemas.length > 0) return;

    const result = await getSchemaService().introspect(connectionId) as IntrospectResult;

    const objects: CatalogTableEntry[] = [
      ...result.tables.map((t) => ({ name: t.name, schema: t.schema, rowCount: t.rowCount, kind: "table" as const })),
      ...result.views.map((v) => ({ name: v.name, schema: v.schema, rowCount: null, kind: "view" as const })),
    ];

    const catalog: ConnectionCatalog = {
      schemas: result.schemas,
      objects,
      columnsByTable: existing?.columnsByTable ?? new Map(),
      columnsLoaded: existing?.columnsLoaded ?? new Set(),
      columnsLoading: existing?.columnsLoading ?? new Map(),
    };

    set((state) => {
      const next = new Map(state.catalogs);
      next.set(connectionId, catalog);
      return { catalogs: next };
    });
  },

  ensureTableColumns: async (connectionId, schema, table) => {
    const catalog = get().catalogs.get(connectionId);
    const key = tableKey(schema, table);

    if (catalog?.columnsLoaded.has(key)) {
      return catalog.columnsByTable.get(key) ?? [];
    }

    const inFlight = catalog?.columnsLoading.get(key);
    if (inFlight) return inFlight;

    const fetchPromise = (async () => {
      const info = await getSchemaService().getTableInfo(connectionId, schema, table) as TableInfo;
      const columns = info.columns;

      set((state) => {
        const cat = state.catalogs.get(connectionId);
        if (!cat) return state;
        const next = new Map(state.catalogs);
        const updated = { ...cat };
        updated.columnsByTable = new Map(cat.columnsByTable);
        updated.columnsByTable.set(key, columns);
        updated.columnsLoaded = new Set(cat.columnsLoaded);
        updated.columnsLoaded.add(key);
        updated.columnsLoading = new Map(cat.columnsLoading);
        updated.columnsLoading.delete(key);
        next.set(connectionId, updated);
        return { catalogs: next };
      });

      return columns;
    })();

    set((state) => {
      const cat = state.catalogs.get(connectionId);
      if (!cat) return state;
      const next = new Map(state.catalogs);
      const updated = { ...cat };
      updated.columnsLoading = new Map(cat.columnsLoading);
      updated.columnsLoading.set(key, fetchPromise);
      next.set(connectionId, updated);
      return { catalogs: next };
    });

    return fetchPromise;
  },

  getColumns: (connectionId, schema, table) => {
    const catalog = get().catalogs.get(connectionId);
    if (!catalog) return undefined;
    return catalog.columnsByTable.get(tableKey(schema, table));
  },

  getCatalog: (connectionId) => {
    return get().catalogs.get(connectionId);
  },

  invalidateConnection: (connectionId) => {
    set((state) => {
      const next = new Map(state.catalogs);
      next.delete(connectionId);
      return { catalogs: next };
    });
  },

  invalidateTable: (connectionId, schema, table) => {
    set((state) => {
      const cat = state.catalogs.get(connectionId);
      if (!cat) return state;
      const key = tableKey(schema, table);
      const next = new Map(state.catalogs);
      const updated = { ...cat };
      updated.columnsByTable = new Map(cat.columnsByTable);
      updated.columnsByTable.delete(key);
      updated.columnsLoaded = new Set(cat.columnsLoaded);
      updated.columnsLoaded.delete(key);
      next.set(connectionId, updated);
      return { catalogs: next };
    });
  },

  reset: () => set({ catalogs: new Map() }),
}));
