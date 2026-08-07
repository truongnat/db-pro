import { describe, expect, it } from "vitest";

import type { ConnectionCatalog } from "../stores/schema-catalog.store";
import type { SchemaColumnDto } from "@/modules/schema/types/schema.types";
import { resolveSymbolAtOffset, getWordAtOffset } from "../services/sql-symbol-resolver";

/* ------------------------------------------------------------------ */
/*  Test fixtures                                                      */
/* ------------------------------------------------------------------ */

function col(
  name: string,
  dataType: string,
  opts: { nullable?: boolean; isPrimaryKey?: boolean; schema?: string; tableName?: string } = {},
): SchemaColumnDto {
  return {
    name,
    dataType,
    nullable: opts.nullable ?? true,
    defaultValue: null,
    isPrimaryKey: opts.isPrimaryKey ?? false,
    tableName: opts.tableName ?? "users",
    schema: opts.schema ?? "public",
  };
}

const usersColumns: SchemaColumnDto[] = [
  col("id", "integer", { isPrimaryKey: true, nullable: false }),
  col("name", "varchar(255)", { nullable: false }),
  col("email", "varchar(255)", { nullable: false }),
  col("created_at", "timestamp", { nullable: true }),
];

const ordersColumns: SchemaColumnDto[] = [
  col("id", "integer", { isPrimaryKey: true, nullable: false, tableName: "orders" }),
  col("user_id", "integer", { nullable: false, tableName: "orders" }),
  col("total", "numeric", { nullable: true, tableName: "orders" }),
  col("status", "varchar(50)", { nullable: true, tableName: "orders" }),
];

function createCatalog(): ConnectionCatalog {
  const columnsByTable = new Map<string, SchemaColumnDto[]>();
  columnsByTable.set("public.users", usersColumns);
  columnsByTable.set("public.orders", ordersColumns);

  return {
    schemas: [{ name: "public" }, { name: "inventory" }],
    objects: [
      { name: "users", schema: "public", rowCount: 100, kind: "table" },
      { name: "orders", schema: "public", rowCount: 500, kind: "table" },
      { name: "products", schema: "inventory", rowCount: 50, kind: "table" },
    ],
    columnsByTable,
    columnsLoaded: new Set(["public.users", "public.orders"]),
    columnsLoading: new Map(),
  };
}

/* ------------------------------------------------------------------ */
/*  getWordAtOffset                                                    */
/* ------------------------------------------------------------------ */

describe("getWordAtOffset", () => {
  it("returns word at a normal position", () => {
    const result = getWordAtOffset("SELECT name FROM", 8);
    expect(result).toEqual({ word: "name", start: 7, end: 11 });
  });

  it("returns word at start of text", () => {
    const result = getWordAtOffset("SELECT", 0);
    expect(result).toEqual({ word: "SELECT", start: 0, end: 6 });
  });

  it("returns word at end of text (last char)", () => {
    const result = getWordAtOffset("SELECT", 5);
    expect(result).toEqual({ word: "SELECT", start: 0, end: 6 });
  });

  it("returns null past end of text", () => {
    const result = getWordAtOffset("SELECT", 6);
    expect(result).toBeNull();
  });

  it("returns null for whitespace offset", () => {
    const result = getWordAtOffset("SELECT name", 6);
    expect(result).toBeNull();
  });

  it("returns null for empty text", () => {
    const result = getWordAtOffset("", 0);
    expect(result).toBeNull();
  });

  it("handles word with underscores", () => {
    const result = getWordAtOffset("created_at", 5);
    expect(result).toEqual({ word: "created_at", start: 0, end: 10 });
  });
});

/* ------------------------------------------------------------------ */
/*  resolveSymbolAtOffset — table resolution                           */
/* ------------------------------------------------------------------ */

