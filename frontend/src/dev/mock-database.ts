/**
 * In-browser fixture database used by the dev-only browser backend
 * (`src/dev/browser-backend.ts`).
 *
 * It mirrors `fixtures/postgres/schema.sql` so every type-specific rendering
 * path in the UI (NULL / bool / int64 / float64 / text / uuid / datetime /
 * jsonb) is exercised, and ships a small SQL subset so the editor, result
 * grid, history and explain surfaces are interactive without the Rust core.
 *
 * This module is never imported from production code paths — see
 * `browser-backend.ts` for the DEV guard.
 */

import type { CellValue, ColumnMeta, QueryResult, Row } from "@/modules/query/types/query.types";
import type {
  IntrospectResult,
  PrimaryKeyDto,
  SchemaColumnDto,
  SchemaDto,
  SchemaForeignKeyDto,
  SchemaIndexDto,
  TableDto,
  TableInfo,
  TriggerDto,
  ViewDto,
} from "@/modules/schema/types/schema.types";

/* ------------------------------------------------------------------ */
/*  Shape                                                              */
/* ------------------------------------------------------------------ */

type Raw = string | number | boolean | null | Record<string, unknown>;

export interface MockColumnDef {
  name: string;
  /** PostgreSQL type name, as the real introspector reports it. */
  dataType: string;
  nullable: boolean;
  defaultValue?: string | null;
  primaryKey?: boolean;
}

export interface MockTableDef {
  schema: string;
  name: string;
  isView?: boolean;
  definition?: string;
  columns: MockColumnDef[];
  rows: Raw[][];
}

/* ------------------------------------------------------------------ */
/*  Fixture schema                                                     */
/* ------------------------------------------------------------------ */

const USERS_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: false, primaryKey: true, defaultValue: "nextval" },
  { name: "email", dataType: "text", nullable: false },
  { name: "name", dataType: "varchar", nullable: false },
  { name: "bio", dataType: "text", nullable: true },
  { name: "is_active", dataType: "bool", nullable: false, defaultValue: "true" },
  { name: "created_at", dataType: "timestamptz", nullable: false, defaultValue: "now()" },
  { name: "updated_at", dataType: "timestamptz", nullable: false, defaultValue: "now()" },
];

const CATEGORIES_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int4", nullable: false, primaryKey: true },
  { name: "name", dataType: "varchar", nullable: false },
  { name: "slug", dataType: "varchar", nullable: false },
  { name: "parent_id", dataType: "int4", nullable: true },
];

const PRODUCTS_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: false, primaryKey: true },
  { name: "name", dataType: "varchar", nullable: false },
  { name: "description", dataType: "text", nullable: true },
  { name: "price", dataType: "numeric", nullable: false, defaultValue: "0.00" },
  { name: "category_id", dataType: "int4", nullable: true },
  { name: "in_stock", dataType: "bool", nullable: false, defaultValue: "true" },
  { name: "created_at", dataType: "timestamptz", nullable: false, defaultValue: "now()" },
];

const ORDERS_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: false, primaryKey: true },
  { name: "user_id", dataType: "int8", nullable: false },
  { name: "status", dataType: "varchar", nullable: false, defaultValue: "'pending'" },
  { name: "total", dataType: "numeric", nullable: false, defaultValue: "0.00" },
  { name: "created_at", dataType: "timestamptz", nullable: false, defaultValue: "now()" },
  { name: "shipped_at", dataType: "timestamptz", nullable: true },
];

const ORDER_ITEMS_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: false, primaryKey: true },
  { name: "order_id", dataType: "int8", nullable: false },
  { name: "product_id", dataType: "int8", nullable: false },
  { name: "quantity", dataType: "int4", nullable: false, defaultValue: "1" },
  { name: "unit_price", dataType: "numeric", nullable: false },
];

const TAGS_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int4", nullable: false, primaryKey: true },
  { name: "name", dataType: "varchar", nullable: false },
];

const PRODUCT_TAGS_COLUMNS: MockColumnDef[] = [
  { name: "product_id", dataType: "int8", nullable: false, primaryKey: true },
  { name: "tag_id", dataType: "int4", nullable: false, primaryKey: true },
];

const AUDIT_LOG_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: false, primaryKey: true },
  { name: "entity_type", dataType: "varchar", nullable: false },
  { name: "entity_id", dataType: "int8", nullable: false },
  { name: "action", dataType: "varchar", nullable: false },
  { name: "old_value", dataType: "jsonb", nullable: true },
  { name: "new_value", dataType: "jsonb", nullable: true },
  { name: "changed_by", dataType: "int8", nullable: true },
  { name: "created_at", dataType: "timestamptz", nullable: false, defaultValue: "now()" },
];

const ACTIVE_USERS_SUMMARY_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: true },
  { name: "email", dataType: "text", nullable: true },
  { name: "name", dataType: "varchar", nullable: true },
  { name: "order_count", dataType: "int8", nullable: true },
  { name: "total_spent", dataType: "numeric", nullable: true },
];

const PRODUCT_CATALOG_COLUMNS: MockColumnDef[] = [
  { name: "id", dataType: "int8", nullable: true },
  { name: "product_name", dataType: "varchar", nullable: true },
  { name: "price", dataType: "numeric", nullable: true },
  { name: "in_stock", dataType: "bool", nullable: true },
  { name: "category_name", dataType: "varchar", nullable: true },
];

