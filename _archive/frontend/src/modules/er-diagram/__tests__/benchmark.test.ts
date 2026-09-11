import { describe, it, expect } from "vitest";
import type { IntrospectResult } from "@/modules/schema/types/schema.types";

import {
  buildColumnsByTable,
  buildPrimaryKeysByTable,
  buildFkColumnSet,
  buildTableNodes,
} from "../renderer/er-node-builder";
import {
  deriveNeighborhoodVisibleSet,
  initialLargeSchemaState,
  largeSchemaReducer,
  NEIGHBORHOOD_NODE_CAP,
} from "../utils/large-schema";
import { buildAdjacencyMap } from "../utils/neighborhood";
import { generateErFixture } from "./er-fixture";

/**
 * Benchmarks the REAL node-building pipeline (P3.3) — the same code path
 * `er-diagram.tsx` runs (pre-index once, then build all nodes via O(1) map
 * lookups). This remains the small/medium baseline and guards against
 * reintroducing per-table `data.columns.filter(...)` scans.
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

/**
 * Gate 4 QA-P1-13 strategy benchmark.
 *
 * Unlike the baseline above, this measures the production large-schema path:
 * explicit table selection -> deterministic bounded traversal -> materialize
 * only the visible React Flow neighborhood. It therefore proves the benchmark
 * covers the rendering strategy itself rather than only metadata indexing.
 */
function buildBoundedNeighborhoodNodes(data: IntrospectResult, schema: string) {
  const tables = data.tables.filter((table) => table.schema === schema);
  const knownTableKeys = new Set(tables.map((table) => `${table.schema}.${table.name}`));
  const seed = `${schema}.${tables[0].name}`;
  const state = largeSchemaReducer(initialLargeSchemaState, {
    type: "SELECT_TABLE",
    tableKey: seed,
  });
  const adjacency = buildAdjacencyMap(data.foreignKeys);
  const visible = deriveNeighborhoodVisibleSet(state, adjacency, 2, knownTableKeys);
  const tableByKey = new Map(tables.map((table) => [`${table.schema}.${table.name}`, table]));
  const visibleTables = visible.tableIds
    .map((id) => tableByKey.get(id))
    .filter((table): table is (typeof tables)[number] => table !== undefined);

  const indexes = {
    columnsByTable: buildColumnsByTable(data.columns),
    primaryKeysByTable: buildPrimaryKeysByTable(data.primaryKeys),
    fkColumnSet: buildFkColumnSet(data.foreignKeys),
  };

  return {
    seed,
    visible,
    nodes: buildTableNodes(visibleTables, indexes, { compact: true }),
  };
}

const LARGE_SCHEMA_STRATEGY_FIXTURES = [
  { name: "gate4-500", count: 500, budgetMs: 100 },
  { name: "gate4-1000", count: 1000, budgetMs: 150 },
];

describe("Gate 4 bounded rendering strategy performance", () => {
  for (const { name, count, budgetMs } of LARGE_SCHEMA_STRATEGY_FIXTURES) {
    it(`${name}: derives and materializes only the bounded neighborhood under ${budgetMs}ms`, () => {
      const data = generateErFixture(count);
      const iterations = 10;
      const times: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const start = performance.now();
        const result = buildBoundedNeighborhoodNodes(data, "public");
        times.push(performance.now() - start);

        expect(result.visible.tableIds.length).toBeLessThanOrEqual(NEIGHBORHOOD_NODE_CAP);
        expect(result.nodes.length).toBe(result.visible.tableIds.length);
        expect(result.visible.tableIds).toContain(result.seed);
      }

      const avg = times.reduce((sum, elapsed) => sum + elapsed, 0) / times.length;
      const p95 = times.sort((a, b) => a - b)[Math.floor(times.length * 0.95)];

      expect(avg).toBeLessThan(budgetMs);

      if (p95 > budgetMs * 2) {
        console.warn(
          `WARNING: ${name} bounded strategy p95 (${p95.toFixed(1)}ms) exceeds 2x budget (${budgetMs}ms)`,
        );
      }
    });
  }
});
