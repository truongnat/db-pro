import { describe, it, expect } from "vitest";
import type { IntrospectResult } from "@/modules/schema/types/schema.types";

import {
  buildColumnsByTable,
  buildPrimaryKeysByTable,
  buildFkColumnSet,
  buildTableNodes,
} from "../renderer/er-node-builder";
import { generateErFixture } from "./er-fixture";

/**
 * Benchmarks the REAL node-building pipeline (P3.3) — the same code path
 * `er-diagram.tsx` runs (pre-index once, then build all nodes via O(1) map
 * lookups). This is the automated performance regression test required by
 * P3.7: if anyone reintroduces a per-table `data.columns.filter(...)` scan,
 * the 500/1000-table budgets fail.
 */
function buildNodeDataIndex(data: IntrospectResult, schema: string) {
  const indexes = {
    columnsByTable: buildColumnsByTable(data.columns),
    primaryKeysByTable: buildPrimaryKeysByTable(data.primaryKeys),
    fkColumnSet: buildFkColumnSet(data.foreignKeys),
  };
  const tables = data.tables.filter((t) => t.schema === schema);
  return buildTableNodes(tables, indexes, { compact: false });
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
      const data = generateErFixture(count);
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
        console.warn(
          `WARNING: ${name} p95 (${p95.toFixed(1)}ms) exceeds 2x budget (${budgetMs}ms)`,
        );
      }
    });
  }
});
