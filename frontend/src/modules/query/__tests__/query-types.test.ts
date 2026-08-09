import { describe, expect, it } from "vitest";
import { renderCellValue, type CellValue } from "../types/query.types";

describe("renderCellValue", () => {
  it("renders null", () => {
    const cell: CellValue = { type: "null" };
    expect(renderCellValue(cell)).toBe("NULL");
  });

  it("renders boolean true", () => {
    expect(renderCellValue({ type: "bool", value: true })).toBe("true");
  });

  it("renders boolean false", () => {
    expect(renderCellValue({ type: "bool", value: false })).toBe("false");
  });

  it("renders int64", () => {
    expect(renderCellValue({ type: "int64", value: 42 })).toBe("42");
  });

  it("renders float64", () => {
    expect(renderCellValue({ type: "float64", value: 3.14 })).toBe("3.14");
  });

  it("renders text", () => {
    expect(renderCellValue({ type: "text", value: "hello" })).toBe("hello");
  });

  it("renders uuid", () => {
    expect(renderCellValue({ type: "uuid", value: "abc-123" })).toBe("abc-123");
  });

  it("renders datetime", () => {
    expect(renderCellValue({ type: "datetime", value: "2024-01-01T00:00:00Z" })).toBe(
      "2024-01-01T00:00:00Z",
    );
  });

  it("renders bytes with length", () => {
    const cell: CellValue = { type: "bytes", value: [1, 2, 3] };
    expect(renderCellValue(cell)).toBe("<binary (3 bytes)>");
  });

  it("renders empty bytes", () => {
    const cell: CellValue = { type: "bytes", value: [] };
    expect(renderCellValue(cell)).toBe("<binary (0 bytes)>");
  });

  it("renders json as pretty-printed string", () => {
    const cell: CellValue = { type: "json", value: { a: 1 } };
    expect(renderCellValue(cell)).toBe(JSON.stringify({ a: 1 }, null, 2));
  });

  it("renders json null", () => {
    const cell: CellValue = { type: "json", value: null };
    expect(renderCellValue(cell)).toBe("null");
  });
});
