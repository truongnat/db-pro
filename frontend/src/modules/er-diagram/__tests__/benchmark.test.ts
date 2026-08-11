import { describe, it, expect } from "vitest";
import type {
  IntrospectResult,
  SchemaColumnDto,
  PrimaryKeyDto,
  SchemaIndexDto,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

const COLUMN_TYPES = [
  "integer", "bigint", "text", "varchar(255)", "boolean",
  "timestamp", "date", "uuid", "numeric(10,2)", "jsonb",
  "serial", "bytea", "inet", "interval", "char(1)",
];

const TABLE_PREFIXES = [
  "app", "auth", "billing", "catalog", "config", "core", "data",
  "event", "geo", "iam", "integration", "log", "media", "meta",
  "notification", "order", "payment", "report", "schema", "session",
  "system", "task", "tenant", "user", "workflow",
];

function seededRandom(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return s / 2147483647;
  };
}

function generateFixture(tableCount: number, seed = 42): IntrospectResult {
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

function buildNodeDataIndex(data: IntrospectResult, schema: string) {
  const columnsByTable = new Map<string, SchemaColumnDto[]>();
  for (const col of data.columns) {
    const key = `${col.schema}.${col.tableName}`;
    const list = columnsByTable.get(key);
    if (list) list.push(col);
    else columnsByTable.set(key, [col]);
  }

  const primaryKeysByTable = new Map<string, Set<string>>();
  for (const pk of data.primaryKeys) {
    const key = `${pk.schema}.${pk.tableName}`;
    const existing = primaryKeysByTable.get(key);
    if (existing) for (const c of pk.columns) existing.add(c);
    else primaryKeysByTable.set(key, new Set(pk.columns));
  }

  const fkColumnSet = new Set<string>();
  for (const fk of data.foreignKeys) {
    fkColumnSet.add(`${fk.schema}.${fk.fromTable}:${fk.fromColumn}`);
  }

  const tables = data.tables.filter((t) => t.schema === schema);
  const nodes = tables.map((table) => {
    const tableKey = `${table.schema}.${table.name}`;
    const cols = columnsByTable.get(tableKey) ?? [];
    const pkCols = primaryKeysByTable.get(tableKey);
    return {
      id: tableKey,
      columns: cols.map((c) => ({
        name: c.name,
        isPrimaryKey: pkCols?.has(c.name) ?? false,
        isForeignKey: fkColumnSet.has(`${tableKey}:${c.name}`),
      })),
    };
  });

  return nodes;
}

const FIXTURES = [
  { name: "small-20", count: 20, budgetMs: 5 },
  { name: "medium-100", count: 100, budgetMs: 20 },
  { name: "large-500", count: 500, budgetMs: 50 },
  { name: "xlarge-1000", count: 1000, budgetMs: 100 },
];

describe("ER diagram node build performance", () => {
  for (const { name, count, budgetMs } of FIXTURES) {
    it(`${name}: builds node index under ${budgetMs}ms`, () => {
      const data = generateFixture(count);
      const iterations = 10;
      const times: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const start = performance.now();
        const nodes = buildNodeDataIndex(data, "public");
        const elapsed = performance.now() - start;
        times.push(elapsed);
        expect(nodes.length).toBe(data.tables.length);
      }

      const avg = times.reduce((a, b) => a + b, 0) / times.length;
      const p95 = times.sort((a, b) => a - b)[Math.floor(times.length * 0.95)];

      expect(avg).toBeLessThan(budgetMs);

      if (p95 > budgetMs * 2) {
        console.warn(`WARNING: ${name} p95 (${p95.toFixed(1)}ms) exceeds 2x budget (${budgetMs}ms)`);
      }
    });
  }
});
