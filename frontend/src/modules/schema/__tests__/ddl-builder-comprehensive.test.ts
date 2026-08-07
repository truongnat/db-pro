import { describe, expect, it } from "vitest";

import { getSqlDialect } from "@/modules/query/sql/dialect";
import {
  buildAddColumn,
  buildCreateIndex,
  buildCreateTable,
  buildCreateView,
  buildDropColumn,
  buildDropIndex,
  buildDropTable,
  buildDropView,
  buildRenameTable,
  generateDdlPreview,
  type ColumnDef,
} from "../services/ddl-builder";

const pg = getSqlDialect("postgres");
const sqlite = getSqlDialect("sqlite");

const USERS_COLS: ColumnDef[] = [
  { name: "id", dataType: "BIGSERIAL", nullable: false, defaultValue: "", isPk: true },
  { name: "email", dataType: "TEXT", nullable: false, defaultValue: "", isPk: false },
  { name: "name", dataType: "VARCHAR(255)", nullable: true, defaultValue: "", isPk: false },
];

const ORDERS_COLS: ColumnDef[] = [
  { name: "id", dataType: "INTEGER", nullable: false, defaultValue: "", isPk: true },
  { name: "user_id", dataType: "INTEGER", nullable: false, defaultValue: "", isPk: false },
  { name: "total", dataType: "NUMERIC(10,2)", nullable: false, defaultValue: "0.00", isPk: false },
  { name: "status", dataType: "TEXT", nullable: false, defaultValue: "'pending'", isPk: false },
];

describe("DDL builder — buildCreateTable", () => {
  it("generates CREATE TABLE with PK constraint for postgres", () => {
    const sql = buildCreateTable("public", "users", USERS_COLS, pg);
    expect(sql).toContain('CREATE TABLE "public"."users"');
    expect(sql).toContain('"id" BIGSERIAL NOT NULL');
    expect(sql).toContain('"email" TEXT NOT NULL');
    expect(sql).toContain('"name" VARCHAR(255)');
    expect(sql).not.toContain('"name" VARCHAR(255) NOT NULL');
    expect(sql).toContain('PRIMARY KEY ("id")');
  });

  it("generates CREATE TABLE with composite PK", () => {
    const cols: ColumnDef[] = [
      { name: "order_id", dataType: "INT", nullable: false, defaultValue: "", isPk: true },
      { name: "product_id", dataType: "INT", nullable: false, defaultValue: "", isPk: true },
      { name: "qty", dataType: "INT", nullable: false, defaultValue: "1", isPk: false },
    ];
    const sql = buildCreateTable("public", "order_items", cols, pg);
    expect(sql).toContain('PRIMARY KEY ("order_id", "product_id")');
  });

  it("generates CREATE TABLE for sqlite dialect", () => {
    const cols: ColumnDef[] = [
      { name: "id", dataType: "INTEGER", nullable: false, defaultValue: "", isPk: true },
      { name: "label", dataType: "TEXT", nullable: true, defaultValue: "", isPk: false },
    ];
    const sql = buildCreateTable("main", "items", cols, sqlite);
    expect(sql).toContain('CREATE TABLE "main"."items"');
    expect(sql).toContain('PRIMARY KEY ("id")');
  });

  it("includes DEFAULT value when present", () => {
    const sql = buildCreateTable("public", "orders", ORDERS_COLS, pg);
    expect(sql).toContain("DEFAULT 0.00");
    expect(sql).toContain("DEFAULT 'pending'");
  });

  it("handles table with no PK columns", () => {
    const cols: ColumnDef[] = [
      { name: "log", dataType: "TEXT", nullable: true, defaultValue: "", isPk: false },
    ];
    const sql = buildCreateTable("public", "logs", cols, pg);
    expect(sql).not.toContain("PRIMARY KEY");
  });
});