export const MOCK_TABLES: MockTableDef[] = [
  {
    schema: "public",
    name: "users",
    columns: USERS_COLUMNS,
    rows: [
      [
        1,
        "alice@example.com",
        "Alice Johnson",
        "Software engineer",
        true,
        "2025-11-02 09:14:22+00",
        "2026-08-01 11:02:10+00",
      ],
      [
        2,
        "bob@example.com",
        "Bob Smith",
        "Product manager",
        true,
        "2025-11-03 08:00:00+00",
        "2026-07-19 14:31:45+00",
      ],
      [
        3,
        "carol@example.com",
        "Carol Williams",
        "Data analyst",
        true,
        "2025-11-05 12:45:09+00",
        "2026-06-30 09:12:00+00",
      ],
      [
        4,
        "dave@example.com",
        "Dave Brown",
        "Designer",
        false,
        "2025-11-08 16:20:31+00",
        "2026-05-11 18:44:02+00",
      ],
      [
        5,
        "eve@example.com",
        "Eve Davis",
        "DevOps engineer",
        true,
        "2025-11-11 07:55:18+00",
        "2026-08-09 06:03:55+00",
      ],
      [
        6,
        "frank@example.com",
        "Frank Miller",
        null,
        true,
        "2025-12-01 10:05:44+00",
        "2026-08-02 22:10:09+00",
      ],
      [
        7,
        "grace@example.com",
        "Grace Lee",
        "Support lead",
        true,
        "2025-12-14 13:30:00+00",
        "2026-07-25 08:45:12+00",
      ],
      [
        8,
        "heidi@example.com",
        "Heidi Nguyen",
        "Backend engineer",
        true,
        "2026-01-06 09:00:00+00",
        "2026-08-08 15:22:37+00",
      ],
      [
        9,
        "ivan@example.com",
        "Ivan Petrov",
        "QA engineer",
        false,
        "2026-01-19 11:11:11+00",
        "2026-04-02 10:00:00+00",
      ],
      [
        10,
        "judy@example.com",
        "Judy Alvarez",
        "Security analyst",
        true,
        "2026-02-02 08:30:00+00",
        "2026-08-10 07:15:48+00",
      ],
      [
        11,
        "karl@example.com",
        "Karl Weber",
        "Data engineer",
        true,
        "2026-02-21 14:02:33+00",
        "2026-08-05 19:41:20+00",
      ],
      [
        12,
        "lena@example.com",
        "Lena Fischer",
        "Technical writer",
        null,
        "2026-03-08 09:45:00+00",
        "2026-08-10 12:00:00+00",
      ],
    ],
  },
  {
    schema: "public",
    name: "categories",
    columns: CATEGORIES_COLUMNS,
    rows: [
      [1, "Electronics", "electronics", null],
      [2, "Books", "books", null],
      [3, "Clothing", "clothing", null],
      [4, "Laptops", "laptops", 1],
      [5, "Phones", "phones", 1],
      [6, "Fiction", "fiction", 2],
      [7, "Non-Fiction", "non-fiction", 2],
    ],
  },
  {
    schema: "public",
    name: "products",
    columns: PRODUCTS_COLUMNS,
    rows: [
      [
        1,
        "MacBook Pro 14",
        "M4 Pro, 24 GB unified memory",
        2499.0,
        4,
        true,
        "2025-11-20 10:00:00+00",
      ],
      [2, "ThinkPad X1 Carbon", "14 inch, 32 GB RAM", 1899.5, 4, true, "2025-11-21 10:00:00+00"],
      [3, "iPhone 17 Pro", "256 GB, titanium", 1199.0, 5, true, "2025-12-01 10:00:00+00"],
      [4, "Pixel 10", "128 GB, obsidian", 799.99, 5, false, "2025-12-02 10:00:00+00"],
      [
        5,
        "The Pragmatic Programmer",
        "20th anniversary edition",
        44.95,
        7,
        true,
        "2026-01-04 10:00:00+00",
      ],
      [6, "Designing Data-Intensive Applications", null, 51.2, 7, true, "2026-01-05 10:00:00+00"],
      [7, "Dune", "Frank Herbert, hardcover", 28.0, 6, true, "2026-01-06 10:00:00+00"],
      [8, "Merino Wool Sweater", "Charcoal, size M", 129.0, 3, false, "2026-02-11 10:00:00+00"],
      [9, "Selvedge Denim Jeans", "Raw indigo, 32x32", 189.0, 3, true, "2026-02-12 10:00:00+00"],
      [10, "USB-C Hub 8-in-1", null, 59.0, 1, true, "2026-03-03 10:00:00+00"],
    ],
  },
  {
    schema: "public",
    name: "orders",
    columns: ORDERS_COLUMNS,
    rows: [
      [1, 1, "shipped", 2499.0, "2026-04-02 09:12:00+00", "2026-04-05 13:20:00+00"],
      [2, 1, "shipped", 73.95, "2026-04-18 15:41:00+00", "2026-04-21 08:05:00+00"],
      [3, 2, "delivered", 1199.0, "2026-05-01 11:00:00+00", "2026-05-04 16:44:00+00"],
      [4, 3, "processing", 238.0, "2026-06-11 10:22:00+00", null],
      [5, 5, "delivered", 96.15, "2026-06-14 17:03:00+00", "2026-06-17 09:31:00+00"],
      [6, 10, "pending", 189.0, "2026-07-02 08:15:00+00", null],
      [7, 8, "cancelled", 1899.5, "2026-07-09 12:44:00+00", null],
      [8, 7, "delivered", 59.0, "2026-07-21 14:29:00+00", "2026-07-24 10:02:00+00"],
      [9, 11, "processing", 799.99, "2026-08-01 09:59:00+00", null],
      [10, 6, "shipped", 129.0, "2026-08-04 18:11:00+00", "2026-08-07 11:47:00+00"],
      [11, 12, "pending", 44.95, "2026-08-08 07:40:00+00", null],
      [12, 9, "delivered", 1199.0, "2026-08-09 20:15:00+00", "2026-08-10 12:00:00+00"],
    ],
  },
  {
    schema: "public",
    name: "order_items",
    columns: ORDER_ITEMS_COLUMNS,
    rows: [
      [1, 1, 1, 1, 2499.0],
      [2, 2, 5, 1, 44.95],
      [3, 2, 7, 1, 29.0],
      [4, 3, 3, 1, 1199.0],
      [5, 4, 7, 2, 28.0],
      [6, 4, 8, 1, 129.0],
      [7, 5, 5, 1, 44.95],
      [8, 5, 6, 1, 51.2],
      [9, 6, 9, 1, 189.0],
      [10, 7, 2, 1, 1899.5],
      [11, 8, 10, 1, 59.0],
      [12, 9, 4, 1, 799.99],
      [13, 10, 8, 1, 129.0],
      [14, 11, 5, 1, 44.95],
      [15, 12, 3, 1, 1199.0],
    ],
  },
  {
    schema: "public",
    name: "tags",
    columns: TAGS_COLUMNS,
    rows: [
      [1, "featured"],
      [2, "new-arrival"],
      [3, "clearance"],
      [4, "staff-pick"],
      [5, "limited"],
      [6, "bestseller"],
      [7, "eco"],
      [8, "refurbished"],
    ],
  },
  {
    schema: "public",
    name: "product_tags",
    columns: PRODUCT_TAGS_COLUMNS,
    rows: [
      [1, 1],
      [1, 6],
      [2, 4],
      [3, 1],
      [3, 2],
      [4, 3],
      [5, 6],
      [6, 4],
      [7, 2],
      [8, 3],
      [9, 7],
      [10, 8],
      [10, 3],
    ],
  },
  {
    schema: "public",
    name: "audit_log",
    columns: AUDIT_LOG_COLUMNS,
    rows: [
      [
        1,
        "product",
        4,
        "update",
        { in_stock: true },
        { in_stock: false },
        3,
        "2026-05-12 10:00:00+00",
      ],
      [2, "product", 8, "update", { price: 149 }, { price: 129 }, 3, "2026-06-01 12:00:00+00"],
      [
        3,
        "order",
        7,
        "update",
        { status: "processing" },
        { status: "cancelled" },
        2,
        "2026-07-10 09:15:00+00",
      ],
      [4, "user", 12, "insert", null, { email: "lena@example.com" }, 1, "2026-03-08 09:45:00+00"],
      [5, "category", 7, "insert", null, { name: "Non-Fiction" }, 2, "2026-01-02 08:20:00+00"],
      [
        6,
        "product",
        10,
        "insert",
        null,
        { name: "USB-C Hub 8-in-1", price: 59 },
        8,
        "2026-03-03 10:00:00+00",
      ],
      [
        7,
        "user",
        9,
        "update",
        { is_active: true },
        { is_active: false },
        10,
        "2026-04-02 10:00:00+00",
      ],
      [
        8,
        "order",
        12,
        "update",
        { status: "shipped" },
        { status: "delivered" },
        5,
        "2026-08-10 12:00:00+00",
      ],
    ],
  },
  {
    schema: "public",
    name: "active_users_summary",
    isView: true,
    definition:
      "SELECT u.id, u.email, u.name, COUNT(o.id) AS order_count,\n       COALESCE(SUM(o.total), 0) AS total_spent\n  FROM public.users u\n  LEFT JOIN public.orders o ON o.user_id = u.id\n WHERE u.is_active = true\n GROUP BY u.id, u.email, u.name",
    columns: ACTIVE_USERS_SUMMARY_COLUMNS,
    rows: [
      [1, "alice@example.com", "Alice Johnson", 2, 2572.95],
      [2, "bob@example.com", "Bob Smith", 1, 1199.0],
      [3, "carol@example.com", "Carol Williams", 1, 238.0],
      [5, "eve@example.com", "Eve Davis", 1, 96.15],
      [6, "frank@example.com", "Frank Miller", 1, 129.0],
      [7, "grace@example.com", "Grace Lee", 1, 59.0],
      [8, "heidi@example.com", "Heidi Nguyen", 0, 0],
      [10, "judy@example.com", "Judy Alvarez", 1, 189.0],
      [11, "karl@example.com", "Karl Weber", 1, 799.99],
      [12, "lena@example.com", "Lena Fischer", 1, 44.95],
    ],
  },
  {
    schema: "public",
    name: "product_catalog",
    isView: true,
    definition:
      "SELECT p.id, p.name AS product_name, p.price, p.in_stock, c.name AS category_name\n  FROM public.products p\n  LEFT JOIN public.categories c ON c.id = p.category_id",
    columns: PRODUCT_CATALOG_COLUMNS,
    rows: [
      [1, "MacBook Pro 14", 2499.0, true, "Laptops"],
      [2, "ThinkPad X1 Carbon", 1899.5, true, "Laptops"],
      [3, "iPhone 17 Pro", 1199.0, true, "Phones"],
      [4, "Pixel 10", 799.99, false, "Phones"],
      [5, "The Pragmatic Programmer", 44.95, true, "Non-Fiction"],
      [6, "Designing Data-Intensive Applications", 51.2, true, "Non-Fiction"],
      [7, "Dune", 28.0, true, "Fiction"],
      [8, "Merino Wool Sweater", 129.0, false, "Clothing"],
      [9, "Selvedge Denim Jeans", 189.0, true, "Clothing"],
      [10, "USB-C Hub 8-in-1", 59.0, true, "Electronics"],
    ],
  },
];

