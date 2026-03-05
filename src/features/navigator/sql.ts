import type { DbEngine, NavigatorObject } from "@/types";

function quoteIdentifier(engine: DbEngine | undefined, identifier: string): string {
  if (engine === "mysql") {
    return `\`${identifier.replace(/`/g, "``")}\``;
  }

  return `"${identifier.replace(/"/g, "\"\"")}"`;
}

export function buildQualifiedObjectName(
  engine: DbEngine | undefined,
  schemaName: string,
  objectName: string,
): string {
  const schemaPart = quoteIdentifier(engine, schemaName);
  const objectPart = quoteIdentifier(engine, objectName);
  return `${schemaPart}.${objectPart}`;
}

export function buildSelectFromObjectSql(
  engine: DbEngine | undefined,
  schemaName: string,
  objectName: string,
  limit = 200,
): string {
  const qualifiedName = buildQualifiedObjectName(engine, schemaName, objectName);
  return `SELECT *\nFROM ${qualifiedName}\nLIMIT ${limit};`;
}

export function buildInsertTemplateSql(
  engine: DbEngine | undefined,
  schemaName: string,
  object: NavigatorObject,
): string {
  const qualifiedName = buildQualifiedObjectName(engine, schemaName, object.name);
  const columnNames =
    object.columns.length > 0
      ? object.columns.map((column) => quoteIdentifier(engine, column.name))
      : [quoteIdentifier(engine, "column_1")];
  const values =
    object.columns.length > 0
      ? object.columns.map((column) => `:${column.name}`)
      : [":column_1"];

  return `INSERT INTO ${qualifiedName} (${columnNames.join(", ")})\nVALUES (${values.join(", ")});`;
}

export function buildUpdateTemplateSql(
  engine: DbEngine | undefined,
  schemaName: string,
  object: NavigatorObject,
): string {
  const qualifiedName = buildQualifiedObjectName(engine, schemaName, object.name);
  const fallbackColumn = quoteIdentifier(engine, "column_1");
  const setClauses =
    object.columns.length > 0
      ? object.columns
          .slice(0, Math.min(object.columns.length, 4))
          .map(
            (column) =>
              `${quoteIdentifier(engine, column.name)} = :${column.name}`,
          )
      : [`${fallbackColumn} = :column_1`];
  const whereColumn =
    object.columns[0]?.name ?? object.columns[1]?.name ?? "id";

  return `UPDATE ${qualifiedName}\nSET ${setClauses.join(",\n    ")}\nWHERE ${quoteIdentifier(engine, whereColumn)} = :${whereColumn};`;
}

export function buildDdlTemplateSql(
  engine: DbEngine | undefined,
  schemaName: string,
  object: NavigatorObject,
): string {
  const qualifiedName = buildQualifiedObjectName(engine, schemaName, object.name);

  if (object.kind === "view") {
    return `CREATE VIEW ${qualifiedName} AS\nSELECT\n  -- add projected columns\n  1 AS sample_column;`;
  }

  const columnLines =
    object.columns.length > 0
      ? object.columns.map((column) => {
          const nullableClause = column.nullable ? "" : " NOT NULL";
          const dataType = column.dataType || "TEXT";
          return `  ${quoteIdentifier(engine, column.name)} ${dataType}${nullableClause}`;
        })
      : ["  id INTEGER PRIMARY KEY"];

  return `CREATE TABLE ${qualifiedName} (\n${columnLines.join(",\n")}\n);`;
}
