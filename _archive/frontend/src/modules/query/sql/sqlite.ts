import type { SelectOptions, SqlDialect } from "./dialect";

function quoteIdentifier(name: string): string {
  const escaped = name.replace(/"/g, '""');
  return `"${escaped}"`;
}

function qualify(schema: string | null, object: string): string {
  if (!schema) return quoteIdentifier(object);
  return `${quoteIdentifier(schema)}.${quoteIdentifier(object)}`;
}

function generateSelect({ schema, table, limit }: SelectOptions): string {
  const limitClause = limit != null && limit > 0 ? ` LIMIT ${limit}` : "";
  return `SELECT * FROM ${qualify(schema, table)}${limitClause};`;
}

export const sqliteDialect: SqlDialect = {
  driver: "sqlite",
  formatterLanguage: "sqlite",
  quoteIdentifier,
  qualify,
  generateSelect,
};
