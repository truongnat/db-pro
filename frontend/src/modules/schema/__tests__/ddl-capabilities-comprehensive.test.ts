import { describe, expect, it } from "vitest";
import { getSqlDialect } from "@/modules/query/sql/dialect";
import {
  getDdlCapabilities,
  getDdlCapabilitiesForDialect,
  checkOperationSupported,
  buildSqliteTableRebuild,
  type TableRebuildInput,
} from "../services/ddl-capabilities";

describe("DDL Capabilities — per-driver", () => {
  describe("PostgreSQL capabilities", () => {
    const caps = getDdlCapabilities("postgres");

    it("supports all native DDL operations", () => {
      expect(caps.supportsDropColumn).toBe(true);
      expect(caps.supportsAlterColumn).toBe(true);
      expect(caps.supportsRenameTable).toBe(true);
      expect(caps.supportsRenameColumn).toBe(true);
      expect(caps.supportsAddColumnWithConstraint).toBe(true);
      expect(caps.supportsTransactionalDdl).toBe(true);
      expect(caps.supportsIdentity).toBe(true);
      expect(caps.supportsTriggers).toBe(true);
    });

    it("does not require table rebuild for any operation", () => {
      expect(caps.requiresTableRebuild).toEqual([]);
    });
  });

  describe("SQLite capabilities", () => {
    const caps = getDdlCapabilities("sqlite");

    it("does not support ALTER COLUMN", () => {
      expect(caps.supportsAlterColumn).toBe(false);
    });

    it("does not support ADD COLUMN with constraints", () => {
      expect(caps.supportsAddColumnWithConstraint).toBe(false);
    });

    it("does not support transactional DDL", () => {
      expect(caps.supportsTransactionalDdl).toBe(false);
    });

    it("does not support IDENTITY columns", () => {
      expect(caps.supportsIdentity).toBe(false);
    });

    it("requires table rebuild for alterColumn and addForeignKey", () => {
      expect(caps.requiresTableRebuild).toContain("alterColumn");
      expect(caps.requiresTableRebuild).toContain("addForeignKey");
    });

    it("supports basic operations", () => {
      expect(caps.supportsDropColumn).toBe(true);
      expect(caps.supportsRenameTable).toBe(true);
      expect(caps.supportsRenameColumn).toBe(true);
      expect(caps.supportsTriggers).toBe(true);
    });
  });

  describe("getDdlCapabilitiesForDialect", () => {
    it("returns postgres capabilities for postgres dialect", () => {
      const pg = getSqlDialect("postgres");
      const caps = getDdlCapabilitiesForDialect(pg);
      expect(caps.supportsTransactionalDdl).toBe(true);
    });

    it("returns sqlite capabilities for sqlite dialect", () => {
      const sqlite = getSqlDialect("sqlite");
      const caps = getDdlCapabilitiesForDialect(sqlite);
      expect(caps.supportsTransactionalDdl).toBe(false);
    });
  });
});

describe("checkOperationSupported", () => {
  describe("PostgreSQL", () => {
    it("all operations are supported", () => {
      const ops = [
        "dropColumn",
        "alterColumn",
        "renameTable",
        "renameColumn",
        "addForeignKey",
        "addConstraint",
      ];
      for (const op of ops) {
        const result = checkOperationSupported("postgres", op);
        expect(result.supported, `Expected ${op} to be supported`).toBe(true);
      }
    });
  });

  describe("SQLite", () => {
    it("alterColumn is not supported and requires rebuild", () => {
      const result = checkOperationSupported("sqlite", "alterColumn");
      expect(result.supported).toBe(false);
      if (!result.supported) {
        expect(result.reason).toContain("ALTER COLUMN");
        expect(result.requiresRebuild).toBe(true);
      }
    });

    it("addForeignKey is not supported and requires rebuild", () => {
      const result = checkOperationSupported("sqlite", "addForeignKey");
      expect(result.supported).toBe(false);
      if (!result.supported) {
        expect(result.requiresRebuild).toBe(true);
      }
    });

    it("addConstraint is not supported", () => {
      const result = checkOperationSupported("sqlite", "addConstraint");
      expect(result.supported).toBe(false);
      if (!result.supported) {
        expect(result.reason).toContain("PRIMARY KEY or UNIQUE");
      }
    });

    it("dropColumn is supported", () => {
      const result = checkOperationSupported("sqlite", "dropColumn");
      expect(result.supported).toBe(true);
    });

    it("renameTable is supported", () => {
      const result = checkOperationSupported("sqlite", "renameTable");
      expect(result.supported).toBe(true);
    });

    it("unknown operations default to supported", () => {
      const result = checkOperationSupported("postgres", "someUnknownOp");
      expect(result.supported).toBe(true);
    });
  });
});