describe("resolveSymbolAtOffset — table", () => {
  const catalog = createCatalog();

  it("resolves an unqualified table name", () => {
    // SELECT * FROM users
    //             ^^^^^
    const sql = "SELECT * FROM users";
    const result = resolveSymbolAtOffset(sql, 14, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("table");
    expect(result!.schema).toBe("public");
    expect(result!.table).toBe("users");
  });

  it("resolves a schema-qualified table name (hover on table part)", () => {
    // SELECT * FROM inventory.products
    //                       ^^^^^^^^
    const sql = "SELECT * FROM inventory.products";
    const result = resolveSymbolAtOffset(sql, 25, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("table");
    expect(result!.schema).toBe("inventory");
    expect(result!.table).toBe("products");
  });

  it("resolves a schema-qualified table name (hover on schema part)", () => {
    // SELECT * FROM inventory.products
    //             ^^^^^^^^^
    const sql = "SELECT * FROM inventory.products";
    const result = resolveSymbolAtOffset(sql, 16, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("table");
    expect(result!.schema).toBe("inventory");
    expect(result!.table).toBe("products");
  });

  it("returns null for SQL keywords", () => {
    const sql = "SELECT * FROM users";
    const result = resolveSymbolAtOffset(sql, 2, catalog); // on "SELECT"
    expect(result).toBeNull();
  });
});

/* ------------------------------------------------------------------ */
/*  resolveSymbolAtOffset — qualified column                           */
/* ------------------------------------------------------------------ */

describe("resolveSymbolAtOffset — qualified column", () => {
  const catalog = createCatalog();

  it("resolves alias.column", () => {
    // SELECT u.email FROM users u
    //          ^^^^^
    const sql = "SELECT u.email FROM users u";
    const result = resolveSymbolAtOffset(sql, 10, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("column");
    expect(result!.column!.name).toBe("email");
    expect(result!.table).toBe("users");
  });

  it("resolves table.column (no alias)", () => {
    // SELECT users.email FROM users
    //               ^^^^^
    const sql = "SELECT users.email FROM users";
    const result = resolveSymbolAtOffset(sql, 13, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("column");
    expect(result!.column!.name).toBe("email");
    expect(result!.table).toBe("users");
  });

  it("returns null for unknown column on known alias", () => {
    // SELECT u.unknown FROM users u
    const sql = "SELECT u.unknown FROM users u";
    const result = resolveSymbolAtOffset(sql, 11, catalog);
    expect(result).toBeNull();
  });

  it("returns null for unknown qualifier", () => {
    // SELECT x.email FROM users u
    const sql = "SELECT x.email FROM users u";
    const result = resolveSymbolAtOffset(sql, 10, catalog);
    expect(result).toBeNull();
  });
});

/* ------------------------------------------------------------------ */
/*  resolveSymbolAtOffset — unqualified column                         */
/* ------------------------------------------------------------------ */

describe("resolveSymbolAtOffset — unqualified column", () => {
  const catalog = createCatalog();

  it("resolves an unqualified column from FROM table", () => {
    // SELECT email FROM users
    //        ^^^^^
    const sql = "SELECT email FROM users";
    const result = resolveSymbolAtOffset(sql, 8, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("column");
    expect(result!.column!.name).toBe("email");
    expect(result!.table).toBe("users");
  });

  it("resolves column from JOIN table", () => {
    const sql = "SELECT user_id FROM users JOIN orders ON users.id = orders.user_id";
    const result = resolveSymbolAtOffset(sql, 9, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("column");
    expect(result!.column!.name).toBe("user_id");
    expect(result!.table).toBe("orders");
  });

  it("returns null for unknown unqualified column", () => {
    // SELECT unknown_col FROM users
    const sql = "SELECT unknown_col FROM users";
    const result = resolveSymbolAtOffset(sql, 10, catalog);
    expect(result).toBeNull();
  });
});

/* ------------------------------------------------------------------ */
/*  resolveSymbolAtOffset — schema                                     */
/* ------------------------------------------------------------------ */

describe("resolveSymbolAtOffset — schema", () => {
  const catalog = createCatalog();

  it("resolves a schema name when not part of schema.table", () => {
    // SET search_path TO public
    //                   ^^^^^^
    const sql = "SET search_path TO public";
    const result = resolveSymbolAtOffset(sql, 19, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("schema");
    expect(result!.schema).toBe("public");
  });
});

/* ------------------------------------------------------------------ */
/*  resolveSymbolAtOffset — multi-statement                            */
/* ------------------------------------------------------------------ */

describe("resolveSymbolAtOffset — multi-statement", () => {
  const catalog = createCatalog();

  it("resolves within the current statement only", () => {
    // SELECT 1; SELECT email FROM users
    //                      ^^^^^
    const sql = "SELECT 1; SELECT email FROM users";
    const result = resolveSymbolAtOffset(sql, 17, catalog);
    expect(result).not.toBeNull();
    expect(result!.kind).toBe("column");
    expect(result!.column!.name).toBe("email");
  });
});
