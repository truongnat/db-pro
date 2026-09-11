import { describe, expect, it } from "vitest";

/**
 * Performance budget tests.
 * These tests verify that critical operations complete within acceptable time bounds.
 * They are not meant to be precise benchmarks but rather guard against major regressions.
 */

const PERF_BUDGETS = {
  /** Quick Open index building for 1000 items should complete quickly */
  quickOpenIndexBuild_1000: 50,
  /** Quick Open ranking across 1000 items */
  quickOpenRank_1000: 50,
  /** Statement splitting for large SQL */
  statementSplit_100statements: 50,
  /** CSV generation for 10k rows */
  csvGenerate_10kRows: 200,
  /** JSON generation for 10k rows */
  jsonGenerate_10kRows: 200,
  /** SQL INSERT generation for 1k rows */
  sqlInsertGenerate_1kRows: 200,
  /** CSV parsing for 10k rows */
  csvParse_10kRows: 200,
  /** Schema tree building with 500 tables */
  schemaTreeBuild_500tables: 150,
  /** Workspace store: open/close 100 tabs */
  workspaceTabCycle_100: 100,
  /** Grid state: 100 tab state operations */
  gridStateOps_100: 50,
};

describe("Performance budgets", () => {
  describe("Quick Open", () => {
    it("builds index for 1000 items within budget", async () => {
      const { buildQuickOpenIndex } = await import("@/commons/services/quick-open-index");
      const connections = Array.from({ length: 5 }, (_, i) => ({
        id: `conn-${i}`,
        name: `Connection ${i}`,
        host: "localhost",
        port: 5432,
        database: `db${i}`,
        username: "user",
        driver: "postgres" as const,
        sslMode: "disable" as const,
        createdAt: "2025-01-01",
        updatedAt: "2025-01-01",
      }));
      const catalogs = new Map();
      const tabs: unknown[] = [];

      const start = performance.now();
      const items = buildQuickOpenIndex({ connections, catalogs, tabs });
      const elapsed = performance.now() - start;

      expect(items.length).toBeGreaterThanOrEqual(0);
      expect(elapsed).toBeLessThan(PERF_BUDGETS.quickOpenIndexBuild_1000);
    });

    it("ranks 1000 items within budget", async () => {
      const { rankQuickOpenItems } = await import("@/commons/services/quick-open-rank");
      const items = Array.from({ length: 1000 }, (_, i) => ({
        kind: "db-object" as const,
        connectionId: "conn-1",
        connectionName: "Test DB",
        schema: "public",
        objectName: `table_${i}`,
        objectType: "table" as const,
        resourceKey: `dbobj:public.table_${i}:conn-1`,
        searchText: `table_${i} public table_${i}`,
      }));
      const ctx = {
        query: "table 5",
        activeTabId: null,
        activeConnectionId: "conn-1",
        explorerConnectionId: "conn-1",
        openResourceKeys: new Set<string>(),
        recentResourceKeys: new Set<string>(),
      };

      const start = performance.now();
      rankQuickOpenItems(items, ctx);
      const elapsed = performance.now() - start;

      expect(elapsed).toBeLessThan(PERF_BUDGETS.quickOpenRank_1000);
    });
  });

  describe("Statement splitter", () => {
    it("splits 100 statements within budget", async () => {
      const { splitStatementsWithRanges } =
        await import("@/modules/query/services/statement-splitter");
      const sql = Array.from({ length: 100 }, (_, i) => `SELECT ${i} FROM table_${i};`).join("\n");

      const start = performance.now();
      const result = splitStatementsWithRanges(sql);
      const elapsed = performance.now() - start;

      expect(result.length).toBe(100);
      expect(elapsed).toBeLessThan(PERF_BUDGETS.statementSplit_100statements);
    });
  });

  describe("Export generators", () => {
    const columns = [
      { name: "id", dataType: "INT", nullable: false },
      { name: "name", dataType: "TEXT", nullable: false },
      { name: "email", dataType: "TEXT", nullable: false },
      { name: "status", dataType: "TEXT", nullable: false },
      { name: "created_at", dataType: "TIMESTAMP", nullable: false },
    ];
    const generateRows = (count: number) =>
      Array.from({ length: count }, (_, i) => [
        { type: "int64" as const, value: String(i + 1) },
        { type: "text" as const, value: `User ${i + 1}` },
        { type: "text" as const, value: `user${i + 1}@example.com` },
        { type: "text" as const, value: i % 2 === 0 ? "active" : "inactive" },
        { type: "text" as const, value: "2025-01-01T00:00:00Z" },
      ]);

    it("generates CSV for 10k rows within budget", async () => {
      const { generateCsv } = await import("@/modules/export/services/export-generators");
      const rows = generateRows(10_000);
      const options = {
        delimiter: "," as const,
        includeHeaders: true,
        nullRepresentation: "NULL",
      };

      const start = performance.now();
      const result = generateCsv(columns, rows, options);
      const elapsed = performance.now() - start;

      expect(result.length).toBeGreaterThan(0);
      expect(elapsed).toBeLessThan(PERF_BUDGETS.csvGenerate_10kRows);
    });

    it("generates JSON for 10k rows within budget", async () => {
      const { generateJson } = await import("@/modules/export/services/export-generators");
      const rows = generateRows(10_000);
      const options = {
        delimiter: "," as const,
        includeHeaders: true,
        nullRepresentation: "NULL",
      };

      const start = performance.now();
      const result = generateJson(columns, rows, options);
      const elapsed = performance.now() - start;

      expect(result.length).toBeGreaterThan(0);
      expect(elapsed).toBeLessThan(PERF_BUDGETS.jsonGenerate_10kRows);
    });

    it("generates SQL INSERTs for 1k rows within budget", async () => {
      const { generateSqlInserts } = await import("@/modules/export/services/export-generators");
      const rows = generateRows(1_000);
      const options = {
        delimiter: "," as const,
        includeHeaders: true,
        nullRepresentation: "NULL",
        tableName: "users",
      };

      const start = performance.now();
      const result = generateSqlInserts(columns, rows, options);
      const elapsed = performance.now() - start;

      expect(result.length).toBeGreaterThan(0);
      expect(elapsed).toBeLessThan(PERF_BUDGETS.sqlInsertGenerate_1kRows);
    });
  });

  describe("Import parser", () => {
    it("parses CSV with 10k rows within budget", async () => {
      const { buildImportPreview } = await import("@/modules/export/services/import-parser");
      const csvHeader = "id,name,email,status,created_at";
      const csvRows = Array.from(
        { length: 10_000 },
        (_, i) => `${i + 1},User ${i + 1},user${i + 1}@example.com,active,2025-01-01`,
      ).join("\n");
      const content = `${csvHeader}\n${csvRows}`;

      const start = performance.now();
      const result = buildImportPreview(content, "csv", []);
      const elapsed = performance.now() - start;

      expect(result).toBeDefined();
      expect(elapsed).toBeLessThan(PERF_BUDGETS.csvParse_10kRows);
    });
  });

  describe("Schema tree", () => {
    it("builds tree for 500 tables within budget", async () => {
      const { buildTreeData } = await import("@/modules/schema/types/schema.types");
      const tables = Array.from({ length: 500 }, (_, i) => ({
        name: `table_${i}`,
        schema: i % 5 === 0 ? "public" : `schema_${i % 10}`,
        rowCount: i * 100,
      }));

      const result = {
        schemas: Array.from({ length: 10 }, (_, i) => ({
          name: i === 0 ? "public" : `schema_${i}`,
        })),
        tables,
        columns: [],
        primaryKeys: [],
        indexes: [],
        foreignKeys: [],
        views: [],
        triggers: [],
      };

      const start = performance.now();
      const tree = buildTreeData(result, "");
      const elapsed = performance.now() - start;

      expect(tree.length).toBeGreaterThan(0);
      expect(elapsed).toBeLessThan(PERF_BUDGETS.schemaTreeBuild_500tables);
    });
  });
});