const PRIMARY_KEYS: PrimaryKeyDto[] = [
  { constraintName: "users_pkey", columns: ["id"], tableName: "users", schema: "public" },
  { constraintName: "categories_pkey", columns: ["id"], tableName: "categories", schema: "public" },
  { constraintName: "products_pkey", columns: ["id"], tableName: "products", schema: "public" },
  { constraintName: "orders_pkey", columns: ["id"], tableName: "orders", schema: "public" },
  {
    constraintName: "order_items_pkey",
    columns: ["id"],
    tableName: "order_items",
    schema: "public",
  },
  { constraintName: "tags_pkey", columns: ["id"], tableName: "tags", schema: "public" },
  {
    constraintName: "product_tags_pkey",
    columns: ["product_id", "tag_id"],
    tableName: "product_tags",
    schema: "public",
  },
  { constraintName: "audit_log_pkey", columns: ["id"], tableName: "audit_log", schema: "public" },
];

const INDEXES: SchemaIndexDto[] = [
  { name: "users_pkey", columns: ["id"], unique: true, tableName: "users", schema: "public" },
  {
    name: "users_email_key",
    columns: ["email"],
    unique: true,
    tableName: "users",
    schema: "public",
  },
  {
    name: "idx_users_email",
    columns: ["email"],
    unique: false,
    tableName: "users",
    schema: "public",
  },
  {
    name: "idx_users_is_active",
    columns: ["is_active"],
    unique: false,
    tableName: "users",
    schema: "public",
  },
  {
    name: "categories_slug_key",
    columns: ["slug"],
    unique: true,
    tableName: "categories",
    schema: "public",
  },
  {
    name: "idx_products_category",
    columns: ["category_id"],
    unique: false,
    tableName: "products",
    schema: "public",
  },
  {
    name: "idx_products_price",
    columns: ["price"],
    unique: false,
    tableName: "products",
    schema: "public",
  },
  {
    name: "idx_orders_user",
    columns: ["user_id"],
    unique: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "idx_orders_status",
    columns: ["status"],
    unique: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "order_items_order_id_product_id_key",
    columns: ["order_id", "product_id"],
    unique: true,
    tableName: "order_items",
    schema: "public",
  },
  {
    name: "idx_order_items_order",
    columns: ["order_id"],
    unique: false,
    tableName: "order_items",
    schema: "public",
  },
  { name: "tags_name_key", columns: ["name"], unique: true, tableName: "tags", schema: "public" },
  {
    name: "product_tags_pkey",
    columns: ["product_id", "tag_id"],
    unique: true,
    tableName: "product_tags",
    schema: "public",
  },
  {
    name: "idx_audit_log_entity",
    columns: ["entity_type", "entity_id"],
    unique: false,
    tableName: "audit_log",
    schema: "public",
  },
  {
    name: "idx_audit_log_created",
    columns: ["created_at"],
    unique: false,
    tableName: "audit_log",
    schema: "public",
  },
];

