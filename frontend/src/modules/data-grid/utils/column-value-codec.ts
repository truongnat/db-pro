import type { CellValue } from "../types/data-grid.types";

export type NormalizedColumnType = CellValue["type"] | "numeric" | "decimal" | "bigint";

const NON_EDITABLE_TYPES = new Set<string>(["bytes", "numeric", "decimal", "bigint"]);

export function normalizeColumnType(dataType: string): NormalizedColumnType {
  const dt = dataType.toLowerCase().replace(/\(.*\)/, "").trim();

  if (dt === "boolean" || dt === "bool") return "bool";
  if (dt === "uuid") return "uuid";
  if (dt === "json" || dt === "jsonb") return "json";
  if (dt === "bytea" || dt === "blob") return "bytes";

  if (dt === "bigint" || dt === "int8" || dt === "bigserial") return "bigint";
  if (
    dt === "smallint" || dt === "int2" || dt === "integer" || dt === "int4" ||
    dt === "int" || dt === "serial" || dt === "smallserial"
  ) return "int64";

  if (dt === "numeric" || dt === "decimal") return dt;

  if (dt === "real" || dt === "float4" || dt === "float8" || dt === "float" || dt === "double precision" || dt === "double") return "float64";

  if (
    dt === "timestamp" || dt === "timestamptz" || dt === "date" || dt === "time" ||
    dt === "timetz" || dt === "timestamp without time zone" || dt === "timestamp with time zone"
  ) return "datetime";

  if (dt === "interval") return "text";

  if (dt.includes("int")) return "int64";
  if (dt.includes("float") || dt.includes("double") || dt.includes("real")) return "float64";
  if (dt.includes("char") || dt.includes("text")) return "text";
  if (dt.includes("timestamp") || dt.includes("date") || dt.includes("time")) return "datetime";
  if (dt.includes("numeric") || dt.includes("decimal")) return "numeric";
  if (dt.includes("bytea") || dt.includes("blob")) return "bytes";

  return "text";
}

export function isCellTypeEditable(cellType: NormalizedColumnType): boolean {
  return !NON_EDITABLE_TYPES.has(cellType);
}

export function getUnsupportedEditReason(cellType: NormalizedColumnType): string | null {
  switch (cellType) {
    case "bytes":
      return "Binary editing is not supported yet";
    case "numeric":
    case "decimal":
      return "High-precision decimal editing is not supported yet";
    case "bigint":
      return "Large integer editing is not supported yet";
    default:
      return null;
  }
}
