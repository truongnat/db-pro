import type { CellValue, ColumnMeta } from "../types/data-grid.types";

/**
 * Maps a column's dataType string to the appropriate CellValue type.
 * This ensures filter values are correctly typed for the backend,
 * rather than always being sent as { type: "text" }.
 */
export function parseFilterValue(column: ColumnMeta, rawInput: string): CellValue {
  const dt = column.dataType.toLowerCase();

  // Boolean
  if (dt === "bool" || dt === "boolean") {
    const lower = rawInput.toLowerCase().trim();
    if (lower === "true" || lower === "1" || lower === "yes" || lower === "t") {
      return { type: "bool", value: true };
    }
    if (lower === "false" || lower === "0" || lower === "no" || lower === "f") {
      return { type: "bool", value: false };
    }
    // Fallback: treat as text if not a recognized boolean value
    return { type: "text", value: rawInput };
  }

  // Integer types
  if (
    dt === "int2" ||
    dt === "int4" ||
    dt === "int8" ||
    dt === "integer" ||
    dt === "smallint" ||
    dt === "bigint" ||
    dt === "serial" ||
    dt === "bigserial" ||
    dt === "smallserial"
  ) {
    const parsed = Number.parseInt(rawInput, 10);
    if (!Number.isNaN(parsed)) {
      return { type: "int64", value: parsed };
    }
    return { type: "text", value: rawInput };
  }

  // Float / numeric / decimal / real / double
  if (
    dt === "float4" ||
    dt === "float8" ||
    dt === "numeric" ||
    dt === "decimal" ||
    dt === "real" ||
    dt === "double precision" ||
    dt.startsWith("numeric(") ||
    dt.startsWith("decimal(")
  ) {
    const parsed = Number.parseFloat(rawInput);
    if (!Number.isNaN(parsed)) {
      return { type: "float64", value: parsed };
    }
    return { type: "text", value: rawInput };
  }

  // UUID
  if (dt === "uuid") {
    return { type: "uuid", value: rawInput.trim() };
  }

  // Date / time / timestamp
  if (
    dt === "date" ||
    dt === "time" ||
    dt === "timetz" ||
    dt === "timestamp" ||
    dt === "timestamptz" ||
    dt.startsWith("timestamp") ||
    dt.startsWith("time")
  ) {
    return { type: "datetime", value: rawInput.trim() };
  }

  // JSON / JSONB
  if (dt === "json" || dt === "jsonb") {
    try {
      const parsed = JSON.parse(rawInput);
      return { type: "json", value: parsed };
    } catch {
      return { type: "text", value: rawInput };
    }
  }

  // Default: text
  return { type: "text", value: rawInput };
}
