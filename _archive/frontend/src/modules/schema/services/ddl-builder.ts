import { getSqlDialect, type SqlDialect } from "@/modules/query/sql/dialect";

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
  | "dropIndex"
  | "enableTrigger"
  | "disableTrigger";

export function buildCreateTable(
  schema: string,
  table: string,
  columns: ColumnDef[],
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  const qualified = dialect.qualify(schema, table);
  const pkCols = columns.filter((c) => c.isPk);
  const lines = columns.map((col) => {
    let line = `    ${dialect.quoteIdentifier(col.name)} ${col.dataType}`;
    if (!col.nullable) line += " NOT NULL";
    if (col.defaultValue.trim()) line += ` DEFAULT ${col.defaultValue}`;
    return line;
  });

  if (pkCols.length > 0) {
    lines.push(
      `    PRIMARY KEY (${pkCols.map((c) => dialect.quoteIdentifier(c.name)).join(", ")})`,
    );
  }

  return `CREATE TABLE ${qualified} (\n${lines.join(",\n")}\n);`;
}

export function buildAddColumn(
  schema: string,
  table: string,
  column: ColumnDef,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  let line = `ALTER TABLE ${dialect.qualify(schema, table)} ADD COLUMN ${dialect.quoteIdentifier(column.name)} ${column.dataType}`;
  if (!column.nullable) line += " NOT NULL";
  if (column.defaultValue.trim()) line += ` DEFAULT ${column.defaultValue}`;
  return `${line};`;
}

export function buildDropColumn(
  schema: string,
  table: string,
  columnName: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  return `ALTER TABLE ${dialect.qualify(schema, table)} DROP COLUMN ${dialect.quoteIdentifier(columnName)};`;
}

export function buildRenameTable(
  schema: string,
  table: string,
  newName: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  return `ALTER TABLE ${dialect.qualify(schema, table)} RENAME TO ${dialect.quoteIdentifier(newName)};`;
}

export function buildDropTable(
  schema: string,
  table: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  return `DROP TABLE ${dialect.qualify(schema, table)};`;
}

export function buildCreateView(
  schema: string,
  name: string,
  selectSql: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  return `CREATE VIEW ${dialect.qualify(schema, name)} AS\n${selectSql};`;
}

export function buildDropView(
  schema: string,
  name: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  return `DROP VIEW ${dialect.qualify(schema, name)};`;
}

export function buildCreateIndex(
  schema: string,
  table: string,
  indexName: string,
  columns: string[],
  unique: boolean,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  const qualified = dialect.qualify(schema, table);
  const cols = columns.map(dialect.quoteIdentifier).join(", ");
  const u = unique ? "UNIQUE " : "";
  return `CREATE ${u}INDEX ${dialect.quoteIdentifier(indexName)} ON ${qualified} (${cols});`;
}

export function buildDropIndex(
  schema: string,
  indexName: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  return `DROP INDEX ${dialect.qualify(schema, indexName)};`;
}

export function buildCreateTrigger(
  schema: string,
  table: string,
  triggerName: string,
  timing: string,
  event: string,
  body: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  const qualified = dialect.qualify(schema, table);
  const name = dialect.quoteIdentifier(triggerName);
  return `CREATE TRIGGER ${name}\n  ${timing} ${event} ON ${qualified}\n  ${body}`;
}

export function buildDropTrigger(
  schema: string,
  table: string,
  triggerName: string,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  const qualified = dialect.qualify(schema, table);
  const name = dialect.quoteIdentifier(triggerName);
  return `DROP TRIGGER ${name} ON ${qualified};`;
}

export function buildSetTriggerEnabled(
  schema: string,
  table: string,
  triggerName: string,
  enabled: boolean,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  // SQLite does not support ALTER TABLE … ENABLE/DISABLE TRIGGER.
  if (dialect.driver === "sqlite") return "";
  const qualified = dialect.qualify(schema, table);
  const name = dialect.quoteIdentifier(triggerName);
  const action = enabled ? "ENABLE" : "DISABLE";
  return `ALTER TABLE ${qualified} ${action} TRIGGER ${name};`;
}

export function generateDdlPreview(
  operation: DdlOperation,
  schema: string,
  table: string,
  columns: ColumnDef[],
  extra: Record<string, string>,
  dialect: SqlDialect = getSqlDialect("postgres"),
): string {
  switch (operation) {
    case "createTable":
      return buildCreateTable(schema, table, columns, dialect);
    case "addColumn":
      return columns.length > 0 ? buildAddColumn(schema, table, columns[0], dialect) : "";
    case "dropColumn":
      return extra.columnName ? buildDropColumn(schema, table, extra.columnName, dialect) : "";
    case "renameTable":
      return extra.newName ? buildRenameTable(schema, table, extra.newName, dialect) : "";
    case "dropTable":
      return buildDropTable(schema, table, dialect);
    case "createView":
      return extra.selectSql ? buildCreateView(schema, table, extra.selectSql, dialect) : "";
    case "dropView":
      return buildDropView(schema, table, dialect);
    case "createIndex":
      return extra.indexColumns
        ? buildCreateIndex(
            schema,
            table,
            extra.indexName ?? "idx_new",
            extra.indexColumns
              .split(",")
              .map((s) => s.trim())
              .filter(Boolean),
            extra.unique === "true",
            dialect,
          )
        : "";
    case "dropIndex":
      return extra.indexName ? buildDropIndex(schema, extra.indexName, dialect) : "";
    case "enableTrigger":
      return extra.triggerName
        ? buildSetTriggerEnabled(schema, table, extra.triggerName, true, dialect)
        : "";
    case "disableTrigger":
      return extra.triggerName
        ? buildSetTriggerEnabled(schema, table, extra.triggerName, false, dialect)
        : "";
    default:
      return "";
  }
}
