/**
 * Dev-only browser backend.
 *
 * DB Pro is a Tauri desktop app: every screen talks to the Rust core over IPC.
 * That makes the UI impossible to review without a packaged desktop shell.
 *
 * This module installs `window.__TAURI_INTERNALS__.invoke` (via Tauri's own
 * `mockIPC` helper) so `vite dev` renders the real components against the
 * in-memory fixture database in `mock-database.ts`. Nothing here is reachable
 * from a production bundle:
 *
 *   - `installBrowserBackend()` is only called from `main.tsx` inside
 *     `import.meta.env.DEV`, which Rollup folds to `false` for `vite build`
 *     and dead-code-eliminates together with the dynamic import below.
 *   - Even in dev we refuse to install when a real Tauri IPC bridge is
 *     present, so `tauri dev` always uses the Rust core.
 */

import { mockIPC } from "@tauri-apps/api/mocks";

import type { CellValue } from "@/modules/query/types/query.types";
import type {
  QueryHistoryEntry,
  RunConfig,
  SavedQuery,
  SavedQueryFolder,
} from "@/modules/query/types/query.types";
import type { Connection, ConnectionConfig } from "@/modules/connection/types/connection.types";
import type {
  DataDiff,
  ObjectDependency,
  PartitionInfo,
  SchemaDiff,
  TablespaceInfo,
} from "@/modules/schema/types/schema.types";
import type {
  FetchRowsResult,
  GridFilter,
  GridSort,
  MutateRowResult,
} from "@/modules/data-grid/types/data-grid.types";
import type { ExportResult } from "@/modules/export/types/export.types";
import type { DatabaseUser, Privilege } from "@/modules/user-management/types/user.types";
import type { BackupResult } from "@/modules/backup/types/backup.types";
import type { MultiQueryResult, QueryResult } from "@/modules/query/types/query.types";

import {
  MOCK_TABLES,
  MockSqlError,
  buildExplainPlan,
  buildIntrospectResult,
  buildTableDdl,
  buildTableInfo,
  columnIndexOf,
  findTable,
  runStatement,
  toRow,
} from "./mock-database";

/* ------------------------------------------------------------------ */
/*  Fixture connections                                                */
/* ------------------------------------------------------------------ */

const NOW = "2026-09-10T08:00:00Z";

function makeConnection(
  partial: Partial<Connection> & Pick<Connection, "id" | "name">,
): Connection {
  return {
    host: "localhost",
    port: 5432,
    database: "acme",
    username: "postgres",
    driver: "postgres",
    sslMode: "disable",
    queryTimeoutMs: 30_000,
    maxRows: 5000,
    createdAt: NOW,
    updatedAt: NOW,
    ...partial,
  };
}

const CONNECTIONS: Connection[] = [
  makeConnection({
    id: "conn-production",
    name: "Acme Production",
    host: "db.acme.internal",
    database: "acme_prod",
    group: "Production",
    color: "#f05d6f",
    favorite: true,
    tags: ["prod", "read-only"],
    sslMode: "verify-full",
  }),
  makeConnection({
    id: "conn-staging",
    name: "Acme Staging",
    host: "staging-db.acme.dev",
    database: "acme_staging",
    group: "Staging",
    color: "#f5b942",
    tags: ["staging"],
  }),
  makeConnection({
    id: "conn-local-sqlite",
    name: "Local Dev (SQLite)",
    driver: "sqlite",
    host: "",
    port: 0,
    database: "./fixtures/sqlite/acme.db",
    group: "Local",
    color: "#39d98a",
  }),
];

const STATUSES = new Map<string, "connected" | "disconnected">();

/* ------------------------------------------------------------------ */
/*  Saved queries / folders / run configs                              */
/* ------------------------------------------------------------------ */

