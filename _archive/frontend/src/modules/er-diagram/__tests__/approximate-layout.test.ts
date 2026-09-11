import { describe, expect, it } from "vitest";

import { computeApproximateOverviewLayout } from "../utils/approximate-layout";
import type { ErGraphModel, TableId } from "../renderer/types";

/**
 * P1-1 (review F-REV-1) — the overview must paint in the first frame instead of
 * waiting 8–122 s for cold dagre. `computeApproximateOverviewLayout` is the
 * fast deterministic initial paint. These tests pin: full coverage, hub-first
 * ordering, determinism, and the sub-1 s budget at the xlarge fixture scale.
 */

function buildModel(n: number, hubAtStart = false, seed = 42): ErGraphModel {
  // Deterministic pseudo-random generator (mulberry32) so fixture shapes are
  // stable across runs.
  let s = seed >>> 0;
  const rand = () => {
    s |= 0;
    s = (s + 0x6d2b79f5) | 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };

  const tables = Array.from({ length: n }, (_, i) => ({
    id: `t${i}` as TableId,
    label: `t${i}`,
    schema: "public",
    columnCount: 4,
    fkCount: hubAtStart ? (i === 0 ? Math.floor(n / 2) : 1) : Math.floor(rand() * 5),
  }));

  // Ring topology: every table connects to its next + a few random extra edges.
  const relations = [];
  for (let i = 0; i < n; i++) {
    relations.push({
      id: `r${i}`,
      source: `t${i}` as TableId,
      target: `t${(i + 1) % n}` as TableId,
      name: `fk${i}`,
    });
    if (hubAtStart && i === 0) {
      for (let k = 1; k < n; k += 7) {
        relations.push({
          id: `rh${k}`,
          source: "t0" as TableId,
          target: `t${k}` as TableId,
          name: `hub${k}`,
        });
      }
    }
  }

  const adjacency = new Map<TableId, Set<TableId>>();
  for (const rel of relations) {
    let from = adjacency.get(rel.source);
    if (!from) adjacency.set(rel.source, (from = new Set()));
    from.add(rel.target);
    let to = adjacency.get(rel.target);
    if (!to) adjacency.set(rel.target, (to = new Set()));
    to.add(rel.source);
  }

  return {
    tables,
    relations,
    adjacency,
    stats: { tables: n, relations: relations.length, columns: n * 4 },
  };
}

describe("computeApproximateOverviewLayout (P1-1)", () => {
  it("covers every table (no dropped nodes)", () => {
    const model = buildModel(500);
    const positions = computeApproximateOverviewLayout(model);
    expect(positions.size).toBe(500);
    for (const table of model.tables) {
      expect(positions.has(table.id)).toBe(true);
    }
  });

  it("is deterministic — same graph, same positions", () => {
    const model = buildModel(200, true);
    const a = computeApproximateOverviewLayout(model);
    const b = computeApproximateOverviewLayout(model);
    expect(a).toEqual(b);
  });

  it("is hub-first — high-degree nodes get the first slots", () => {
    const model = buildModel(300, true);
    const positions = computeApproximateOverviewLayout(model);
    // The hub (t0) must be placed first, i.e. at angle 0 → x=radius, y=0.
    const hub = positions.get("t0");
    expect(hub).toBeDefined();
    expect(hub!.y).toBeCloseTo(0, 5);
    expect(hub!.x).toBeGreaterThan(0);
  });

  it("is fast — under 100 ms at 1000 tables (budget: sub-1 s TTD)", () => {
    const model = buildModel(1000);
    const t0 = performance.now();
    const positions = computeApproximateOverviewLayout(model);
    const elapsed = performance.now() - t0;
    expect(positions.size).toBe(1000);
    // Generous bound: the real budget is a sub-1 s time-to-diagram including
    // canvas mount; this pure O(N log N) computation is typically < 5 ms.
    expect(elapsed).toBeLessThan(100);
  });

  it("is finite and bounded — no NaN positions at any scale", () => {
    for (const n of [1, 2, 10, 100, 1000]) {
      const positions = computeApproximateOverviewLayout(buildModel(n));
      for (const [id, pos] of positions) {
        expect(Number.isFinite(pos.x)).toBe(true);
        expect(Number.isFinite(pos.y)).toBe(true);
        expect(id.length).toBeGreaterThan(0);
      }
    }
  });
});
