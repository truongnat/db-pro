import { describe, expect, it } from "vitest";

import type {
  IntrospectResult,
  PrimaryKeyDto,
  SchemaColumnDto,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

import type { TableNodeData } from "../components/lod/types";
import {
  buildColumnsByTable,
  buildPrimaryKeysByTable,
  buildFkColumnSet,
  buildTableNodes,
  buildErNodeIndexes,
} from "../renderer/er-node-builder";
import { generateErFixture } from "./er-fixture";

/* ── Hand-built small fixture with composite PK + FK ─────────────────────── */

const ORGS: TableDto = { name: "orgs", schema: "public", rowCount: 10 };
const USERS: TableDto = { name: "users", schema: "public", rowCount: 100 };
const AUDIT: TableDto = { name: "audit", schema: "public", rowCount: 5000 };

const cols: SchemaColumnDto[] = [
  // orgs
  {
    name: "id",
    dataType: "serial",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: true,
    tableName: "orgs",
    schema: "public",
  },
  {
    name: "name",
    dataType: "text",
    nullable: true,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "orgs",
    schema: "public",
  },
  // users
  {
    name: "id",
    dataType: "serial",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: true,
    tableName: "users",
    schema: "public",
  },
  {
    name: "email",
    dataType: "text",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "users",
    schema: "public",
  },
  {
    name: "org_id",
    dataType: "integer",
    nullable: true,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "users",
    schema: "public",
  },
  // audit — composite PK (tenant_id, id)
  {
    name: "tenant_id",
    dataType: "integer",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: true,
    tableName: "audit",
    schema: "public",
  },
  {
    name: "id",
    dataType: "bigint",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: true,
    tableName: "audit",
    schema: "public",
  },
];

const pks: PrimaryKeyDto[] = [
  { constraintName: "orgs_pkey", columns: ["id"], tableName: "orgs", schema: "public" },
  { constraintName: "users_pkey", columns: ["id"], tableName: "users", schema: "public" },
  {
    constraintName: "audit_pkey",
    columns: ["tenant_id", "id"],
    tableName: "audit",
    schema: "public",
  },
];

const fks: SchemaForeignKeyDto[] = [
  {
    name: "users_org_id_fkey",
    fromTable: "users",
    fromColumn: "org_id",
    toTable: "orgs",
    toColumn: "id",
    schema: "public",
    toSchema: "public",
  },
];

/* ── Pre-index builders ──────────────────────────────────────────────────── */

describe("buildColumnsByTable", () => {
  it("indexes every column under schema.tableName", () => {
    const map = buildColumnsByTable(cols);
    expect(map.size).toBe(3);
    expect(map.get("public.orgs")).toHaveLength(2);
    expect(map.get("public.users")).toHaveLength(3);
    expect(map.get("public.audit")).toHaveLength(2);
  });
});

describe("buildPrimaryKeysByTable", () => {
  it("merges composite PK columns into one set per table", () => {
    const map = buildPrimaryKeysByTable(pks);
    expect(map.get("public.audit")).toEqual(new Set(["tenant_id", "id"]));
    expect(map.get("public.users")).toEqual(new Set(["id"]));
    expect(map.get("public.orgs")).toEqual(new Set(["id"]));
  });
});

describe("buildFkColumnSet", () => {
  it("indexes FK source columns as schema.table:column", () => {
    const set = buildFkColumnSet(fks);
    expect(set.has("public.users:org_id")).toBe(true);
    expect(set.has("public.users:id")).toBe(false);
    expect(set.has("public.orgs:id")).toBe(false);
  });
});

/* ── buildTableNodes ─────────────────────────────────────────────────────── */

describe("buildTableNodes", () => {
  it("attaches columns with PK/FK flags via O(1) index lookups", () => {
    const indexes = {
      columnsByTable: buildColumnsByTable(cols),
      primaryKeysByTable: buildPrimaryKeysByTable(pks),
      fkColumnSet: buildFkColumnSet(fks),
    };
    const nodes = buildTableNodes([ORGS, USERS, AUDIT], indexes, { compact: false });
    expect(nodes.map((n) => n.id)).toEqual(["public.orgs", "public.users", "public.audit"]);

    const usersData = nodes.find((n) => n.id === "public.users")!.data as TableNodeData;
    expect(usersData.columns).toEqual([
      { name: "id", dataType: "serial", nullable: false, isPrimaryKey: true, isForeignKey: false },
      {
        name: "email",
        dataType: "text",
        nullable: false,
        isPrimaryKey: false,
        isForeignKey: false,
      },
      {
        name: "org_id",
        dataType: "integer",
        nullable: true,
        isPrimaryKey: false,
        isForeignKey: true,
      },
    ]);

    const auditData = nodes.find((n) => n.id === "public.audit")!.data as TableNodeData;
    expect(auditData.columns.map((c) => [c.name, c.isPrimaryKey])).toEqual([
      ["tenant_id", true],
      ["id", true],
    ]);
  });

  it("is behavior-identical to a naive per-table filter reference (parity)", () => {
    const data = generateErFixture(100);
    const indexes = buildErNodeIndexes(data);
    const tables = data.tables.filter((t) => t.schema === "public");
    const actual = buildTableNodes(tables, indexes, { compact: false });

    // The O(T×C) reference implementation — must produce identical output.
    const reference = tables.map((table) => {
      const tableKey = `${table.schema}.${table.name}`;
      const tableCols = data.columns.filter((c) => `${c.schema}.${c.tableName}` === tableKey);
      const pkCols = new Set(
        data.primaryKeys
          .filter((pk) => `${pk.schema}.${pk.tableName}` === tableKey)
          .flatMap((pk) => pk.columns),
      );
      const fkCols = new Set(
        data.foreignKeys
          .filter((fk) => `${fk.schema}.${fk.fromTable}` === tableKey)
          .map((fk) => fk.fromColumn),
      );
      return {
        id: tableKey,
        type: "table",
        position: { x: 0, y: 0 },
        data: {
          label: table.name,
          schema: table.schema,
          columns: tableCols.map((col) => ({
            name: col.name,
            dataType: col.dataType,
            nullable: col.nullable,
            isPrimaryKey: pkCols.has(col.name),
            isForeignKey: fkCols.has(col.name),
          })),
          compact: false,
          lod: "detail",
        },
      };
    });

    expect(actual.map(({ id, type, position, data }) => ({ id, type, position, data }))).toEqual(
      reference,
    );
  });

  it("builds 500 tables with no dropped columns and all FK flags (scale parity)", () => {
    const data: IntrospectResult = generateErFixture(500);
    const indexes = buildErNodeIndexes(data);
    const nodes = buildTableNodes(data.tables, indexes, { compact: false });

    expect(nodes).toHaveLength(500);

    // Every column in the fixture appears on exactly one node.
    const totalNodeColumns = nodes.reduce(
      (sum, n) => sum + (n.data as TableNodeData).columns.length,
      0,
    );
    expect(totalNodeColumns).toBe(data.columns.length);

    // Every fixture FK source column is flagged on its owning node.
    for (const fk of data.foreignKeys) {
      const node = nodes.find((n) => n.id === `${fk.schema}.${fk.fromTable}`);
      expect(node, `node for ${fk.fromTable}`).toBeDefined();
      const col = (node!.data as TableNodeData).columns.find((c) => c.name === fk.fromColumn);
      expect(col?.isForeignKey).toBe(true);
    }
  });
});
