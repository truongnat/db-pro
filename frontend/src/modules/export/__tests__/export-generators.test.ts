import { describe, it, expect } from "vitest";
import { generateCsv, generateJson, generateSqlInserts } from "../services/export-generators";
import type { ColumnMeta, Row } from "@/modules/query/types/query.types";

const columns: ColumnMeta[] = [
  { name: "id", data_type: "int", nullable: false },
  { name: "name", data_type: "text", nullable: true },
  { name: "active", data_type: "bool", nullable: false },
];

const rows: Row[] = [
  [
    { type: "int64", value: 1 },
    { type: "text", value: "Alice" },
    { type: "bool", value: true },
  ],
  [
    { type: "int64", value: 2 },
    { type: "null" },
    { type: "bool", value: false },
  ],
  [
    { type: "int64", value: 3 },
    { type: "text", value: "has,comma" },
    { type: "bool", value: true },
  ],
];

describe("generateCsv", () => {
  it("generates CSV with headers by default", () => {
    const csv = generateCsv(columns, rows);
    expect(csv).toContain("id,name,active");
    expect(csv).toContain("1,Alice,true");
    expect(csv).toContain("2,,false");
  });

  it("generates CSV without headers", () => {
    const csv = generateCsv(columns, rows, { includeHeaders: false });
    expect(csv).not.toContain("id,name,active");
    expect(csv.split("\n")).toHaveLength(3);
  });

  it("uses custom delimiter", () => {
    const csv = generateCsv(columns, rows, { delimiter: ";" });
    expect(csv).toContain("id;name;active");
    expect(csv).toContain("1;Alice;true");
  });

  it("escapes fields containing delimiter", () => {
    const csv = generateCsv(columns, rows, { delimiter: "," });
    expect(csv).toContain('"has,comma"');
  });

  it("uses custom NULL representation", () => {
    const csv = generateCsv(columns, rows, { nullRepresentation: "NULL" });
    expect(csv).toContain("2,NULL,false");
  });

  it("handles empty rows", () => {
    const csv = generateCsv(columns, []);
    expect(csv).toBe("id,name,active");
  });
});

describe("generateJson", () => {
  it("generates pretty JSON by default", () => {
    const json = generateJson(columns, rows);
    const parsed = JSON.parse(json);
    expect(parsed).toHaveLength(3);
    expect(parsed[0]).toEqual({ id: 1, name: "Alice", active: true });
    expect(parsed[1].name).toBeNull();
  });

  it("generates compact JSON", () => {
    const json = generateJson(columns, rows, { pretty: false });
    expect(json).not.toContain("\n");
    const parsed = JSON.parse(json);
    expect(parsed).toHaveLength(3);
  });

  it("uses custom NULL representation", () => {
    const json = generateJson(columns, rows, { nullRepresentation: "N/A" });
    const parsed = JSON.parse(json);
    expect(parsed[1].name).toBe("N/A");
  });
});

describe("generateSqlInserts", () => {
  it("generates INSERT statements", () => {
    const sql = generateSqlInserts(columns, rows, {
      tableName: "users",
      nullRepresentation: "",
    });
    expect(sql).toContain('INSERT INTO "users"');
    expect(sql).toContain('"id", "name", "active"');
    expect(sql).toContain("VALUES (1, 'Alice', TRUE)");
    expect(sql).toContain("VALUES (2, NULL, FALSE)");
  });

  it("escapes single quotes in text values", () => {
    const specialRows: Row[] = [
      [
        { type: "int64", value: 1 },
        { type: "text", value: "O'Brien" },
        { type: "bool", value: true },
      ],
    ];
    const sql = generateSqlInserts(columns, specialRows, {
      tableName: "users",
      nullRepresentation: "",
    });
    expect(sql).toContain("O''Brien");
  });

  it("uses custom NULL representation", () => {
    const sql = generateSqlInserts(columns, rows, {
      tableName: "users",
      nullRepresentation: "NULL",
    });
    expect(sql).toContain("VALUES (2, 'NULL', FALSE)");
  });

  it("handles JSON cell type with escaped single quotes", () => {
    const jsonColumns: ColumnMeta[] = [
      { name: "id", data_type: "int", nullable: false },
      { name: "metadata", data_type: "json", nullable: true },
    ];
    const jsonRows: Row[] = [
      [
        { type: "int64", value: 1 },
        { type: "json", value: { key: "it's", nested: [1, 2] } },
      ],
    ];
    const sql = generateSqlInserts(jsonColumns, jsonRows, {
      tableName: "docs",
      nullRepresentation: "",
    });
    expect(sql).toContain('INSERT INTO "docs"');
    // JSON.stringify produces the string, then single quotes are escaped
    expect(sql).toContain("it''s");
  });

  it("handles bytes cell type in CSV as [binary]", () => {
    const binColumns: ColumnMeta[] = [
      { name: "id", data_type: "int", nullable: false },
      { name: "data", data_type: "bytea", nullable: true },
    ];
    const binRows: Row[] = [
      [
        { type: "int64", value: 1 },
        { type: "bytes", value: "\x00\x01" },
      ],
    ];
    const csv = generateCsv(binColumns, binRows);
    expect(csv).toContain("[binary]");
  });
});
