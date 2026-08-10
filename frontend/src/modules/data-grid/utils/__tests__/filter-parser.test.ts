import { describe, expect, it } from "vitest";

import { parseFilterValue } from "../filter-parser";
import type { ColumnMeta } from "../../types/data-grid.types";

function col(dataType: string, name = "test_col"): ColumnMeta {
  return { name, dataType, nullable: true };
}

describe("parseFilterValue", () => {
  describe("integer types", () => {
    it("parses int4 (PostgreSQL sqlx format)", () => {
      expect(parseFilterValue(col("INT4"), "42")).toEqual({ type: "int64", value: 42 });
      expect(parseFilterValue(col("int4"), "42")).toEqual({ type: "int64", value: 42 });
    });

    it("parses int8 (bigint)", () => {
      expect(parseFilterValue(col("INT8"), "9999999999")).toEqual({ type: "int64", value: 9999999999 });
    });

    it("parses int2 (smallint)", () => {
      expect(parseFilterValue(col("INT2"), "7")).toEqual({ type: "int64", value: 7 });
    });

    it("parses information_schema format: integer, bigint, smallint", () => {
      expect(parseFilterValue(col("integer"), "10")).toEqual({ type: "int64", value: 10 });
      expect(parseFilterValue(col("bigint"), "20")).toEqual({ type: "int64", value: 20 });
      expect(parseFilterValue(col("smallint"), "5")).toEqual({ type: "int64", value: 5 });
    });

    it("parses serial types", () => {
      expect(parseFilterValue(col("serial"), "1")).toEqual({ type: "int64", value: 1 });
      expect(parseFilterValue(col("bigserial"), "2")).toEqual({ type: "int64", value: 2 });
      expect(parseFilterValue(col("smallserial"), "3")).toEqual({ type: "int64", value: 3 });
    });

    it("falls back to text for non-numeric input on integer column", () => {
      expect(parseFilterValue(col("INT4"), "abc")).toEqual({ type: "text", value: "abc" });
    });

    it("handles negative integers", () => {
      expect(parseFilterValue(col("INT4"), "-5")).toEqual({ type: "int64", value: -5 });
    });
  });

  describe("float types", () => {
    it("parses float4 and float8", () => {
      expect(parseFilterValue(col("FLOAT4"), "3.14")).toEqual({ type: "float64", value: 3.14 });
      expect(parseFilterValue(col("FLOAT8"), "2.718")).toEqual({ type: "float64", value: 2.718 });
    });

    it("parses numeric/decimal with precision", () => {
      expect(parseFilterValue(col("numeric"), "99.99")).toEqual({ type: "float64", value: 99.99 });
      expect(parseFilterValue(col("numeric(10,2)"), "123.45")).toEqual({ type: "float64", value: 123.45 });
      expect(parseFilterValue(col("decimal(8,3)"), "1.234")).toEqual({ type: "float64", value: 1.234 });
    });

    it("parses real and double precision", () => {
      expect(parseFilterValue(col("real"), "1.5")).toEqual({ type: "float64", value: 1.5 });
      expect(parseFilterValue(col("double precision"), "2.5")).toEqual({ type: "float64", value: 2.5 });
    });

    it("falls back to text for non-numeric input on float column", () => {
      expect(parseFilterValue(col("FLOAT4"), "xyz")).toEqual({ type: "text", value: "xyz" });
    });
  });

  describe("boolean types", () => {
    it("parses true variants", () => {
      expect(parseFilterValue(col("BOOL"), "true")).toEqual({ type: "bool", value: true });
      expect(parseFilterValue(col("BOOL"), "TRUE")).toEqual({ type: "bool", value: true });
      expect(parseFilterValue(col("BOOL"), "1")).toEqual({ type: "bool", value: true });
      expect(parseFilterValue(col("BOOL"), "yes")).toEqual({ type: "bool", value: true });
      expect(parseFilterValue(col("BOOL"), "t")).toEqual({ type: "bool", value: true });
    });

    it("parses false variants", () => {
      expect(parseFilterValue(col("BOOL"), "false")).toEqual({ type: "bool", value: false });
      expect(parseFilterValue(col("BOOL"), "FALSE")).toEqual({ type: "bool", value: false });
      expect(parseFilterValue(col("BOOL"), "0")).toEqual({ type: "bool", value: false });
      expect(parseFilterValue(col("BOOL"), "no")).toEqual({ type: "bool", value: false });
      expect(parseFilterValue(col("BOOL"), "f")).toEqual({ type: "bool", value: false });
    });

    it("falls back to text for unrecognized boolean input", () => {
      expect(parseFilterValue(col("BOOL"), "maybe")).toEqual({ type: "text", value: "maybe" });
    });

    it("handles boolean (information_schema format)", () => {
      expect(parseFilterValue(col("boolean"), "true")).toEqual({ type: "bool", value: true });
    });
  });

  describe("UUID type", () => {
    it("parses uuid", () => {
      const uuid = "550e8400-e29b-41d4-a716-446655440000";
      expect(parseFilterValue(col("UUID"), uuid)).toEqual({ type: "uuid", value: uuid });
      expect(parseFilterValue(col("uuid"), `  ${uuid}  `)).toEqual({ type: "uuid", value: uuid });
    });
  });

  describe("datetime types", () => {
    it("parses date", () => {
      expect(parseFilterValue(col("DATE"), "2024-01-15")).toEqual({
        type: "datetime",
        value: "2024-01-15",
      });
    });

    it("parses timestamp and timestamptz", () => {
      expect(parseFilterValue(col("TIMESTAMP"), "2024-01-15 10:30:00")).toEqual({
        type: "datetime",
        value: "2024-01-15 10:30:00",
      });
      expect(parseFilterValue(col("TIMESTAMPTZ"), "2024-01-15T10:30:00Z")).toEqual({
        type: "datetime",
        value: "2024-01-15T10:30:00Z",
      });
    });

    it("parses time and timetz", () => {
      expect(parseFilterValue(col("TIME"), "10:30:00")).toEqual({
        type: "datetime",
        value: "10:30:00",
      });
      expect(parseFilterValue(col("TIMETZ"), "10:30:00+05")).toEqual({
        type: "datetime",
        value: "10:30:00+05",
      });
    });
  });

  describe("JSON types", () => {
    it("parses valid JSON", () => {
      expect(parseFilterValue(col("JSON"), '{"key":"value"}')).toEqual({
        type: "json",
        value: { key: "value" },
      });
      expect(parseFilterValue(col("JSONB"), "[1,2,3]")).toEqual({
        type: "json",
        value: [1, 2, 3],
      });
    });

    it("falls back to text for invalid JSON", () => {
      expect(parseFilterValue(col("JSON"), "not json")).toEqual({
        type: "text",
        value: "not json",
      });
    });
  });

  describe("text fallback", () => {
    it("returns text for unknown types", () => {
      expect(parseFilterValue(col("TEXT"), "hello")).toEqual({ type: "text", value: "hello" });
      expect(parseFilterValue(col("VARCHAR"), "world")).toEqual({ type: "text", value: "world" });
      expect(parseFilterValue(col("citext"), "mixed")).toEqual({ type: "text", value: "mixed" });
    });

    it("returns text for empty dataType", () => {
      expect(parseFilterValue(col(""), "anything")).toEqual({ type: "text", value: "anything" });
    });
  });
});