describe("DDL builder — buildAddColumn", () => {
  it("generates ALTER TABLE ADD COLUMN for postgres", () => {
    const col: ColumnDef = { name: "age", dataType: "INTEGER", nullable: true, defaultValue: "", isPk: false };
    const sql = buildAddColumn("public", "users", col, pg);
    expect(sql).toBe('ALTER TABLE "public"."users" ADD COLUMN "age" INTEGER;');
  });

  it("includes NOT NULL when column is not nullable", () => {
    const col: ColumnDef = { name: "age", dataType: "INTEGER", nullable: false, defaultValue: "", isPk: false };
    const sql = buildAddColumn("public", "users", col, pg);
    expect(sql).toContain("NOT NULL");
  });

  it("includes DEFAULT when present", () => {
    const col: ColumnDef = { name: "status", dataType: "TEXT", nullable: false, defaultValue: "'active'", isPk: false };
    const sql = buildAddColumn("public", "users", col, pg);
    expect(sql).toContain("DEFAULT 'active'");
    expect(sql).toContain("NOT NULL");
  });

  it("works with sqlite dialect", () => {
    const col: ColumnDef = { name: "email", dataType: "TEXT", nullable: true, defaultValue: "", isPk: false };
    const sql = buildAddColumn("main", "users", col, sqlite);
    expect(sql).toBe('ALTER TABLE "main"."users" ADD COLUMN "email" TEXT;');
  });
});

describe("DDL builder — buildDropColumn", () => {
  it("generates ALTER TABLE DROP COLUMN", () => {
    const sql = buildDropColumn("public", "users", "age", pg);
    expect(sql).toBe('ALTER TABLE "public"."users" DROP COLUMN "age";');
  });

  it("works with sqlite dialect", () => {
    const sql = buildDropColumn("main", "users", "age", sqlite);
    expect(sql).toBe('ALTER TABLE "main"."users" DROP COLUMN "age";');
  });
});

describe("DDL builder — buildRenameTable", () => {
  it("generates ALTER TABLE RENAME TO", () => {
    const sql = buildRenameTable("public", "users", "accounts", pg);
    expect(sql).toBe('ALTER TABLE "public"."users" RENAME TO "accounts";');
  });

  it("works with sqlite dialect", () => {
    const sql = buildRenameTable("main", "users", "accounts", sqlite);
    expect(sql).toBe('ALTER TABLE "main"."users" RENAME TO "accounts";');
  });
});

describe("DDL builder — buildDropTable", () => {
  it("generates DROP TABLE", () => {
    const sql = buildDropTable("public", "users", pg);
    expect(sql).toBe('DROP TABLE "public"."users";');
  });

  it("works with sqlite dialect", () => {
    const sql = buildDropTable("main", "users", sqlite);
    expect(sql).toBe('DROP TABLE "main"."users";');
  });
});

describe("DDL builder — buildCreateView", () => {
  it("generates CREATE VIEW AS", () => {
    const sql = buildCreateView("public", "active_users", "SELECT * FROM users WHERE active = true", pg);
    expect(sql).toBe('CREATE VIEW "public"."active_users" AS\nSELECT * FROM users WHERE active = true;');
  });

  it("works with sqlite dialect", () => {
    const sql = buildCreateView("main", "active_users", "SELECT * FROM users WHERE active = 1", sqlite);
    expect(sql).toContain('CREATE VIEW "main"."active_users"');
  });
});

describe("DDL builder — buildDropView", () => {
  it("generates DROP VIEW", () => {
    const sql = buildDropView("public", "active_users", pg);
    expect(sql).toBe('DROP VIEW "public"."active_users";');
  });
});

describe("DDL builder — buildCreateIndex", () => {
  it("generates CREATE INDEX with single column", () => {
    const sql = buildCreateIndex("public", "users", "idx_users_email", ["email"], false, pg);
    expect(sql).toBe('CREATE INDEX "idx_users_email" ON "public"."users" ("email");');
  });

  it("generates CREATE UNIQUE INDEX", () => {
    const sql = buildCreateIndex("public", "users", "idx_users_email", ["email"], true, pg);
    expect(sql).toBe('CREATE UNIQUE INDEX "idx_users_email" ON "public"."users" ("email");');
  });

  it("generates index with multiple columns", () => {
    const sql = buildCreateIndex("public", "orders", "idx_orders_user_status", ["user_id", "status"], false, pg);
    expect(sql).toContain('("user_id", "status")');
  });

  it("works with sqlite dialect", () => {
    const sql = buildCreateIndex("main", "users", "idx_email", ["email"], true, sqlite);
    expect(sql).toContain('CREATE UNIQUE INDEX "idx_email" ON "main"."users"');
  });
});

