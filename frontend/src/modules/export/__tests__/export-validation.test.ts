import { describe, expect, it } from "vitest";

import { getExportValidationError } from "../components/export-dialog";

const base = {
  rowCount: 1,
  connectionId: "conn-1",
  sql: "SELECT 1",
  tableName: "target",
};

describe("getExportValidationError", () => {
  it("requires result rows", () => {
    expect(getExportValidationError({ ...base, format: "csv", rowCount: 0 })).toBe("export.noRows");
  });

  it("requires a table name for SQL inserts", () => {
    expect(getExportValidationError({ ...base, format: "sql", tableName: " " })).toBe(
      "export.tableNameRequired",
    );
  });

  it("requires a connection and query for Excel", () => {
    expect(getExportValidationError({ ...base, format: "excel", connectionId: null })).toBe(
      "export.connectionRequired",
    );
    expect(getExportValidationError({ ...base, format: "excel", sql: "" })).toBe(
      "export.queryRequired",
    );
  });

  it("allows a valid export", () => {
    expect(getExportValidationError({ ...base, format: "json" })).toBeNull();
  });
});
