import { describe, expect, it } from "vitest";

import { getSqlDialect } from "@/modules/query/sql/dialect";
import {
  buildCreateTable,
  generateDdlPreview,
  type ColumnDef,
} from "@/modules/schema/services/ddl-builder";

const COLUMNS: ColumnDef[] = [
  { name: "id", dataType: "BIGSERIAL", nullable: false, defaultValue: "", isPk: true },
  { name: "email", dataType: "TEXT", nullable: false, defaultValue: "", isPk: false },
];

describe("dialect-aware DDL builder (UX-R7.2b)", () => {
  it("defaults to postgres dialect", () => {
    expect(buildCreateTable("public", "users", COLUMNS)).toBe(
      `CREATE TABLE "public"."users" (\n    "id" BIGSERIAL NOT NULL,\n    "email" TEXT NOT NULL,\n    PRIMARY KEY ("id")\n);`,
    );
  });

  it("escapes identifiers via the dialect", () => {
    const pg = getSqlDialect("postgres");
    const sqlite = getSqlDialect("sqlite");
    const oddCol: ColumnDef[] = [
      { name: 'a"b', dataType: "TEXT", nullable: true, defaultValue: "", isPk: false },
    ];
    expect(buildCreateTable("public", "t", oddCol, pg)).toContain('"a""b" TEXT');
    expect(buildCreateTable("main", "t", oddCol, sqlite)).toContain('"a""b" TEXT');
  });

  it("qualifies schema on both dialects", () => {
    const pg = getSqlDialect("postgres");
    const sqlite = getSqlDialect("sqlite");
    expect(generateDdlPreview("createTable", "public", "users", COLUMNS, {}, pg)).toContain(
      'CREATE TABLE "public"."users"',
    );
    expect(generateDdlPreview("createTable", "main", "users", COLUMNS, {}, sqlite)).toContain(
      'CREATE TABLE "main"."users"',
    );
  });
});
