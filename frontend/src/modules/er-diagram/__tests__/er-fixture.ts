import type {
  IntrospectResult,
  SchemaColumnDto,
  PrimaryKeyDto,
  SchemaIndexDto,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

/**
 * Shared deterministic ER fixtures for er-diagram unit/perf tests.
 *
 * Mirrors the QA fixture shapes (A100 / B500 / C1000): each table gets an id
 * PK, 3–14 extra columns, and ~70% of non-first tables get an FK into an
 * earlier table. Deterministic (Park–Miller LCG, seed 42 default) so perf
 * budgets and parity assertions are stable across runs.
 */

export const COLUMN_TYPES = [
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

export const TABLE_PREFIXES = [
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

function seededRandom(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return s / 2147483647;
  };
}

export function generateErFixture(tableCount: number, seed = 42): IntrospectResult {
  const rand = seededRandom(seed);
  const pick = <T>(arr: T[]): T => arr[Math.floor(rand() * arr.length)];

  const tables: TableDto[] = [];
  const columns: SchemaColumnDto[] = [];
  const primaryKeys: PrimaryKeyDto[] = [];
  const foreignKeys: SchemaForeignKeyDto[] = [];
  const indexes: SchemaIndexDto[] = [];

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
    schemas: [{ name: "public" }],
    tables,
    columns,
    primaryKeys,
    indexes,
    foreignKeys,
    views: [],
    triggers: [],
  };
}
