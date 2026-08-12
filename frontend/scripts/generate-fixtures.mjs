/* eslint-disable no-console */
/**
 * Generate IntrospectResult fixtures at different scales for performance testing.
 *
 * Usage: node scripts/generate-fixtures.mjs
 * Output: test/fixtures/schemas/{small-20,medium-100,large-500,xlarge-1000}.json
 */

import { writeFileSync, mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const outDir = resolve(__dirname, "../test/fixtures/schemas");
mkdirSync(outDir, { recursive: true });

const COLUMN_TYPES = [
  "integer",
  "bigint",
  "text",
  "varchar(255)",
  "boolean",
  "timestamp",
  "date",
  "uuid",
  "numeric(10,2)",
  "jsonb",
  "serial",
  "bytea",
  "inet",
  "interval",
  "char(1)",
];

const TABLE_PREFIXES = [
  "app",
  "auth",
  "billing",
  "catalog",
  "config",
  "core",
  "data",
  "event",
  "geo",
  "iam",
  "integration",
  "log",
  "media",
  "meta",
  "notification",
  "order",
  "payment",
  "report",
  "schema",
  "session",
  "system",
  "task",
  "tenant",
  "user",
  "workflow",
];

function seededRandom(seed) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return s / 2147483647;
  };
}

function generateFixture(tableCount, seed = 42) {
  const rand = seededRandom(seed);
  const pick = (arr) => arr[Math.floor(rand() * arr.length)];

  const tables = [];
  const columns = [];
  const primaryKeys = [];
  const foreignKeys = [];
  const indexes = [];

  for (let i = 0; i < tableCount; i++) {
    const prefix = TABLE_PREFIXES[i % TABLE_PREFIXES.length];
    const suffix = i < TABLE_PREFIXES.length ? "" : `_${Math.floor(i / TABLE_PREFIXES.length)}`;
    const tableName = `${prefix}${suffix}`;
    const schema = "public";

    tables.push({ name: tableName, schema, rowCount: Math.floor(rand() * 100000) });

    const colCount = 4 + Math.floor(rand() * 12);
    const pkName = "id";

    columns.push({
      name: pkName,
      dataType: "serial",
      nullable: false,
      defaultValue: "nextval(...)",
      isPrimaryKey: true,
      tableName,
      schema,
    });
    primaryKeys.push({
      constraintName: `${tableName}_pkey`,
      columns: [pkName],
      tableName,
      schema,
    });
    indexes.push({
      name: `${tableName}_pkey`,
      columns: [pkName],
      unique: true,
      tableName,
      schema,
    });

    for (let c = 1; c < colCount; c++) {
      const colName = `col_${c}`;
      const dataType = pick(COLUMN_TYPES);
      const nullable = rand() > 0.3;
      columns.push({
        name: colName,
        dataType,
        nullable,
        defaultValue: nullable ? null : "''",
        isPrimaryKey: false,
        tableName,
        schema,
      });
    }

    if (i > 0 && rand() > 0.3) {
      const targetIdx = Math.floor(rand() * i);
      const targetTable = tables[targetIdx];
      const fkName = `fk_${tableName}_${targetTable.name}`;
      const fromColumn = `col_${1 + Math.floor(rand() * (colCount - 1))}`;

      foreignKeys.push({
        name: fkName,
        fromTable: tableName,
        fromColumn,
        toTable: targetTable.name,
        toColumn: "id",
        schema,
        toSchema: schema,
      });

      indexes.push({
        name: `idx_${tableName}_${fromColumn}`,
        columns: [fromColumn],
        unique: false,
        tableName,
        schema,
      });
    }
  }

  return {
    schemas: [{ name: "public", owner: "postgres" }],
    tables,
    columns,
    primaryKeys,
    indexes,
    foreignKeys,
    views: [],
    triggers: [],
  };
}

const SCALES = [
  { name: "small-20", count: 20 },
  { name: "medium-100", count: 100 },
  { name: "large-500", count: 500 },
  { name: "xlarge-1000", count: 1000 },
];

for (const { name, count } of SCALES) {
  const fixture = generateFixture(count);
  const path = resolve(outDir, `${name}.json`);
  writeFileSync(path, JSON.stringify(fixture, null, 2));
  console.log(
    `${name}: ${fixture.tables.length} tables, ${fixture.columns.length} columns, ${fixture.foreignKeys.length} FKs → ${path}`,
  );
}
