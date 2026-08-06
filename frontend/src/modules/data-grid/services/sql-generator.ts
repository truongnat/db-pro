import type { CellValue, ColumnMeta, Row } from "@/modules/query/types/query.types";

function quoteIdentifier(name: string): string {
  const escaped = name.replace(/"/g, '""');
  return `"${escaped}"`;
}

function qualify(schema: string, table: string): string {
  return `${quoteIdentifier(schema)}.${quoteIdentifier(table)}`;
}

function cellToSqlLiteral(cell: CellValue): string {
  switch (cell.type) {
    case "null":
      return "NULL";
    case "bool":
      return cell.value ? "TRUE" : "FALSE";
    case "int64":
    case "float64":
      return String(cell.value);
    case "text":
    case "uuid":
    case "datetime":
      return `'${cell.value.replace(/'/g, "''")}'`;
    case "bytes":
      return `'\\x${cell.value.map((b) => b.toString(16).padStart(2, "0")).join("")}'`;
    case "json":
      return `'${JSON.stringify(cell.value).replace(/'/g, "''")}'`;
    default:
      return "NULL";
  }
}

function rowToValues(row: Row): string[] {
  return row.map(cellToSqlLiteral);
}

export function generateInsertSQL(
  schema: string,
  table: string,
  columns: ColumnMeta[],
  rows: Row[],
): string {
  if (rows.length === 0) return "";

  const qualified = qualify(schema, table);
  const colNames = columns.map((c) => quoteIdentifier(c.name)).join(", ");

  const valueRows = rows.map((row) => {
    const values = rowToValues(row);
    return `  (${values.join(", ")})`;
  });

  return `INSERT INTO ${qualified} (${colNames})\nVALUES\n${valueRows.join(",\n")};`;
}

export function generateUpdateSQL(
  schema: string,
  table: string,
  columns: ColumnMeta[],
  row: Row,
  pkColumns: string[],
): string {
  const qualified = qualify(schema, table);
  const setClauses: string[] = [];

  for (let i = 0; i < columns.length; i++) {
    const col = columns[i];
    if (pkColumns.includes(col.name)) continue;
    setClauses.push(`  ${quoteIdentifier(col.name)} = ${cellToSqlLiteral(row[i])}`);
  }

  if (setClauses.length === 0) return "";

  const whereClauses: string[] = [];
  for (const pkCol of pkColumns) {
    const idx = columns.findIndex((c) => c.name === pkCol);
    if (idx >= 0) {
      whereClauses.push(`${quoteIdentifier(pkCol)} = ${cellToSqlLiteral(row[idx])}`);
    }
  }

  return `UPDATE ${qualified}\nSET\n${setClauses.join(",\n")}\nWHERE ${whereClauses.join(" AND ")};`;
}

export function generateDeleteSQL(
  schema: string,
  table: string,
  columns: ColumnMeta[],
  row: Row,
  pkColumns: string[],
): string {
  const qualified = qualify(schema, table);

  const whereClauses: string[] = [];
  for (const pkCol of pkColumns) {
    const idx = columns.findIndex((c) => c.name === pkCol);
    if (idx >= 0) {
      whereClauses.push(`${quoteIdentifier(pkCol)} = ${cellToSqlLiteral(row[idx])}`);
    }
  }

  if (whereClauses.length === 0) return "";

  return `DELETE FROM ${qualified}\nWHERE ${whereClauses.join(" AND ")};`;
}

export function generateSelectSQL(
  schema: string,
  table: string,
  columns: ColumnMeta[],
): string {
  const qualified = qualify(schema, table);
  const colNames = columns.map((c) => quoteIdentifier(c.name)).join(",\n  ");
  return `SELECT\n  ${colNames}\nFROM ${qualified};`;
}
