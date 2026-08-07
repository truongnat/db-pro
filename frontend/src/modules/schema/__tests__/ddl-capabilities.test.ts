import { describe, expect, it } from "vitest";

import { getSqlDialect } from "@/modules/query/sql/dialect";
import {
  getDdlCapabilities,
  checkOperationSupported,
  buildSqliteTableRebuild,
  type TableRebuildInput,
} from "../services/ddl-capabilities";

describe("DDL capabilities", () => {
  describe("getDdlCapabilities", () => {
    it("postgres supports all DDL operations", () => {
      const caps = getDdlCapabilities("postgres");
      expect(caps.supportsDropColumn).toBe(true);
      expect(caps.supportsAlterColumn).toBe(true);
      expect(caps.supportsRenameTable).toBe(true);
      expect(caps.supportsRenameColumn).toBe(true);
      expect(caps.supportsAddColumnWithConstraint).toBe(true);
      expect(caps.supportsTransactionalDdl).toBe(true);
      expect(caps.supportsIdentity).toBe(true);
      expect(caps.supportsTriggers).toBe(true);
      expect(caps.requiresTableRebuild).toEqual([]);
    });

    it("sqlite has limited DDL support", () => {
      const caps = getDdlCapabilities("sqlite");
      expect(caps.supportsDropColumn).toBe(true);
      expect(caps.supportsAlterColumn).toBe(false);
      expect(caps.supportsRenameTable).toBe(true);
      expect(caps.supportsRenameColumn).toBe(true);
      expect(caps.supportsAddColumnWithConstraint).toBe(false);
      expect(caps.supportsTransactionalDdl).toBe(false);
      expect(caps.supportsIdentity).toBe(false);
      expect(caps.supportsTriggers).toBe(true);
    });

    it("sqlite requires table rebuild for alterColumn and addForeignKey", () => {
      const caps = getDdlCapabilities("sqlite");
      expect(caps.requiresTableRebuild).toContain("alterColumn");
      expect(caps.requiresTableRebuild).toContain("addForeignKey");
    });
  });

  describe("checkOperationSupported", () => {
    it("postgres supports dropColumn", () => {
      expect(checkOperationSupported("postgres", "dropColumn")).toEqual({ supported: true });
    });

    it("sqlite alterColumn is not supported and requires rebuild", () => {
      const result = checkOperationSupported("sqlite", "alterColumn");
      expect(result.supported).toBe(false);
      if (!result.supported) {
        expect(result.requiresRebuild).toBe(true);
        expect(result.reason).toContain("rebuilding the table");
      }
    });

    it("sqlite addForeignKey requires rebuild", () => {
      const result = checkOperationSupported("sqlite", "addForeignKey");
      expect(result.supported).toBe(false);
      if (!result.supported) {
        expect(result.requiresRebuild).toBe(true);
      }
    });

    it("sqlite addConstraint is not supported", () => {
      const result = checkOperationSupported("sqlite", "addConstraint");
      expect(result.supported).toBe(false);
    });

    it("unknown operations default to supported", () => {
      expect(checkOperationSupported("postgres", "createTable")).toEqual({ supported: true });
      expect(checkOperationSupported("sqlite", "createTable")).toEqual({ supported: true });
    });
  });

  describe("buildSqliteTableRebuild", () => {
    const dialect = getSqlDialect("sqlite");

    it("generates rebuild plan with correct steps", () => {
      const input: TableRebuildInput = {
        schema: "main",
        table: "users",
        currentColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
          { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
          { name: "email", dataType: "TEXT", nullable: true, defaultValue: null, isPk: false },
        ],
        newColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
          { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
          { name: "email", dataType: "TEXT", nullable: false, defaultValue: "''", isPk: false },
        ],
      };

      const plan = buildSqliteTableRebuild(input, dialect);

      expect(plan.statements[0]).toBe("BEGIN TRANSACTION");
      expect(plan.statements[1]).toContain("CREATE TABLE");
      expect(plan.statements[1]).toContain("_rebuild_users");
      expect(plan.statements[2]).toContain("INSERT INTO");
      expect(plan.statements[2]).toContain("SELECT");
      expect(plan.statements[3]).toContain("DROP TABLE");
      expect(plan.statements[4]).toContain("RENAME TO");
      expect(plan.statements[plan.statements.length - 1]).toBe("COMMIT");
    });

    it("copies only common columns", () => {
      const input: TableRebuildInput = {
        schema: "main",
        table: "users",
        currentColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
          { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
        ],
        newColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
          { name: "name", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
          { name: "email", dataType: "TEXT", nullable: true, defaultValue: null, isPk: false },
        ],
      };

      const plan = buildSqliteTableRebuild(input, dialect);
      const copyStatement = plan.statements[2];

      // Should copy id and name (common columns), not email (new)
      expect(copyStatement).toContain('"id"');
      expect(copyStatement).toContain('"name"');
      expect(copyStatement).not.toContain('"email"');
    });

    it("recreates indexes after rebuild", () => {
      const input: TableRebuildInput = {
        schema: "main",
        table: "users",
        currentColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
          { name: "email", dataType: "TEXT", nullable: false, defaultValue: null, isPk: false },
        ],
        newColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
          { name: "email", dataType: "VARCHAR(255)", nullable: false, defaultValue: null, isPk: false },
        ],
        indexes: [
          { name: "idx_email", columns: ["email"], unique: true },
        ],
      };

      const plan = buildSqliteTableRebuild(input, dialect);
      const indexSql = plan.statements.find((s) => s.includes("INDEX"));
      expect(indexSql).toBeDefined();
      expect(indexSql).toContain("UNIQUE");
      expect(indexSql).toContain("idx_email");
    });

    it("includes description", () => {
      const input: TableRebuildInput = {
        schema: "main",
        table: "users",
        currentColumns: [
          { name: "id", dataType: "INTEGER", nullable: false, defaultValue: null, isPk: true },
        ],
        newColumns: [
          { name: "id", dataType: "BIGINT", nullable: false, defaultValue: null, isPk: true },
        ],
      };

      const plan = buildSqliteTableRebuild(input, dialect);
      expect(plan.description).toContain("rebuilding");
      expect(plan.description).toContain("users");
    });
  });
});
