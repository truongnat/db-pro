import { describe, expect, it } from "vitest";

import { computeLayoutHash } from "../utils/layout-hash";
import {
  buildLayoutInputFromModel,
  OVERVIEW_LAYOUT_PROFILE,
  REACT_FLOW_LAYOUT_PROFILE,
} from "../utils/layout-profile";
import type { ErGraphModel, TableId } from "../renderer/types";

/**
 * P1-2 (review F-REV-2) — the layout engine must use the target renderer's
 * geometry. The canvas overview paints compact 160×28 chips; React Flow paints
 * column-aware cards. These tests pin both profiles and prove the cache hash
 * separates them.
 */

function buildModel(): ErGraphModel {
  const tables = Array.from({ length: 3 }, (_, i) => ({
    id: `t${i}` as TableId,
    label: `t${i}`,
    schema: "public",
    columnCount: 12,
    fkCount: 1,
  }));
  const relations = [
    { id: "r0", source: "t0" as TableId, target: "t1" as TableId, name: "fk0" },
    { id: "r1", source: "t1" as TableId, target: "t2" as TableId, name: "fk1" },
  ];
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
    stats: { tables: 3, relations: 2, columns: 36 },
  };
}

describe("layout profiles (P1-2)", () => {
  it("overview profile is compact and column-independent", () => {
    // 12 columns, but the overview chip is still 160×28 — matches the canvas
    // node style. This is the contract fix: dagre no longer thinks a 12-column
    // table is 220×280 when the canvas paints 160×28.
    const input = buildLayoutInputFromModel(buildModel(), OVERVIEW_LAYOUT_PROFILE);
    expect(input.nodes).toHaveLength(3);
    for (const node of input.nodes) {
      expect(node.width).toBe(160);
      expect(node.height).toBe(28);
    }
  });

  it("react-flow profile keeps column-aware heights", () => {
    const input = buildLayoutInputFromModel(buildModel(), REACT_FLOW_LAYOUT_PROFILE);
    for (const node of input.nodes) {
      expect(node.width).toBe(220);
      // 12 columns: 32 header + 12*20 rows + 8 padding = 280
      expect(node.height).toBe(280);
    }
  });

  it("overview and react-flow layouts never share a cache hash", () => {
    const model = buildModel();
    const overview = buildLayoutInputFromModel(model, OVERVIEW_LAYOUT_PROFILE);
    const rf = buildLayoutInputFromModel(model, REACT_FLOW_LAYOUT_PROFILE);

    const overviewHash = computeLayoutHash(overview, { profile: OVERVIEW_LAYOUT_PROFILE.id });
    const rfHash = computeLayoutHash(rf, { profile: REACT_FLOW_LAYOUT_PROFILE.id });
    expect(overviewHash).not.toBe(rfHash);
  });

  it("same profile + same graph → same hash (cache stability)", () => {
    const model = buildModel();
    const a = buildLayoutInputFromModel(model, OVERVIEW_LAYOUT_PROFILE);
    const b = buildLayoutInputFromModel(model, OVERVIEW_LAYOUT_PROFILE);
    expect(computeLayoutHash(a, { profile: "overview" })).toBe(
      computeLayoutHash(b, { profile: "overview" }),
    );
  });
});
