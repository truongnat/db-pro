import type { CellValue } from "../types/data-grid.types";

export type NormalizedColumnType = CellValue["type"] | "numeric" | "decimal";

const NON_EDITABLE_TYPES = new Set<string>(["bytes", "numeric", "decimal"]);

export function normalizeColumnType(dataType: string): NormalizedColumnType {
  const dt = dataType.toLowerCase();
  if (dt === "boolean" || dt === "bool") return "bool";
  if (dt === "uuid") return "uuid";
  if (dt === "json" || dt === "jsonb") return "json";
  if (dt.includes("int")) return "int64";
  if (dt.includes("numeric")) return "numeric";
  if (dt.includes("decimal")) return "decimal";
  if (dt.includes("float") || dt.includes("double") || dt.includes("real")) return "float64";
  if (dt.includes("timestamp") || dt.includes("date") || dt.includes("time")) return "datetime";
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
    default:
      return null;
  }
}
