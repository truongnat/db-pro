export interface ColumnDef {
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue: string;
  isPk: boolean;
}

export type DdlOperation =
  | "createTable"
  | "addColumn"
  | "dropColumn"
  | "renameTable"
  | "dropTable"
  | "createView"
  | "dropView"
  | "createIndex"
  | "dropIndex";

function quote(name: string): string {
  const escaped = name.replace(/"/g, '""');
  return `"${escaped}"`;
}

function qualify(schema: string, table: string): string {
  return `${quote(schema)}.${quote(table)}`;
}

export function buildCreateTable(schema: string, table: string, columns: ColumnDef[]): string {
  const qualified = qualify(schema, table);
  const pkCols = columns.filter((c) => c.isPk);
  const lines = columns.map((col) => {
    let line = `    ${quote(col.name)} ${col.dataType}`;
    if (!col.nullable) line += " NOT NULL";
    if (col.defaultValue.trim()) line += ` DEFAULT ${col.defaultValue}`;
    return line;
  });

  if (pkCols.length > 0) {
    lines.push(`    PRIMARY KEY (${pkCols.map((c) => quote(c.name)).join(", ")})`);
  }

  return `CREATE TABLE ${qualified} (\n${lines.join(",\n")}\n);`;
}

export function buildAddColumn(schema: string, table: string, column: ColumnDef): string {
  let line = `ALTER TABLE ${qualify(schema, table)} ADD COLUMN ${quote(column.name)} ${column.dataType}`;
  if (!column.nullable) line += " NOT NULL";
  if (column.defaultValue.trim()) line += ` DEFAULT ${column.defaultValue}`;
  return `${line};`;
}

export function buildDropColumn(schema: string, table: string, columnName: string): string {
  return `ALTER TABLE ${qualify(schema, table)} DROP COLUMN ${quote(columnName)};`;
}

export function buildRenameTable(schema: string, table: string, newName: string): string {
  return `ALTER TABLE ${qualify(schema, table)} RENAME TO ${quote(newName)};`;
}

export function buildDropTable(schema: string, table: string): string {
  return `DROP TABLE ${qualify(schema, table)};`;
}

export function buildCreateView(schema: string, name: string, selectSql: string): string {
  return `CREATE VIEW ${qualify(schema, name)} AS\n${selectSql};`;
}

export function buildDropView(schema: string, name: string): string {
  return `DROP VIEW ${qualify(schema, name)};`;
}

export function buildCreateIndex(
  schema: string,
  table: string,
  indexName: string,
  columns: string[],
  unique: boolean,
): string {
  const qualified = qualify(schema, table);
  const cols = columns.map(quote).join(", ");
  const u = unique ? "UNIQUE " : "";
  return `CREATE ${u}INDEX ${quote(indexName)} ON ${qualified} (${cols});`;
}

export function buildDropIndex(schema: string, indexName: string): string {
  return `DROP INDEX ${qualify(schema, indexName)};`;
}

export function generateDdlPreview(
  operation: DdlOperation,
  schema: string,
  table: string,
  columns: ColumnDef[],
  extra: Record<string, string>,
): string {
  switch (operation) {
    case "createTable":
      return buildCreateTable(schema, table, columns);
    case "addColumn":
      return columns.length > 0 ? buildAddColumn(schema, table, columns[0]) : "";
    case "dropColumn":
      return extra.columnName ? buildDropColumn(schema, table, extra.columnName) : "";
    case "renameTable":
      return extra.newName ? buildRenameTable(schema, table, extra.newName) : "";
    case "dropTable":
      return buildDropTable(schema, table);
    case "createView":
      return extra.selectSql ? buildCreateView(schema, table, extra.selectSql) : "";
    case "dropView":
      return buildDropView(schema, table);
    case "createIndex":
      return extra.indexColumns
        ? buildCreateIndex(
            schema,
            table,
            extra.indexName ?? "idx_new",
            extra.indexColumns.split(",").map((s) => s.trim()).filter(Boolean),
            extra.unique === "true",
          )
        : "";
    case "dropIndex":
      return extra.indexName ? buildDropIndex(schema, extra.indexName) : "";
    default:
      return "";
  }
}