describe("DDL builder — buildDropIndex", () => {
  it("generates DROP INDEX", () => {
    const sql = buildDropIndex("public", "idx_users_email", pg);
    expect(sql).toBe('DROP INDEX "public"."idx_users_email";');
  });
});

describe("DDL builder — generateDdlPreview", () => {
  it("dispatches createTable", () => {
    const sql = generateDdlPreview("createTable", "public", "users", USERS_COLS, {}, pg);
    expect(sql).toContain("CREATE TABLE");
  });

  it("dispatches addColumn with first column", () => {
    const col: ColumnDef = { name: "age", dataType: "INT", nullable: true, defaultValue: "", isPk: false };
    const sql = generateDdlPreview("addColumn", "public", "users", [col], {}, pg);
    expect(sql).toContain("ADD COLUMN");
    expect(sql).toContain('"age"');
  });

  it("returns empty for addColumn with no columns", () => {
    const sql = generateDdlPreview("addColumn", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });

  it("dispatches dropColumn from extra.columnName", () => {
    const sql = generateDdlPreview("dropColumn", "public", "users", [], { columnName: "age" }, pg);
    expect(sql).toContain("DROP COLUMN");
    expect(sql).toContain('"age"');
  });

  it("returns empty for dropColumn without columnName", () => {
    const sql = generateDdlPreview("dropColumn", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });

  it("dispatches renameTable from extra.newName", () => {
    const sql = generateDdlPreview("renameTable", "public", "users", [], { newName: "accounts" }, pg);
    expect(sql).toContain("RENAME TO");
    expect(sql).toContain('"accounts"');
  });

  it("returns empty for renameTable without newName", () => {
    const sql = generateDdlPreview("renameTable", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });

  it("dispatches dropTable", () => {
    const sql = generateDdlPreview("dropTable", "public", "users", [], {}, pg);
    expect(sql).toContain("DROP TABLE");
  });

  it("dispatches createView from extra.selectSql", () => {
    const sql = generateDdlPreview("createView", "public", "v", [], { selectSql: "SELECT 1" }, pg);
    expect(sql).toContain("CREATE VIEW");
    expect(sql).toContain("SELECT 1");
  });

  it("returns empty for createView without selectSql", () => {
    const sql = generateDdlPreview("createView", "public", "v", [], {}, pg);
    expect(sql).toBe("");
  });

  it("dispatches dropView", () => {
    const sql = generateDdlPreview("dropView", "public", "v", [], {}, pg);
    expect(sql).toContain("DROP VIEW");
  });

  it("dispatches createIndex from extra.indexColumns", () => {
    const sql = generateDdlPreview(
      "createIndex", "public", "users",
      [],
      { indexName: "idx_email", indexColumns: "email", unique: "true" },
      pg,
    );
    expect(sql).toContain("CREATE UNIQUE INDEX");
    expect(sql).toContain('"email"');
  });

  it("returns empty for createIndex without indexColumns", () => {
    const sql = generateDdlPreview("createIndex", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });

  it("dispatches dropIndex from extra.indexName", () => {
    const sql = generateDdlPreview("dropIndex", "public", "users", [], { indexName: "idx_old" }, pg);
    expect(sql).toContain("DROP INDEX");
    expect(sql).toContain('"idx_old"');
  });

  it("returns empty for dropIndex without indexName", () => {
    const sql = generateDdlPreview("dropIndex", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });

  it("returns empty for unknown operation", () => {
    const sql = generateDdlPreview("unknownOp" as any, "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });
});