const SAVED_QUERIES: SavedQuery[] = [
  {
    id: "sq-1",
    connectionId: "conn-staging",
    name: "Top customers by spend",
    folder: "Reporting",
    tags: ["monthly", "revenue"],
    favorite: true,
    createdAt: NOW,
    sql: "SELECT u.id, u.name, COUNT(o.id) AS orders, SUM(o.total) AS spend\nFROM public.users u\nJOIN public.orders o ON o.user_id = u.id\nGROUP BY u.id, u.name\nORDER BY spend DESC\nLIMIT 25;",
  },
  {
    id: "sq-2",
    connectionId: "conn-staging",
    name: "Stale pending orders",
    folder: "Operations",
    tags: ["ops"],
    favorite: false,
    createdAt: NOW,
    sql: "SELECT id, user_id, total, created_at\nFROM public.orders\nWHERE status = 'pending'\nORDER BY created_at ASC;",
  },
  {
    id: "sq-3",
    connectionId: "conn-staging",
    name: "Out of stock products",
    tags: [],
    favorite: false,
    createdAt: NOW,
    sql: "SELECT id, name, category_id FROM public.products WHERE in_stock = false;",
  },
];

const FOLDERS: SavedQueryFolder[] = [
  { id: "f-1", connectionId: "conn-staging", name: "Reporting", createdAt: NOW },
  { id: "f-2", connectionId: "conn-staging", name: "Operations", createdAt: NOW },
];

const RUN_CONFIGS: RunConfig[] = [
  {
    id: "rc-1",
    connectionId: "conn-staging",
    name: "Nightly audit sweep",
    sql: "SELECT entity_type, COUNT(*) FROM public.audit_log GROUP BY entity_type;",
    timeoutMs: 60_000,
    maxRows: 10_000,
    createdAt: NOW,
  },
];

const HISTORY: QueryHistoryEntry[] = [
  {
    id: "h-1",
    connectionId: "conn-staging",
    sql: "SELECT * FROM public.orders ORDER BY created_at DESC LIMIT 50;",
    executedAt: "2026-09-10T07:42:11Z",
    durationMs: 18,
    rowCount: 12,
    status: "success",
    database: "acme_staging",
    schema: "public",
  },
  {
    id: "h-2",
    connectionId: "conn-staging",
    sql: "SELECT COUNT(*) FROM public.products WHERE in_stock = true;",
    executedAt: "2026-09-10T07:31:02Z",
    durationMs: 6,
    rowCount: 1,
    status: "success",
    database: "acme_staging",
    schema: "public",
  },
  {
    id: "h-3",
    connectionId: "conn-staging",
    sql: "SELECT * FROM public.orers;",
    executedAt: "2026-09-09T16:20:44Z",
    durationMs: 3,
    rowCount: 0,
    status: "error",
    database: "acme_staging",
    schema: "public",
  },
];

const USERS: DatabaseUser[] = [
  { name: "postgres", isSuper: true, canCreateDb: true, canCreateRole: true, canLogin: true },
  { name: "acme_app", isSuper: false, canCreateDb: false, canCreateRole: false, canLogin: true },
  {
    name: "acme_readonly",
    isSuper: false,
    canCreateDb: false,
    canCreateRole: false,
    canLogin: true,
  },
  { name: "analytics", isSuper: false, canCreateDb: true, canCreateRole: false, canLogin: true },
];

const PRIVILEGES: Privilege[] = [
  { schema: "public", table: "orders", privilegeType: "SELECT" },
  { schema: "public", table: "orders", privilegeType: "INSERT" },
  { schema: "public", table: "users", privilegeType: "SELECT" },
  { schema: "public", table: "products", privilegeType: "ALL" },
];

/* ------------------------------------------------------------------ */
/*  Helpers                                                            */
/* ------------------------------------------------------------------ */

type Args = Record<string, unknown> | undefined;

function str(args: Args, key: string, fallback = ""): string {
  const value = args?.[key];
  return typeof value === "string" ? value : fallback;
}

function cellToRaw(cell: unknown): string | number | boolean | null | Record<string, unknown> {
  if (!cell || typeof cell !== "object") return cell == null ? null : (cell as string);
  const typed = cell as { type?: string; value?: unknown };
  if (typed.type === "null") return null;
  if (typed.value === undefined || typed.value === null) return null;
  if (typeof typed.value === "object") return typed.value as Record<string, unknown>;
  return typed.value as string | number | boolean;
}

