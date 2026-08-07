import { describe, it, expect } from "vitest";
import {
  parseCsv,
  parseJson,
  detectFormat,
  buildImportPreview,
  parseAllRows,
} from "../services/import-parser";

describe("parseCsv", () => {
  it("parses basic CSV", () => {
    const { headers, rows } = parseCsv("id,name\n1,Alice\n2,Bob");
    expect(headers).toEqual(["id", "name"]);
    expect(rows).toEqual([["1", "Alice"], ["2", "Bob"]]);
  });

  it("handles quoted fields with commas", () => {
    const { rows } = parseCsv('id,name\n1,"Alice, Jr."');
    expect(rows[0]).toEqual(["1", "Alice, Jr."]);
  });

  it("handles escaped quotes", () => {
    const { rows } = parseCsv('id,name\n1,"Say ""hello"""');
    expect(rows[0]).toEqual(["1", 'Say "hello"']);
  });

  it("handles newlines within quotes", () => {
    const { rows } = parseCsv('id,name\n1,"line1\nline2"');
    expect(rows[0]).toEqual(["1", "line1\nline2"]);
  });

  it("handles empty content", () => {
    const { headers, rows } = parseCsv("");
    expect(headers).toEqual([]);
    expect(rows).toEqual([]);
  });

  it("handles headers only", () => {
    const { headers, rows } = parseCsv("id,name");
    expect(headers).toEqual(["id", "name"]);
    expect(rows).toEqual([]);
  });

  it("handles CRLF line endings", () => {
    const { rows } = parseCsv("id,name\r\n1,Alice\r\n2,Bob");
    expect(rows).toEqual([["1", "Alice"], ["2", "Bob"]]);
  });
});

describe("parseJson", () => {
  it("parses JSON array of objects", () => {
    const { keys, rows } = parseJson('[{"id":1,"name":"Alice"},{"id":2}]');
    expect(keys).toEqual(["id", "name"]);
    expect(rows).toHaveLength(2);
    expect(rows[0].name).toBe("Alice");
  });

  it("handles empty array", () => {
    const { keys, rows } = parseJson("[]");
    expect(keys).toEqual([]);
    expect(rows).toEqual([]);
  });
});

describe("detectFormat", () => {
  it("detects JSON", () => {
    expect(detectFormat('[{"a":1}]')).toBe("json");
    expect(detectFormat('  [{"a":1}]')).toBe("json");
  });

  it("detects CSV", () => {
    expect(detectFormat("id,name\n1,Alice")).toBe("csv");
  });
});

describe("buildImportPreview", () => {
  it("builds preview for CSV with auto-mapping", () => {
    const preview = buildImportPreview(
      "id,name\n1,Alice\n2,Bob",
      "csv",
      ["id", "name", "email"],
    );
    expect(preview.sourceColumns).toEqual(["id", "name"]);
    expect(preview.totalRowCount).toBe(2);
    expect(preview.sampleRows).toHaveLength(2);
    expect(preview.mappings[0].targetColumn).toBe("id");
    expect(preview.mappings[1].targetColumn).toBe("name");
  });

  it("builds preview for JSON", () => {
    const preview = buildImportPreview(
      '[{"id":1,"name":"Alice"}]',
      "json",
      ["id", "name"],
    );
    expect(preview.sourceColumns).toEqual(["id", "name"]);
    expect(preview.totalRowCount).toBe(1);
    expect(preview.mappings[0].targetColumn).toBe("id");
  });

  it("does not auto-map unmatched columns", () => {
    const preview = buildImportPreview("foo,bar\n1,2", "csv", ["id", "name"]);
    expect(preview.mappings[0].targetColumn).toBeNull();
    expect(preview.mappings[1].targetColumn).toBeNull();
  });
});

describe("parseAllRows", () => {
  it("parses all CSV rows", () => {
    const rows = parseAllRows("id,name\n1,Alice\n2,Bob", "csv");
    expect(rows).toEqual([
      { id: "1", name: "Alice" },
      { id: "2", name: "Bob" },
    ]);
  });

  it("parses all JSON rows", () => {
    const rows = parseAllRows('[{"id":1,"name":"Alice"}]', "json");
    expect(rows).toEqual([{ id: "1", name: "Alice" }]);
  });
});
