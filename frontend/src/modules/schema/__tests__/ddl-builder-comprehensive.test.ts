import { describe, expect, it } from "vitest";

import { getSqlDialect } from "@/modules/query/sql/dialect";
import {
  buildAddColumn,
  buildCreateIndex,
  buildCreateTable,
  buildCreateTrigger,
  buildCreateView,
  buildDropColumn,
  buildDropIndex,
  buildDropTable,
  buildDropTrigger,
  buildDropView,
  buildRenameTable,
  buildSetTriggerEnabled,
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
    const col: ColumnDef = {
      name: "age",
      dataType: "INTEGER",
      nullable: true,
      defaultValue: "",
      isPk: false,
    };
    const sql = buildAddColumn("public", "users", col, pg);
    expect(sql).toBe('ALTER TABLE "public"."users" ADD COLUMN "age" INTEGER;');
  });

  it("includes NOT NULL when column is not nullable", () => {
    const col: ColumnDef = {
      name: "age",
      dataType: "INTEGER",
      nullable: false,
      defaultValue: "",
      isPk: false,
    };
    const sql = buildAddColumn("public", "users", col, pg);
    expect(sql).toContain("NOT NULL");
  });

  it("includes DEFAULT when present", () => {
    const col: ColumnDef = {
      name: "status",
      dataType: "TEXT",
      nullable: false,
      defaultValue: "'active'",
      isPk: false,
    };
    const sql = buildAddColumn("public", "users", col, pg);
    expect(sql).toContain("DEFAULT 'active'");
    expect(sql).toContain("NOT NULL");
  });

  it("works with sqlite dialect", () => {
    const col: ColumnDef = {
      name: "email",
      dataType: "TEXT",
      nullable: true,
      defaultValue: "",
      isPk: false,
    };
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
    const sql = buildCreateView(
      "public",
      "active_users",
      "SELECT * FROM users WHERE active = true",
      pg,
    );
    expect(sql).toBe(
      'CREATE VIEW "public"."active_users" AS\nSELECT * FROM users WHERE active = true;',
    );
  });

  it("works with sqlite dialect", () => {
    const sql = buildCreateView(
      "main",
      "active_users",
      "SELECT * FROM users WHERE active = 1",
      sqlite,
    );
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
    const sql = buildCreateIndex(
      "public",
      "orders",
      "idx_orders_user_status",
      ["user_id", "status"],
      false,
      pg,
    );
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
    const col: ColumnDef = {
      name: "age",
      dataType: "INT",
      nullable: true,
      defaultValue: "",
      isPk: false,
    };
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
    const sql = generateDdlPreview(
      "renameTable",
      "public",
      "users",
      [],
      { newName: "accounts" },
      pg,
    );
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
      "createIndex",
      "public",
      "users",
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

  it("uses default index name when indexName is not provided", () => {
    const sql = generateDdlPreview(
      "createIndex",
      "public",
      "users",
      [],
      { indexColumns: "email", unique: "false" },
      pg,
    );
    expect(sql).toContain('"idx_new"');
    expect(sql).toContain("CREATE INDEX");
  });

  it("dispatches dropIndex from extra.indexName", () => {
    const sql = generateDdlPreview(
      "dropIndex",
      "public",
      "users",
      [],
      { indexName: "idx_old" },
      pg,
    );
    expect(sql).toContain("DROP INDEX");
    expect(sql).toContain('"idx_old"');
  });

  it("returns empty for dropIndex without indexName", () => {
    const sql = generateDdlPreview("dropIndex", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });

  it("returns empty for unknown operation", () => {
    const sql = generateDdlPreview(
      "unknownOp" as unknown as "create_table",
      "public",
      "users",
      [],
      {},
      pg,
    );
    expect(sql).toBe("");
  });
});

describe("buildCreateTrigger", () => {
  it("generates CREATE TRIGGER with quoted identifiers", () => {
    const sql = buildCreateTrigger(
      "public",
      "users",
      "tr_audit",
      "BEFORE",
      "INSERT",
      "BEGIN\n  NEW.updated_at = now();\nEND;",
    );
    expect(sql).toContain('CREATE TRIGGER "tr_audit"');
    expect(sql).toContain("BEFORE INSERT");
    expect(sql).toContain('"public"."users"');
    expect(sql).toContain("BEGIN");
  });

  it("escapes double quotes in trigger name", () => {
    const sql = buildCreateTrigger("public", "users", 'tr"name', "AFTER", "UPDATE", "BEGIN END;");
    expect(sql).toContain('"tr""name"');
  });

  it("escapes double quotes in schema and table", () => {
    const sql = buildCreateTrigger(
      'company"data',
      'user"events',
      "tr_log",
      "BEFORE",
      "DELETE",
      "BEGIN END;",
    );
    expect(sql).toContain('"company""data"."user""events"');
  });
});

describe("buildDropTrigger", () => {
  it("generates DROP TRIGGER with qualified table", () => {
    const sql = buildDropTrigger("public", "users", "tr_audit");
    expect(sql).toContain('DROP TRIGGER "tr_audit"');
    expect(sql).toContain('ON "public"."users"');
  });

  it("escapes double quotes in all identifiers", () => {
    const sql = buildDropTrigger('my"schema', 'my"table', 'tr"name');
    expect(sql).toContain('"tr""name"');
    expect(sql).toContain('"my""schema"."my""table"');
  });
});

describe("buildSetTriggerEnabled", () => {
  it("generates ENABLE TRIGGER for PostgreSQL", () => {
    const sql = buildSetTriggerEnabled("public", "users", "tr_audit", true, pg);
    expect(sql).toContain('ALTER TABLE "public"."users" ENABLE TRIGGER "tr_audit"');
  });

  it("generates DISABLE TRIGGER for PostgreSQL", () => {
    const sql = buildSetTriggerEnabled("public", "users", "tr_audit", false, pg);
    expect(sql).toContain('ALTER TABLE "public"."users" DISABLE TRIGGER "tr_audit"');
  });

  it("escapes identifiers with special characters", () => {
    const sql = buildSetTriggerEnabled('my"schema', 'my"table', 'tr"name', true, pg);
    expect(sql).toContain('"tr""name"');
    expect(sql).toContain('"my""schema"."my""table"');
  });
});

describe("generateDdlPreview trigger operations", () => {
  it("generates enable trigger preview", () => {
    const sql = generateDdlPreview(
      "enableTrigger",
      "public",
      "users",
      [],
      { triggerName: "tr_audit" },
      pg,
    );
    expect(sql).toContain("ENABLE TRIGGER");
    expect(sql).toContain('"tr_audit"');
  });

  it("generates disable trigger preview", () => {
    const sql = generateDdlPreview(
      "disableTrigger",
      "public",
      "users",
      [],
      { triggerName: "tr_audit" },
      pg,
    );
    expect(sql).toContain("DISABLE TRIGGER");
    expect(sql).toContain('"tr_audit"');
  });

  it("returns empty string when triggerName is missing", () => {
    const sql = generateDdlPreview("enableTrigger", "public", "users", [], {}, pg);
    expect(sql).toBe("");
  });
});
