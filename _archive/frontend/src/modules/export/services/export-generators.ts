import type { ColumnMeta, Row } from "@/modules/query/types/query.types";

// ─── CSV ──────────────────────────────────────────────────────────

export interface CsvOptions {
  delimiter: "," | ";" | "\t" | "|";
  includeHeaders: boolean;
  nullRepresentation: string;
}

const DEFAULT_CSV_OPTIONS: CsvOptions = {
  delimiter: ",",
  includeHeaders: true,
  nullRepresentation: "",
};

function csvEscape(value: string, delimiter: string): string {
  if (
    value.includes(delimiter) ||
    value.includes('"') ||
    value.includes("\n") ||
    value.includes("\r")
  ) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

function cellToString(
  cell: { type: string; value?: unknown } | undefined,
  nullRep: string,
): string {
  if (!cell || cell.type === "null") return nullRep;
  if (cell.type === "bytes") return "[binary]";
  if (cell.value == null) return nullRep;
  return String(cell.value);
}

export function generateCsv(
  columns: ColumnMeta[],
  rows: Row[],
  options: Partial<CsvOptions> = {},
): string {
  const opts = { ...DEFAULT_CSV_OPTIONS, ...options };
  const lines: string[] = [];

  if (opts.includeHeaders) {
    lines.push(columns.map((c) => csvEscape(c.name, opts.delimiter)).join(opts.delimiter));
  }

  for (const row of rows) {
    const fields = row.map((cell) =>
      csvEscape(cellToString(cell, opts.nullRepresentation), opts.delimiter),
    );
    lines.push(fields.join(opts.delimiter));
  }

  return lines.join("\n");
}

// ─── JSON ─────────────────────────────────────────────────────────

export interface JsonOptions {
  pretty: boolean;
  nullRepresentation: string;
}

const DEFAULT_JSON_OPTIONS: JsonOptions = {
  pretty: true,
  nullRepresentation: "",
};

function cellToJson(cell: { type: string; value?: unknown } | undefined, nullRep: string): unknown {
  if (!cell || cell.type === "null") return nullRep === "" ? null : nullRep;
  if (cell.type === "bytes") return "[binary]";
  return cell.value ?? null;
}

export function generateJson(
  columns: ColumnMeta[],
  rows: Row[],
  options: Partial<JsonOptions> = {},
): string {
  const opts = { ...DEFAULT_JSON_OPTIONS, ...options };
  const data = rows.map((row) => {
    const obj: Record<string, unknown> = {};
    for (const [i, col] of columns.entries()) {
      obj[col.name] = cellToJson(row[i], opts.nullRepresentation);
    }
    return obj;
  });

  return opts.pretty ? JSON.stringify(data, null, 2) : JSON.stringify(data);
}

// ─── SQL (INSERT statements) ──────────────────────────────────────

export interface SqlExportOptions {
  tableName: string;
  nullRepresentation: string;
}

function sqlLiteral(cell: { type: string; value?: unknown } | undefined, nullRep: string): string {
  if (!cell || cell.type === "null") return nullRep === "" ? "NULL" : `'${nullRep}'`;
  if (cell.value == null) return "NULL";
  switch (cell.type) {
    case "bool":
      return cell.value ? "TRUE" : "FALSE";
    case "int64":
    case "float64":
      return String(cell.value);
    case "json":
      return `'${JSON.stringify(cell.value).replace(/'/g, "''")}'`;
    default:
      return `'${String(cell.value).replace(/'/g, "''")}'`;
  }
}

export function generateSqlInserts(
  columns: ColumnMeta[],
  rows: Row[],
  options: SqlExportOptions,
): string {
  const colList = columns.map((c) => `"${c.name}"`).join(", ");
  const lines: string[] = [];

  for (const row of rows) {
    const values = row.map((cell) => sqlLiteral(cell, options.nullRepresentation)).join(", ");
    lines.push(`INSERT INTO "${options.tableName}" (${colList}) VALUES (${values});`);
  }

  return lines.join("\n");
}
