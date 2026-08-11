import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { IntrospectResult } from "@/modules/schema/types/schema.types";

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixtureDir = resolve(__dirname, "../../../../test/fixtures/schemas");

function loadFixture(name: string): IntrospectResult {
  return JSON.parse(readFileSync(resolve(fixtureDir, `${name}.json`), "utf-8"));
}

function buildNodeDataIndex(data: IntrospectResult, schema: string) {
  const columnsByTable = new Map<string, IntrospectResult["columns"]>();
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
  { name: "small-20", budgetMs: 5 },
  { name: "medium-100", budgetMs: 20 },
  { name: "large-500", budgetMs: 50 },
  { name: "xlarge-1000", budgetMs: 100 },
];

describe("ER diagram node build performance", () => {
  for (const { name, budgetMs } of FIXTURES) {
    it(`${name}: builds node index under ${budgetMs}ms`, () => {
      const data = loadFixture(name);
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