describe("buildSqliteTableRebuild", () => {
  const sqlite = getSqlDialect("sqlite");

  const baseInput: TableRebuildInput = {
    schema: "main",
    table: "users",
    currentColumns: [
      { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
      { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
      { name: "email", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
    ],
    newColumns: [
      { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
      { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
      { name: "email", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
      { name: "age", dataType: "INTEGER", nullable: true, defaultValue: null, isPk: false },
    ],
    indexes: [{ name: "idx_email", columns: ["email"], unique: true }],
  };

  it("generates correct sequence of SQL statements", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    expect(plan.statements[0]).toBe("BEGIN TRANSACTION");
    expect(plan.statements[plan.statements.length - 1]).toBe("COMMIT");
    expect(plan.statements[1]).toContain("CREATE TABLE");
    expect(plan.statements[2]).toContain("INSERT INTO");
    expect(plan.statements[3]).toContain("DROP TABLE");
    expect(plan.statements[4]).toContain("RENAME TO");
  });

  it("creates temp table with _rebuild prefix", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    expect(plan.statements[1]).toContain('"_rebuild_users"');
  });

  it("includes new column in CREATE TABLE", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    expect(plan.statements[1]).toContain('"age" INTEGER');
  });

  it("copies data for common columns only", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    const copySql = plan.statements[2];
    expect(copySql).toContain('"id"');
    expect(copySql).toContain('"name"');
    expect(copySql).toContain('"email"');
  });

  it("recreates indexes after rebuild", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    const indexSql = plan.statements.find((s) => s.includes("CREATE UNIQUE INDEX"));
    expect(indexSql).toBeDefined();
    expect(indexSql).toContain('"idx_email"');
    expect(indexSql).toContain('"email"');
  });

  it("includes description for user display", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    expect(plan.description).toContain("users");
    expect(plan.description).toContain("rebuild");
  });

  it("handles column removal (only copies surviving columns)", () => {
    const input: TableRebuildInput = {
      ...baseInput,
      newColumns: [
        { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
        { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
        // email removed
      ],
    };
    const plan = buildSqliteTableRebuild(input, sqlite);
    const copySql = plan.statements[2];
    expect(copySql).toContain('"id"');
    expect(copySql).toContain('"name"');
    // The copy should not reference "email" since it's been removed
  });

  it("includes PK constraint in new table", () => {
    const plan = buildSqliteTableRebuild(baseInput, sqlite);
    expect(plan.statements[1]).toContain("PRIMARY KEY");
  });

  it("handles no indexes gracefully", () => {
    const input = { ...baseInput, indexes: [] };
    const plan = buildSqliteTableRebuild(input, sqlite);
    // Should have: BEGIN, CREATE, COPY, DROP, RENAME, COMMIT (no index SQL)
    expect(plan.statements).toHaveLength(6);
  });

  it("handles undefined indexes gracefully", () => {
    const input: TableRebuildInput = { ...baseInput, indexes: undefined };
    const plan = buildSqliteTableRebuild(input, sqlite);
    expect(plan.statements).toHaveLength(6);
  });
});