const FOREIGN_KEYS: SchemaForeignKeyDto[] = [
  {
    name: "categories_parent_id_fkey",
    fromTable: "categories",
    fromColumns: ["parent_id"],
    toTable: "categories",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "products_category_id_fkey",
    fromTable: "products",
    fromColumns: ["category_id"],
    toTable: "categories",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "orders_user_id_fkey",
    fromTable: "orders",
    fromColumns: ["user_id"],
    toTable: "users",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "order_items_order_id_fkey",
    fromTable: "order_items",
    fromColumns: ["order_id"],
    toTable: "orders",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "order_items_product_id_fkey",
    fromTable: "order_items",
    fromColumns: ["product_id"],
    toTable: "products",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "product_tags_product_id_fkey",
    fromTable: "product_tags",
    fromColumns: ["product_id"],
    toTable: "products",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "product_tags_tag_id_fkey",
    fromTable: "product_tags",
    fromColumns: ["tag_id"],
    toTable: "tags",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
  {
    name: "audit_log_changed_by_fkey",
    fromTable: "audit_log",
    fromColumns: ["changed_by"],
    toTable: "users",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
  },
];

/* ------------------------------------------------------------------ */
/*  Lookup helpers                                                     */
/* ------------------------------------------------------------------ */

export function findTable(schema: string, name: string): MockTableDef | undefined {
  const bare = name.replace(/^"|"$/g, "");
  return MOCK_TABLES.find(
    (t) => t.name === bare && (schema === "public" || schema === "main" || t.schema === schema),
  );
}

export function findTableByBareName(name: string): MockTableDef | undefined {
  const unquoted = name.replace(/"/g, "");
  const parts = unquoted.split(".");
  const bare = parts[parts.length - 1];
  return MOCK_TABLES.find((t) => t.name === bare);
}

export function columnIndexOf(table: MockTableDef, name: string): number {
  return table.columns.findIndex((c) => c.name === name);
}

/* ------------------------------------------------------------------ */
/*  Cell conversion                                                    */
/* ------------------------------------------------------------------ */

export function toCell(column: MockColumnDef, raw: Raw): CellValue {
  if (raw === null || raw === undefined) return { type: "null" };

  switch (column.dataType) {
    case "bool":
      return { type: "bool", value: Boolean(raw) };
    case "int2":
    case "int4":
    case "int8":
    case "serial":
    case "bigserial":
      return { type: "int64", value: String(raw) };
    case "numeric":
    case "float4":
    case "float8":
      return { type: "float64", value: Number(raw) };
    case "uuid":
      return { type: "uuid", value: String(raw) };
    case "timestamptz":
    case "timestamp":
      return { type: "datetime", value: String(raw) };
    case "date":
      return { type: "date", value: String(raw) };
    case "time":
      return { type: "time", value: String(raw) };
    case "json":
    case "jsonb":
      return { type: "json", value: raw };
    case "bytea":
      return { type: "bytes", value: [] };
    default:
      return { type: "text", value: String(raw) };
  }
}

export function toRow(table: MockTableDef, raw: Raw[]): Row {
  return table.columns.map((col, i) => toCell(col, raw[i] ?? null));
}

/* ------------------------------------------------------------------ */
/*  Introspection builders                                             */
/* ------------------------------------------------------------------ */

export function buildIntrospectResult(): IntrospectResult {
  const schemas: SchemaDto[] = [{ name: "public" }];

  const tables: TableDto[] = MOCK_TABLES.filter((t) => !t.isView).map((t) => ({
    name: t.name,
    schema: t.schema,
    rowCount: t.rows.length,
  }));

  const views: ViewDto[] = MOCK_TABLES.filter((t) => t.isView).map((t) => ({
    name: t.name,
    schema: t.schema,
    definition: t.definition ?? `CREATE VIEW ${t.name} AS SELECT 1`,
  }));

  const columns: SchemaColumnDto[] = MOCK_TABLES.flatMap((t) =>
    t.columns.map((c) => ({
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
      defaultValue: c.defaultValue ?? null,
      isPrimaryKey: Boolean(c.primaryKey),
      tableName: t.name,
      schema: t.schema,
    })),
  );

  return {
    schemas,
    tables,
    columns,
    primaryKeys: PRIMARY_KEYS,
    indexes: INDEXES,
    foreignKeys: FOREIGN_KEYS,
    views,
    triggers: [] as TriggerDto[],
  };
}

export function buildTableInfo(schema: string, table: string): TableInfo | null {
  const def = findTable(schema, table);
  if (!def) return null;

  const columns: SchemaColumnDto[] = def.columns.map((c) => ({
    name: c.name,
    dataType: c.dataType,
    nullable: c.nullable,
    defaultValue: c.defaultValue ?? null,
    isPrimaryKey: Boolean(c.primaryKey),
    tableName: def.name,
    schema: def.schema,
  }));

  const pk = PRIMARY_KEYS.find((p) => p.tableName === def.name) ?? null;

  return {
    table: { name: def.name, schema: def.schema, rowCount: def.rows.length },
    columns,
    primaryKey: pk,
    indexes: INDEXES.filter((i) => i.tableName === def.name),
    foreignKeys: FOREIGN_KEYS.filter((fk) => fk.fromTable === def.name),
  };
}

/** Reconstructs a CREATE TABLE the way the Rust `get_table_ddl` command does. */
export function buildTableDdl(schema: string, table: string): string | null {
  const def = findTable(schema, table);
  if (!def) return null;

  if (def.isView) {
    return `CREATE OR REPLACE VIEW ${def.schema}.${def.name} AS\n${def.definition ?? "SELECT 1"};`;
  }

  const pk = PRIMARY_KEYS.find((p) => p.tableName === def.name);
  const lines = def.columns.map((c) => {
    const parts = [`    ${c.name} ${typeToSql(c.dataType)}`];
    if (!c.nullable) parts.push("NOT NULL");
    if (c.defaultValue) parts.push(`DEFAULT ${c.defaultValue}`);
    return parts.join(" ");
  });

  if (pk) lines.push(`    CONSTRAINT ${pk.constraintName} PRIMARY KEY (${pk.columns.join(", ")})`);

  const tableIndexes = INDEXES.filter(
    (i) => i.tableName === def.name && !i.unique && !i.name.endsWith("_pkey"),
  );
  const indexLines = tableIndexes.map(
    (i) => `CREATE INDEX ${i.name} ON ${def.schema}.${def.name} (${i.columns.join(", ")});`,
  );

  const fkLines = FOREIGN_KEYS.filter((fk) => fk.fromTable === def.name).map(
    (fk) =>
      `ALTER TABLE ${def.schema}.${def.name}\n    ADD CONSTRAINT ${fk.name} FOREIGN KEY (${fk.fromColumns.join(", ")})\n    REFERENCES ${fk.toSchema}.${fk.toTable} (${fk.toColumns.join(", ")});`,
  );

  return [
    `CREATE TABLE ${def.schema}.${def.name} (`,
    lines.join(",\n"),
    ");",
    "",
    ...indexLines,
    "",
    ...fkLines,
  ]
    .filter((l) => l !== undefined)
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function typeToSql(dataType: string): string {
  switch (dataType) {
    case "int8":
      return "BIGINT";
    case "int4":
      return "INTEGER";
    case "int2":
      return "SMALLINT";
    case "bool":
      return "BOOLEAN";
    case "numeric":
      return "NUMERIC(12, 2)";
    case "varchar":
      return "VARCHAR(255)";
    case "timestamptz":
      return "TIMESTAMPTZ";
    case "jsonb":
      return "JSONB";
    default:
      return dataType.toUpperCase();
  }
}

/* ------------------------------------------------------------------ */
/*  Minimal SQL engine (single-table subset)                           */
/* ------------------------------------------------------------------ */

/** Thrown for anything the mini engine cannot run; shaped like a Rust command error. */
export class MockSqlError extends Error {
  constructor(
    public error: string,
    message: string,
    public message_id: string,
  ) {
    super(message);
  }
}

interface ParsedSelect {
  projection: string[];
  table: MockTableDef;
  alias: string | null;
  predicates: Predicate[];
  orderBy: { column: string; desc: boolean }[];
  limit: number | null;
  offset: number;
}

interface Predicate {
  column: string;
  op: "eq" | "neq" | "lt" | "lte" | "gt" | "gte" | "like" | "in" | "isNull" | "isNotNull";
  value?: Raw[] | Raw;
}

function splitTopLevel(input: string, delimiter: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let quote: string | null = null;
  let current = "";
  for (let i = 0; i < input.length; i += 1) {
    const ch = input[i];
    if (quote) {
      current += ch;
      if (ch === quote && input[i - 1] !== "\\") quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      quote = ch;
      current += ch;
      continue;
    }
    if (ch === "(") depth += 1;
    if (ch === ")") depth -= 1;
    if (depth === 0 && input.slice(i, i + delimiter.length).toUpperCase() === delimiter) {
      out.push(current);
      current = "";
      i += delimiter.length - 1;
      continue;
    }
    current += ch;
  }
  if (current.trim()) out.push(current);
  return out.map((s) => s.trim()).filter(Boolean);
}

function stripQuotes(value: string): string {
  const trimmed = value.trim();
  if (
    (trimmed.startsWith("'") && trimmed.endsWith("'")) ||
    (trimmed.startsWith('"') && trimmed.endsWith('"'))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function literalToRaw(value: string): Raw {
  const trimmed = value.trim();
  if (trimmed.toUpperCase() === "NULL") return null;
  if (trimmed.toUpperCase() === "TRUE") return true;
  if (trimmed.toUpperCase() === "FALSE") return false;
  if (/^-?\d+$/.test(trimmed)) return Number(trimmed);
  if (/^-?\d*\.\d+$/.test(trimmed)) return Number(trimmed);
  return stripQuotes(trimmed);
}

function parsePredicate(expr: string, alias: string | null): Predicate | null {
  const text = expr.trim();
  const upper = text.toUpperCase();

  const isNull = /\bIS\s+NULL$/i.exec(text);
  if (isNull)
    return { column: stripAlias(text.slice(0, isNull.index).trim(), alias), op: "isNull" };

  const isNotNull = /\bIS\s+NOT\s+NULL$/i.exec(text);
  if (isNotNull) {
    return { column: stripAlias(text.slice(0, isNotNull.index).trim(), alias), op: "isNotNull" };
  }

  const inMatch = /^(.+?)\s+IN\s*\((.*)\)$/i.exec(text);
  if (inMatch) {
    return {
      column: stripAlias(inMatch[1].trim(), alias),
      op: "in",
      value: splitTopLevel(inMatch[2], ",").map(literalToRaw),
    };
  }

  const binary = /^(.+?)\s*(<>|!=|<=|>=|=|<|>)\s*(.+)$/s.exec(text);
  if (binary) {
    const op = binary[2];
    const mapped: Predicate["op"] =
      op === "<>" || op === "!="
        ? "neq"
        : op === "<="
          ? "lte"
          : op === ">="
            ? "gte"
            : op === "<"
              ? "lt"
              : op === ">"
                ? "gt"
                : "eq";
    return {
      column: stripAlias(binary[1].trim(), alias),
      op: mapped,
      value: literalToRaw(binary[3]),
    };
  }

  const like = /^(.+?)\s+(NOT\s+)?LIKE\s+(.+)$/i.exec(text);
  if (like) {
    const raw = literalToRaw(like[3]);
    const pattern = String(raw ?? "")
      .replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
      .replace(/%/g, ".*")
      .replace(/_/g, ".");
    return {
      column: stripAlias(like[1].trim(), alias),
      op: "like",
      value: like[2] ? `!${pattern}` : pattern,
    };
  }

  void upper;
  return null;
}

function stripAlias(column: string, alias: string | null): string {
  const cleaned = column.trim();
  if (alias && cleaned.toLowerCase().startsWith(`${alias.toLowerCase()}.`)) {
    return cleaned.slice(alias.length + 1);
  }
  return cleaned.replace(/^[a-z_][a-z0-9_]*\./i, "");
}

function parseSelect(sql: string): ParsedSelect {
  const text = sql
    .replace(/--.*$/gm, " ")
    .replace(/\/\*[\s\S]*?\*\//g, " ")
    .replace(/;+\s*$/, "")
    .trim();

  const selectMatch = /^SELECT\s+([\s\S]*?)\s+FROM\s+([\s\S]+)$/i.exec(text);
  if (!selectMatch) {
    throw new MockSqlError(
      "QUERY_SYNTAX_ERROR",
      `Browser backend only understands SELECT ... FROM <table>: ${text.slice(0, 60)}`,
      "error.query.syntax",
    );
  }

  let rest = selectMatch[2];
  let limit: number | null = null;
  let offset = 0;
  const orderBy: { column: string; desc: boolean }[] = [];
  const predicates: Predicate[] = [];

  const limitMatch = /\bLIMIT\s+(\d+)/i.exec(rest);
  if (limitMatch) {
    limit = Number(limitMatch[1]);
    rest = rest.replace(limitMatch[0], " ");
  }
  const offsetMatch = /\bOFFSET\s+(\d+)/i.exec(rest);
  if (offsetMatch) {
    offset = Number(offsetMatch[1]);
    rest = rest.replace(offsetMatch[0], " ");
  }
  const orderMatch = /\bORDER\s+BY\s+([\s\S]+?)(?=\bWHERE\b|$)/i.exec(rest);
  if (orderMatch) {
    for (const part of splitTopLevel(orderMatch[1], ",")) {
      const desc = /\bDESC$/i.test(part);
      const column = part.replace(/\b(ASC|DESC)$/i, "").trim();
      orderBy.push({ column, desc });
    }
    rest = rest.replace(orderMatch[0], " ");
  }
  const whereMatch = /\bWHERE\s+([\s\S]+)$/i.exec(rest);
  let alias: string | null = null;
  let tableExpr = rest;
  if (whereMatch) {
    for (const part of splitTopLevel(whereMatch[1], " AND ")) {
      predicates.push(parsePredicate(part, alias) ?? { column: "", op: "eq", value: null });
    }
    rest = rest.replace(whereMatch[0], " ");
    tableExpr = rest;
  }

  // FROM clause — take the first relation; JOINs are not modelled by the fixture engine.
  const fromParts = tableExpr.trim().split(/\s+/);
  const relation = fromParts[0] ?? "";
  const maybeAlias = fromParts[1];
  const aliasCandidate =
    maybeAlias &&
    !/^(INNER|LEFT|RIGHT|FULL|CROSS|JOIN|ON|WHERE|ORDER|LIMIT|GROUP)$/i.test(maybeAlias)
      ? maybeAlias.replace(/\bAS\b/i, "")
      : null;
  alias = aliasCandidate && aliasCandidate.toUpperCase() !== "AS" ? aliasCandidate : null;

  const [schemaPart, namePart] = relation.includes(".")
    ? relation.split(".")
    : ["public", relation];
  const table = findTable(schemaPart, namePart) ?? findTableByBareName(schemaPart);
  if (!table) {
    throw new MockSqlError(
      "QUERY_FAILED",
      `relation "${relation.replace(/^"|"$/g, "")}" does not exist`,
      "error.query.failed",
    );
  }

  // Predicates were parsed before the alias was known — normalise them now.
  const resolvedPredicates = predicates
    .map((p) => ({ ...p, column: stripAlias(p.column, alias) }))
    .filter((p) => p.column !== "");

  return {
    projection: splitTopLevel(selectMatch[1], ","),
    table,
    alias,
    predicates: resolvedPredicates,
    orderBy: orderBy.map((o) => ({ ...o, column: stripAlias(o.column, alias) })),
    limit,
    offset,
  };
}

function compareRaw(a: Raw, b: Raw): number {
  if (a === null && b === null) return 0;
  if (a === null) return 1;
  if (b === null) return -1;
  if (typeof a === "number" && typeof b === "number") return a - b;
  if (typeof a === "boolean" || typeof b === "boolean") {
    return Number(Boolean(a)) - Number(Boolean(b));
  }
  return String(a).localeCompare(String(b));
}

function matches(row: Raw[], table: MockTableDef, predicates: Predicate[]): boolean {
  return predicates.every((p) => {
    const idx = columnIndexOf(table, p.column);
    if (idx === -1) return true;
    const value = row[idx];
    switch (p.op) {
      case "isNull":
        return value === null;
      case "isNotNull":
        return value !== null;
      case "in":
        return Array.isArray(p.value)
          ? (p.value as Raw[]).some((v) => compareRaw(v, value) === 0)
          : false;
      case "like": {
        const pattern = String(p.value ?? "");
        const negate = pattern.startsWith("!");
        const regex = new RegExp(`^${negate ? pattern.slice(1) : pattern}$`, "i");
        return value !== null && regex.test(String(value)) !== negate;
      }
      default: {
        const cmp = compareRaw(value, (p.value as Raw) ?? null);
        if (p.op === "eq") return cmp === 0;
        if (p.op === "neq") return cmp !== 0;
        if (p.op === "lt") return cmp < 0;
        if (p.op === "lte") return cmp <= 0;
        if (p.op === "gt") return cmp > 0;
        return cmp >= 0;
      }
    }
  });
}

interface AggregateExpr {
  fn: "count" | "sum" | "avg" | "min" | "max";
  column: string | "*";
  label: string;
}

interface ParsedProjection {
  columns: MockColumnDef[];
  /** Source column name per projection slot, or "" when it is not a plain column. */
  sources: string[];
  /** Aggregate expression per projection slot, or null for a plain column. */
  aggregates: (AggregateExpr | null)[];
}

function parseProjection(projection: string[], table: MockTableDef): ParsedProjection | "star" {
  if (projection.length === 1 && projection[0].trim() === "*") return "star";

  const columns: MockColumnDef[] = [];
  const sources: string[] = [];
  const aggregates: (AggregateExpr | null)[] = [];

  for (const expr of projection) {
    const asMatch = /^(.+?)\s+AS\s+([a-z_][a-z0-9_]*)$/i.exec(expr);
    const body = (asMatch ? asMatch[1] : expr).trim();
    const label = asMatch ? asMatch[2] : body.replace(/^[a-z_][a-z0-9_]*\./i, "");

    const fnMatch = /^(count|sum|avg|min|max)\s*\((.*)\)$/i.exec(body);
    if (fnMatch) {
      const fn = fnMatch[1].toLowerCase() as AggregateExpr["fn"];
      const arg = fnMatch[2].trim().replace(/^[a-z_][a-z0-9_]*\./i, "");
      aggregates.push({ fn, column: arg === "*" ? "*" : arg, label });
      sources.push("");
      columns.push({ name: label, dataType: fn === "count" ? "int8" : "numeric", nullable: true });
      continue;
    }

    const colName = body.replace(/^[a-z_][a-z0-9_]*\./i, "");
    const found = table.columns.find((c) => c.name === colName);
    if (!found) {
      aggregates.push(null);
      sources.push("");
      columns.push({ name: label, dataType: "text", nullable: true });
      continue;
    }
    aggregates.push(null);
    sources.push(found.name);
    columns.push({ ...found, name: label });
  }

  return { columns, sources, aggregates };
}

function aggregateValue(
  fn: AggregateExpr["fn"],
  column: string,
  table: MockTableDef,
  rows: Raw[][],
): Raw {
  if (fn === "count") {
    if (column === "*") return rows.length;
    const idx = columnIndexOf(table, column);
    return idx === -1 ? 0 : rows.filter((r) => r[idx] !== null).length;
  }
  const idx = columnIndexOf(table, column);
  if (idx === -1) return null;
  const values = rows.map((r) => Number(r[idx])).filter((n) => !Number.isNaN(n));
  if (values.length === 0) return null;
  if (fn === "sum") return Math.round(values.reduce((a, b) => a + b, 0) * 100) / 100;
  if (fn === "avg")
    return Math.round((values.reduce((a, b) => a + b, 0) / values.length) * 100) / 100;
  if (fn === "min") return Math.min(...values);
  return Math.max(...values);
}

/** Runs a single statement against the fixture dataset. */
export function runStatement(sql: string): QueryResult {
  const started = performance.now();
  const trimmed = sql.trim();
  const isWrite = /^(INSERT|UPDATE|DELETE|CREATE|DROP|ALTER|TRUNCATE|GRANT|REVOKE|COMMENT)\b/i.test(
    trimmed,
  );

  if (isWrite) {
    const affected = applyWrite(trimmed);
    return {
      columns: [],
      rows: [],
      rowCount: affected,
      durationMs: Math.max(1, Math.round(performance.now() - started)),
    };
  }

  const parsed = parseSelect(trimmed);
  const { table, predicates, orderBy, limit, offset } = parsed;

  let working = table.rows.filter((row) => matches(row, table, predicates));

  for (const order of [...orderBy].reverse()) {
    const idx = columnIndexOf(table, order.column);
    if (idx === -1) continue;
    working = [...working].sort((a, b) => {
      const cmp = compareRaw(a[idx], b[idx]);
      return order.desc ? -cmp : cmp;
    });
  }

  if (offset > 0) working = working.slice(offset);
  if (limit !== null) working = working.slice(0, limit);

  const projection = parseProjection(parsed.projection, table);
  let columns: ColumnMeta[];
  let rows: Row[];

  if (projection === "star") {
    columns = table.columns.map((c) => ({
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
    }));
    rows = working.map((raw) => toRow(table, raw));
  } else {
    const { columns: projColumns, sources, aggregates } = projection;
    const hasAggregates = aggregates.some((a) => a !== null);
    columns = projColumns.map((c) => ({
      name: c.name,
      dataType: c.dataType,
      nullable: c.nullable,
    }));

    if (hasAggregates) {
      const values: CellValue[] = projColumns.map((col, i) => {
        const agg = aggregates[i];
        if (!agg) {
          const idx = columnIndexOf(table, sources[i] ?? col.name);
          if (idx === -1) return { type: "null" } as const;
          return toCell(table.columns[idx], working[0]?.[idx] ?? null);
        }
        const value = aggregateValue(agg.fn, agg.column, table, working);
        return toCell({ ...col, dataType: col.dataType }, value);
      });
      rows = [values];
    } else {
      rows = working.map((raw) =>
        projColumns.map((col, i): CellValue => {
          const source = sources[i] ?? col.name;
          const idx = columnIndexOf(table, source);
          if (idx === -1) return { type: "null" } as const;
          return toCell(table.columns[idx], raw[idx] ?? null);
        }),
      );
    }
  }

  return {
    columns,
    rows,
    rowCount: rows.length,
    durationMs: Math.max(1, Math.round(performance.now() - started)),
  };
}

/** Applies a single-statement write against the in-memory rows. */
function applyWrite(sql: string): number {
  const upper = sql.toUpperCase();

  if (upper.startsWith("INSERT")) {
    const match = /^INSERT\s+INTO\s+([a-z_0-9."]+)\s*\(([^)]+)\)\s*VALUES\s*([\s\S]+)$/i.exec(sql);
    if (!match) return 0;
    const table = findTableByBareName(match[1]);
    if (!table || table.isView) return 0;
    const names = splitTopLevel(match[2], ",").map((n) => n.trim());
    const tuples = match[3]
      .replace(/^\s*\(/, "")
      .replace(/\)\s*$/, "")
      .split(/\)\s*,\s*\(/)
      .map((tuple) => splitTopLevel(tuple.replace(/^\(|\)$/g, ""), ",").map(literalToRaw));
    for (const tuple of tuples) {
      const next: Raw[] = table.columns.map((c) => (c.primaryKey ? nextId(table) : null));
      names.forEach((name, i) => {
        const idx = columnIndexOf(table, name);
        if (idx !== -1) next[idx] = tuple[i] ?? null;
      });
      table.rows.push(next);
    }
    return tuples.length;
  }

  if (upper.startsWith("UPDATE")) {
    const match = /^UPDATE\s+([a-z_0-9."]+)\s+SET\s+([\s\S]+?)(?:\s+WHERE\s+([\s\S]+))?$/i.exec(
      sql,
    );
    if (!match) return 0;
    const table = findTableByBareName(match[1]);
    if (!table || table.isView) return 0;
    const assignments = splitTopLevel(match[2], ",").map((part) => {
      const [name, value] = part.split("=");
      return { column: name.trim(), value: literalToRaw(value ?? "") };
    });
    const predicates = match[3]
      ? splitTopLevel(match[3], " AND ")
          .map((p) => parsePredicate(p, null))
          .filter((p): p is Predicate => p !== null && p.column !== "")
      : [];
    let affected = 0;
    for (const row of table.rows) {
      if (!matches(row, table, predicates)) continue;
      for (const assignment of assignments) {
        const idx = columnIndexOf(table, assignment.column);
        if (idx !== -1) row[idx] = assignment.value;
      }
      affected += 1;
    }
    return affected;
  }

  if (upper.startsWith("DELETE")) {
    const match = /^DELETE\s+FROM\s+([a-z_0-9."]+)(?:\s+WHERE\s+([\s\S]+))?$/i.exec(sql);
    if (!match) return 0;
    const table = findTableByBareName(match[1]);
    if (!table || table.isView) return 0;
    const predicates = match[2]
      ? splitTopLevel(match[2], " AND ")
          .map((p) => parsePredicate(p, null))
          .filter((p): p is Predicate => p !== null && p.column !== "")
      : [];
    const before = table.rows.length;
    table.rows = table.rows.filter((row) => !matches(row, table, predicates));
    return before - table.rows.length;
  }

  // DDL and everything else: accept and report no row effect.
  return 0;
}

function nextId(table: MockTableDef): number {
  const idx = table.columns.findIndex((c) => c.primaryKey);
  if (idx === -1) return 0;
  return table.rows.reduce((max, row) => Math.max(max, Number(row[idx]) || 0), 0) + 1;
}

/** PostgreSQL-shaped EXPLAIN output for the explain panel. */
export function buildExplainPlan(sql: string): Record<string, unknown> {
  let relation = "orders";
  let planRows = 12;
  try {
    const parsed = parseSelect(sql);
    relation = parsed.table.name;
    planRows = parsed.table.rows.length;
  } catch {
    // fall through with defaults
  }

  const seqScan = {
    "Node Type": "Seq Scan",
    "Relation Name": relation,
    Alias: relation,
    "Startup Cost": 0,
    "Total Cost": Math.round(planRows * 1.35 * 100) / 100,
    "Plan Rows": planRows,
    "Plan Width": 96,
    "Actual Total Time": 0.42,
    "Actual Rows": planRows,
  };

  return {
    Plan: {
      "Node Type": "Limit",
      "Startup Cost": 0,
      "Total Cost": Math.round(planRows * 1.35 * 100) / 100 + 0.01,
      "Plan Rows": planRows,
      "Plan Width": 96,
      "Actual Total Time": 0.51,
      "Actual Rows": planRows,
      Plans: [
        {
          "Node Type": "Sort",
          "Startup Cost": 1.1,
          "Total Cost": Math.round(planRows * 1.3 * 100) / 100,
          "Plan Rows": planRows,
          "Plan Width": 96,
          "Sort Key": ["created_at DESC"],
          "Actual Total Time": 0.38,
          "Actual Rows": planRows,
          Plans: [seqScan],
        },
      ],
    },
  };
}