function commandError(error: string, message: string, messageId: string) {
  return { error, message, message_id: messageId, details: null };
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function requireConnection(connectionId: string): void {
  if (!CONNECTIONS.some((c) => c.id === connectionId)) {
    throw commandError(
      "DB_CONNECTION_NOT_FOUND",
      `Unknown connection: ${connectionId}`,
      "error.notFound",
    );
  }
  if (STATUSES.get(connectionId) !== "connected") {
    throw commandError(
      "DB_CONNECTION_LOST",
      `Connection ${connectionId} is not connected`,
      "error.connectionFailed",
    );
  }
}

function fetchTableRows(args: Args): FetchRowsResult {
  const connectionId = str(args, "connectionId");
  requireConnection(connectionId);

  const request = (args?.request ?? {}) as {
    schema?: string;
    table?: string;
    filters?: (GridFilter & { value: unknown })[];
    sorts?: GridSort[];
    page?: number;
    pageSize?: number;
  };

  const table = findTable(request.schema ?? "public", request.table ?? "");
  if (!table) {
    throw commandError("NOT_FOUND", `relation "${request.table}" does not exist`, "error.notFound");
  }

  const filters = request.filters ?? [];
  const sorts = request.sorts ?? [];
  const page = Math.max(0, request.page ?? 0);
  const pageSize = Math.max(1, request.pageSize ?? 50);

  let rows = table.rows.filter((row) =>
    filters.every((filter) => {
      const idx = columnIndexOf(table, filter.column);
      if (idx === -1) return true;
      const value = row[idx];
      const raw = cellToRaw(filter.value);
      switch (filter.op) {
        case "isNull":
          return value === null;
        case "isNotNull":
          return value !== null;
        case "eq":
          return String(value) === String(raw);
        case "neq":
          return String(value) !== String(raw);
        case "lt":
          return Number(value) < Number(raw);
        case "lte":
          return Number(value) <= Number(raw);
        case "gt":
          return Number(value) > Number(raw);
        case "gte":
          return Number(value) >= Number(raw);
        case "like":
          return String(value ?? "")
            .toLowerCase()
            .includes(String(raw ?? "").toLowerCase());
        default:
          return true;
      }
    }),
  );

  for (const sort of [...sorts].reverse()) {
    const idx = columnIndexOf(table, sort.column);
    if (idx === -1) continue;
    rows = [...rows].sort((a, b) => {
      const av = a[idx];
      const bv = b[idx];
      if (av === bv) return 0;
      if (av === null) return 1;
      if (bv === null) return -1;
      if (typeof av === "number" && typeof bv === "number") return av - bv;
      return String(av).localeCompare(String(bv));
    });
    if (sort.direction === "desc") rows.reverse();
  }

  const totalCount = rows.length;
  const paged = rows.slice(page * pageSize, page * pageSize + pageSize);

  return {
    columns: table.columns.map((c) => ({
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
    })),
    rows: paged.map((raw) => toRow(table, raw)),
    totalCount,
    durationMs: 4,
  };
}

function mutateRow(args: Args): MutateRowResult {
  requireConnection(str(args, "connectionId"));
  // Row mutations are acknowledged but not persisted to the fixture dataset so
  // repeated grid edits never corrupt the demo data across reloads.
  return { affectedRows: 1 };
}

function exportResult(args: Args, format: "csv" | "json" | "excel"): ExportResult {
  const result: QueryResult = runStatement(str(args, "sql", "SELECT 1"));
  const headers = result.columns.map((c) => c.name);

  if (format === "json") {
    const payload = result.rows.map((row) =>
      Object.fromEntries(
        headers.map((name, i) => {
          const cell = row[i];
          return [name, cell && cell.type === "null" ? null : (cell?.value ?? null)];
        }),
      ),
    );
    return {
      fileContent: JSON.stringify(payload, null, 2),
      fileName: "export.json",
      mimeType: "application/json",
      rowCount: result.rowCount,
    };
  }

  const csv = [
    headers.join(","),
    ...result.rows.map((row) =>
      row
        .map((cell) => {
          if (!cell || cell.type === "null") return "";
          const value = String(cell.value ?? "");
          return /[",\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
        })
        .join(","),
    ),
  ].join("\n");

  if (format === "excel") {
    return {
      fileContent: btoa(unescape(encodeURIComponent(csv))),
      fileName: "export.xlsx",
      mimeType: "application/vnd.ms-excel",
      rowCount: result.rowCount,
    };
  }

  return {
    fileContent: csv,
    fileName: "export.csv",
    mimeType: "text/csv",
    rowCount: result.rowCount,
  };
}

/* ------------------------------------------------------------------ */
/*  Command router                                                     */
/* ------------------------------------------------------------------ */

async function handle(cmd: string, args: Args): Promise<unknown> {
  switch (cmd) {
    /* -- app lifecycle -- */
    case "finish_startup":
      return null;

    /* -- connection management -- */
    case "list_connections":
      return [...CONNECTIONS];
    case "get_connection":
      return CONNECTIONS.find((c) => c.id === str(args, "id")) ?? null;
    case "create_connection": {
      const config = args?.config as ConnectionConfig | undefined;
      if (!config)
        throw commandError("VALIDATION_ERROR", "Missing connection config", "error.validation");
      const connection = makeConnection({
        id: `conn-${Math.random().toString(36).slice(2, 8)}`,
        name: config.name,
        host: config.host,
        port: config.port,
        database: config.database,
        username: config.username,
        driver: config.driver,
        sslMode: config.sslMode,
        group: config.group,
        color: config.color,
        tags: config.tags,
        favorite: config.favorite,
        readonly: config.readonly,
        queryTimeoutMs: config.queryTimeoutMs,
        maxRows: config.maxRows,
      });
      CONNECTIONS.push(connection);
      return connection;
    }
    case "update_connection": {
      const id = str(args, "id");
      const config = args?.config as ConnectionConfig | undefined;
      const existing = CONNECTIONS.find((c) => c.id === id);
      if (!existing || !config) return null;
      Object.assign(existing, {
        name: config.name,
        host: config.host,
        port: config.port,
        database: config.database,
        username: config.username,
        driver: config.driver,
        sslMode: config.sslMode,
        group: config.group,
        color: config.color,
        tags: config.tags,
        favorite: config.favorite,
        readonly: config.readonly,
        updatedAt: new Date().toISOString(),
      });
      return null;
    }
    case "delete_connection": {
      const idx = CONNECTIONS.findIndex((c) => c.id === str(args, "id"));
      if (idx >= 0) CONNECTIONS.splice(idx, 1);
      return null;
    }
    case "test_connection": {
      await sleep(350);
      const config = args?.config as ConnectionConfig | undefined;
      if (config?.username === "nobody") {
        throw commandError(
          "DB_AUTH_FAILED",
          'password authentication failed for user "nobody"',
          "error.authFailed",
        );
      }
      return null;
    }
    case "connect": {
      await sleep(300);
      STATUSES.set(str(args, "id"), "connected");
      return null;
    }
    case "disconnect":
      STATUSES.set(str(args, "id"), "disconnected");
      return null;
    case "test_ssh_tunnel":
      await sleep(400);
      return null;

    /* -- introspection -- */
    case "introspect": {
      requireConnection(str(args, "connectionId"));
      await sleep(120);
      return buildIntrospectResult();
    }
    case "get_table_info": {
      const info = buildTableInfo(str(args, "schema"), str(args, "table"));
      if (!info) throw commandError("NOT_FOUND", "Table not found", "error.notFound");
      return info;
    }
    case "get_table_ddl": {
      const ddl = buildTableDdl(str(args, "schema"), str(args, "table"));
      if (!ddl) throw commandError("NOT_FOUND", "Table not found", "error.notFound");
      return ddl;
    }
    case "invalidate_cache":
      return null;
    case "execute_ddl":
    case "execute_ddl_batch":
      return { affectedRows: 0 };
    case "diff_schemas":
      return {
        tablesOnlyInSource: ["audit_log"],
        tablesOnlyInTarget: [],
        columnDiffs: [
          {
            schema: "public",
            table: "users",
            columnsOnlyInSource: ["bio"],
            columnsOnlyInTarget: [],
            typeMismatches: [{ column: "name", sourceType: "varchar(255)", targetType: "text" }],
          },
        ],
        indexesOnlyInSource: ["idx_users_is_active"],
        indexesOnlyInTarget: [],
      } satisfies SchemaDiff;
    case "diff_table_data": {
      const table = findTable(str(args, "schema"), str(args, "table"));
      const count = table?.rows.length ?? 0;
      return {
        schema: str(args, "schema"),
        table: str(args, "table"),
        sourceRowCount: count,
        targetRowCount: Math.max(0, count - 2),
        rowCountDiff: 2,
      } satisfies DataDiff;
    }
    case "get_object_dependencies":
      return [
        {
          objectType: "table",
          objectName: "orders",
          dependsOnType: "table",
          dependsOnName: "users",
        },
        {
          objectType: "view",
          objectName: "product_catalog",
          dependsOnType: "table",
          dependsOnName: "products",
        },
      ] satisfies ObjectDependency[];
    case "list_partitions":
      return [
        {
          schema: "public",
          table: "audit_log",
          partitionStrategy: "RANGE",
          partitions: [
            { name: "audit_log_2026_q1", boundExpr: "FROM ('2026-01-01') TO ('2026-04-01')" },
            { name: "audit_log_2026_q2", boundExpr: "FROM ('2026-04-01') TO ('2026-07-01')" },
          ],
        },
      ] satisfies PartitionInfo[];
    case "list_tablespaces":
      return [
        { name: "pg_default", owner: "postgres", location: "" },
        { name: "analytics_tbs", owner: "analytics", location: "/var/lib/postgresql/analytics" },
      ] satisfies TablespaceInfo[];
    case "rename_schema_object":
      return null;

    /* -- query execution -- */
    case "execute_query": {
      requireConnection(str(args, "connectionId"));
      await sleep(160);
      try {
        return runStatement(str(args, "sql"));
      } catch (error) {
        throw toCommandError(error);
      }
    }
    case "execute_query_multi": {
      requireConnection(str(args, "connectionId"));
      await sleep(200);
      const statements = str(args, "sql")
        .split(";")
        .map((s) => s.trim())
        .filter(Boolean);
      const results: QueryResult[] = [];
      let failure: [number, string] | null = null;
      for (const [index, statement] of statements.entries()) {
        try {
          results.push(runStatement(statement));
        } catch (error) {
          failure = [index, error instanceof Error ? error.message : String(error)];
          break;
        }
      }
      return {
        results,
        totalDurationMs: results.reduce((sum, r) => sum + r.durationMs, 0),
        error: failure,
      } satisfies MultiQueryResult;
    }
    case "cancel_query":
      return null;
    case "explain_query": {
      requireConnection(str(args, "connectionId"));
      await sleep(140);
      return buildExplainPlan(str(args, "sql"));
    }
    case "get_query_history":
      return HISTORY.filter((h) => h.connectionId === str(args, "connectionId"));
    case "save_query": {
      const saved: SavedQuery = {
        id: `sq-${Math.random().toString(36).slice(2, 8)}`,
        connectionId: str(args, "connectionId"),
        name: str(args, "name", "Untitled query"),
        sql: str(args, "sql"),
        folder: str(args, "folder") || undefined,
        tags: [],
        favorite: false,
        createdAt: new Date().toISOString(),
      };
      SAVED_QUERIES.push(saved);
      return saved;
    }
    case "list_saved_queries":
      return SAVED_QUERIES.filter((q) => q.connectionId === str(args, "connectionId"));
    case "delete_saved_query": {
      const idx = SAVED_QUERIES.findIndex((q) => q.id === str(args, "id"));
      if (idx >= 0) SAVED_QUERIES.splice(idx, 1);
      return null;
    }
    case "rename_saved_query": {
      const target = SAVED_QUERIES.find((q) => q.id === str(args, "id"));
      if (target) target.name = str(args, "name");
      return null;
    }
    case "create_folder": {
      const folder: SavedQueryFolder = {
        id: `f-${Math.random().toString(36).slice(2, 8)}`,
        connectionId: str(args, "connectionId"),
        name: str(args, "name"),
        createdAt: new Date().toISOString(),
      };
      FOLDERS.push(folder);
      return folder;
    }
    case "list_folders":
      return FOLDERS.filter((f) => f.connectionId === str(args, "connectionId"));
    case "delete_folder": {
      const idx = FOLDERS.findIndex((f) => f.id === str(args, "id"));
      if (idx >= 0) FOLDERS.splice(idx, 1);
      return null;
    }
    case "save_run_config": {
      const config: RunConfig = {
        id: `rc-${Math.random().toString(36).slice(2, 8)}`,
        connectionId: str(args, "connectionId"),
        name: str(args, "name"),
        sql: str(args, "sql"),
        timeoutMs: Number(args?.timeoutMs ?? 30_000),
        maxRows: Number(args?.maxRows ?? 1000),
        createdAt: new Date().toISOString(),
      };
      RUN_CONFIGS.push(config);
      return config;
    }
    case "list_run_configs":
      return RUN_CONFIGS.filter((r) => r.connectionId === str(args, "connectionId"));
    case "delete_run_config": {
      const idx = RUN_CONFIGS.findIndex((r) => r.id === str(args, "id"));
      if (idx >= 0) RUN_CONFIGS.splice(idx, 1);
      return null;
    }

    /* -- data grid -- */
    case "fetch_table_rows":
      return fetchTableRows(args);
    case "insert_table_row":
    case "update_table_row":
    case "delete_table_row":
      return mutateRow(args);

    /* -- export -- */
    case "export_csv":
      return exportResult(args, "csv");
    case "export_json":
      return exportResult(args, "json");
    case "export_excel":
      return exportResult(args, "excel");

    /* -- user management -- */
    case "list_users":
      return USERS;
    case "list_privileges":
      return PRIVILEGES;
    case "create_role":
      USERS.push({
        name: String((args?.req as { name?: string } | undefined)?.name ?? "new_role"),
        isSuper: false,
        canCreateDb: false,
        canCreateRole: false,
        canLogin: Boolean((args?.req as { login?: boolean } | undefined)?.login),
      });
      return null;
    case "drop_role": {
      const name = (args?.req as { name?: string } | undefined)?.name;
      const idx = USERS.findIndex((u) => u.name === name);
      if (idx >= 0) USERS.splice(idx, 1);
      return null;
    }
    case "grant_privilege":
    case "revoke_privilege":
      return null;

    /* -- backup / restore -- */
    case "backup_database": {
      await sleep(600);
      const req = args?.req as { outputPath?: string } | undefined;
      return {
        outputPath: req?.outputPath ?? "/tmp/acme_backup.sql",
        sizeBytes: 1_284_512,
      } satisfies BackupResult;
    }
    case "restore_database":
      await sleep(600);
      return null;
    case "reveal_backup_path":
      return null;

    default:
      // Tauri plugin bridges (dialog/fs/event) — acknowledge so pickers simply cancel.
      if (cmd.startsWith("plugin:")) return null;
      throw commandError("UNKNOWN", `Browser backend has no handler for "${cmd}"`, "error.unknown");
  }
}

function toCommandError(error: unknown) {
  if (error instanceof MockSqlError) {
    return commandError(error.error, error.message, error.message_id);
  }
  if (error instanceof Error) {
    return commandError("INTERNAL_ERROR", error.message, "error.internal");
  }
  return commandError("UNKNOWN", String(error), "error.unknown");
}

/* ------------------------------------------------------------------ */
/*  Installation                                                       */
/* ------------------------------------------------------------------ */

function hasRealTauriBridge(): boolean {
  const internals = (window as unknown as { __TAURI_INTERNALS__?: { invoke?: unknown } })
    .__TAURI_INTERNALS__;
  return typeof internals?.invoke === "function";
}

/**
 * Installs the fixture backend when, and only when, we are running in a plain
 * browser with no Tauri IPC bridge. Returns `true` when installed.
 */
export function installBrowserBackend(): boolean {
  if (hasRealTauriBridge()) return false;

  // Seed a connected state so the workspace opens straight into live data.
  STATUSES.set("conn-staging", "connected");

  mockIPC(
    async (cmd, payload) => {
      try {
        return await handle(cmd, payload as Args);
      } catch (error) {
        // `handle` already throws Rust-shaped error objects; only wrap the
        // unexpected ones or the code/message_id would be flattened to UNKNOWN.
        throw error instanceof Error ? toCommandError(error) : error;
      }
    },
    { shouldMockEvents: true },
  );

  console.info(
    `[db-pro] browser backend active — ${MOCK_TABLES.length} fixture relations, no Rust core`,
  );
  return true;
}

/** Value type helper re-exported for tests that build filter values. */
export type { CellValue };
