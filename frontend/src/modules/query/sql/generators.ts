import type { SqlDialect } from "./dialect";

export interface SqlColumn {
  name: string;
  isPrimaryKey?: boolean;
  nullable?: boolean;
  defaultValue?: string | null;
}

export function generateSelectSQL(
  dialect: SqlDialect,
  schema: string | null,
  table: string,
  columns: SqlColumn[],
): string {
  const q = dialect.qualify(schema, table);
  const cols = columns.map((c) => `  ${dialect.quoteIdentifier(c.name)}`).join(",\n");
  return `SELECT\n${cols}\nFROM ${q};`;
}

export function generateInsertSQL(
  dialect: SqlDialect,
  schema: string | null,
  table: string,
  columns: SqlColumn[],
): string {
  const q = dialect.qualify(schema, table);
  const colNames = columns.map((c) => dialect.quoteIdentifier(c.name)).join(", ");
  const placeholders = columns.map((c) => {
    if (c.isPrimaryKey && c.defaultValue) return "DEFAULT";
    if (!c.nullable && !c.defaultValue) return `'<${c.name}>'`;
    if (c.defaultValue) return c.defaultValue;
    return "NULL";
  });
  return `INSERT INTO ${q} (${colNames})\nVALUES (${placeholders.join(", ")});`;
}

export function generateUpdateSQL(
  dialect: SqlDialect,
  schema: string | null,
  table: string,
  columns: SqlColumn[],
): string {
  const q = dialect.qualify(schema, table);
  const pkCols = columns.filter((c) => c.isPrimaryKey);
  const nonPkCols = columns.filter((c) => !c.isPrimaryKey);

  const setClauses = nonPkCols
    .map((c) => `  ${dialect.quoteIdentifier(c.name)} = <${c.name}>`)
    .join(",\n");
  const whereClauses = pkCols
    .map((c) => `${dialect.quoteIdentifier(c.name)} = <${c.name}>`)
    .join(" AND ");

  return `UPDATE ${q}\nSET\n${setClauses}\nWHERE ${whereClauses || "1 = 1 /* no PK */"};`;
}

export function generateDeleteSQL(
  dialect: SqlDialect,
  schema: string | null,
  table: string,
  columns: SqlColumn[],
): string {
  const q = dialect.qualify(schema, table);
  const pkCols = columns.filter((c) => c.isPrimaryKey);
  const whereClauses = pkCols
    .map((c) => `${dialect.quoteIdentifier(c.name)} = <${c.name}>`)
    .join(" AND ");
  return `DELETE FROM ${q}\nWHERE ${whereClauses || "1 = 1 /* no PK */"};`;
}
