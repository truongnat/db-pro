import { describe, expect, it } from "vitest";

import {
  getBoundedNeighborhood,
  largeSchemaReducer,
  initialLargeSchemaState,
  shouldEnterLargeSchemaFlow,
  deriveNeighborhoodVisibleSet,
  NEIGHBORHOOD_NODE_CAP,
  type LargeSchemaState,
} from "../utils/large-schema";
import { buildAdjacencyMap } from "../utils/neighborhood";
import { generateErFixture } from "./er-fixture";
import { groupForeignKeys } from "../utils/edge-builder";
import { buildTableNodes, buildErNodeIndexes } from "../renderer/er-node-builder";

/* ── Fixtures ─────────────────────────────────────────────────────────────── */

function makeFk(from: string, to: string, schema = "public") {
  return {
    name: `fk_${from}_${to}`,
    fromTable: from,
    fromColumns: ["id"],
    toTable: to,
    toColumns: ["id"],
    schema,
    toSchema: schema,
  };
}

/** Linear chain: A → B → C → D → E */
function chainFks() {
  return [makeFk("a", "b"), makeFk("b", "c"), makeFk("c", "d"), makeFk("d", "e")];
}

/** Star topology: center connected to n spokes */
function starFks(spokeCount: number) {
  const fks = [];
  for (let i = 0; i < spokeCount; i++) {
    fks.push(makeFk("center", `spoke_${i}`));
  }
  return fks;
}

/* ── getBoundedNeighborhood ───────────────────────────────────────────────── */

describe("getBoundedNeighborhood", () => {
  it("returns seed only when hops=0", () => {
    const adj = buildAdjacencyMap(chainFks());
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.a", 0);

    expect(nodes.has("public.a")).toBe(true);
    expect(nodes.size).toBe(1);
    expect(truncated).toBe(false);
  });

  it("returns immediate neighbors at hops=1", () => {
    const adj = buildAdjacencyMap(chainFks());
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.b", 1);

    expect(nodes).toEqual(new Set(["public.a", "public.b", "public.c"]));
    expect(truncated).toBe(false);
  });

  it("expands to full chain within hop radius", () => {
    const adj = buildAdjacencyMap(chainFks());
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.a", 10);

    expect(nodes.size).toBe(5);
    expect(truncated).toBe(false);
  });

  it("truncates when node cap is reached before hop exhaustion", () => {
    // Star with 50 spokes, cap at 10 nodes
    const adj = buildAdjacencyMap(starFks(50));
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.center", 3, 10);

    expect(nodes.size).toBe(10);
    expect(truncated).toBe(true);
    expect(nodes.has("public.center")).toBe(true);
  });

  it("returns empty set when maxNodes=0", () => {
    const adj = buildAdjacencyMap(chainFks());
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.a", 5, 0);

    expect(nodes.size).toBe(0);
    expect(truncated).toBe(true);
  });

  it("handles missing seed gracefully", () => {
    const adj = buildAdjacencyMap(chainFks());
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.nonexistent", 5);

    expect(nodes.has("public.nonexistent")).toBe(true);
    expect(nodes.size).toBe(1);
    expect(truncated).toBe(false);
  });

  it("is deterministic with alphabetical tiebreaking", () => {
    // Hub connected to many tables — order of visit should be stable
    const fks = [makeFk("hub", "zebra"), makeFk("hub", "alpha"), makeFk("hub", "middle")];
    const adj = buildAdjacencyMap(fks);

    const result1 = getBoundedNeighborhood(adj, "public.hub", 2, 3);
    const result2 = getBoundedNeighborhood(adj, "public.hub", 2, 3);

    // Same nodes selected when truncated
    expect([...result1.nodes].sort()).toEqual([...result2.nodes].sort());
  });

  it("uses default NEIGHBORHOOD_NODE_CAP when maxNodes not specified", () => {
    expect(NEIGHBORHOOD_NODE_CAP).toBe(100);

    const adj = buildAdjacencyMap(starFks(200));
    const { nodes, truncated } = getBoundedNeighborhood(adj, "public.center", 1);

    expect(nodes.size).toBe(100);
    expect(truncated).toBe(true);
  });
});

/* ── largeSchemaReducer ───────────────────────────────────────────────────── */

describe("largeSchemaReducer", () => {
  it("starts in search phase with null selections", () => {
    expect(initialLargeSchemaState).toEqual({
      phase: "search",
      seedTable: null,
      focusedNodeId: null,
    });
  });

  it("SELECT_TABLE transitions to neighborhood with seed, focus stays null", () => {
    const state = largeSchemaReducer(initialLargeSchemaState, {
      type: "SELECT_TABLE",
      tableKey: "public.users",
    });

    expect(state.phase).toBe("neighborhood");
    expect(state.seedTable).toBe("public.users");
    expect(state.focusedNodeId).toBeNull();
  });

  it("SHOW_ALL transitions from neighborhood to overview", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: "public.orders",
    };

    const state = largeSchemaReducer(neighborhood, { type: "SHOW_ALL" });

    expect(state.phase).toBe("overview");
    expect(state.seedTable).toBe("public.users");
  });

  it("SHOW_ALL is ignored if not in neighborhood phase", () => {
    const state = largeSchemaReducer(initialLargeSchemaState, { type: "SHOW_ALL" });
    expect(state.phase).toBe("search");
  });

  it("BACK_TO_NEIGHBORHOOD returns from overview preserving seed, focus null", () => {
    const overview: LargeSchemaState = {
      phase: "overview",
      seedTable: "public.users",
      focusedNodeId: null,
    };

    const state = largeSchemaReducer(overview, { type: "BACK_TO_NEIGHBORHOOD" });

    expect(state.phase).toBe("neighborhood");
    expect(state.seedTable).toBe("public.users");
    expect(state.focusedNodeId).toBeNull();
  });

  it("BACK_TO_NEIGHBORHOOD is ignored if not in overview phase", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: null,
    };

    const state = largeSchemaReducer(neighborhood, { type: "BACK_TO_NEIGHBORHOOD" });
    expect(state.phase).toBe("neighborhood");
  });

  it("BACK_TO_SEARCH resets to initial state", () => {
    const overview: LargeSchemaState = {
      phase: "overview",
      seedTable: "public.users",
      focusedNodeId: "public.orders",
    };

    const state = largeSchemaReducer(overview, { type: "BACK_TO_SEARCH" });
    expect(state).toEqual(initialLargeSchemaState);
  });

  it("FOCUS_NODE updates focusedNodeId in neighborhood phase", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: null,
    };

    const state = largeSchemaReducer(neighborhood, {
      type: "FOCUS_NODE",
      nodeKey: "public.orders",
    });

    expect(state.focusedNodeId).toBe("public.orders");
    expect(state.seedTable).toBe("public.users");
  });

  it("FOCUS_NODE is ignored outside neighborhood phase", () => {
    const state = largeSchemaReducer(initialLargeSchemaState, {
      type: "FOCUS_NODE",
      nodeKey: "public.orders",
    });

    expect(state.phase).toBe("search");
    expect(state.focusedNodeId).toBeNull();
  });

  it("CLEAR_FOCUS resets focusedNodeId to null", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: "public.orders",
    };

    const state = largeSchemaReducer(neighborhood, { type: "CLEAR_FOCUS" });

    expect(state.focusedNodeId).toBeNull();
    expect(state.seedTable).toBe("public.users");
  });

  /* ── #42 focus contract tests ────────────────────────────────────────────── */

  it("#42: focus B → focus C → exactly 1 detail (focusedNodeId = C)", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: null,
    };

    // Focus node B
    const afterB = largeSchemaReducer(neighborhood, {
      type: "FOCUS_NODE",
      nodeKey: "public.orders",
    });
    expect(afterB.focusedNodeId).toBe("public.orders");

    // Focus node C — should replace B, not add to it
    const afterC = largeSchemaReducer(afterB, {
      type: "FOCUS_NODE",
      nodeKey: "public.products",
    });
    expect(afterC.focusedNodeId).toBe("public.products");
    expect(afterC.seedTable).toBe("public.users"); // seed preserved
  });

  it("#42: changing seed (SELECT_TABLE) clears previous focus", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: "public.orders",
    };

    // Select a different table → new seed, focus must reset to null
    const newState = largeSchemaReducer(neighborhood, {
      type: "SELECT_TABLE",
      tableKey: "public.products",
    });

    expect(newState.phase).toBe("neighborhood");
    expect(newState.seedTable).toBe("public.products");
    expect(newState.focusedNodeId).toBeNull(); // old focus cleared
  });

  it("#42: BACK_TO_SEARCH clears both seed and focus", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: "public.orders",
    };

    const state = largeSchemaReducer(neighborhood, { type: "BACK_TO_SEARCH" });

    expect(state).toEqual(initialLargeSchemaState);
    expect(state.seedTable).toBeNull();
    expect(state.focusedNodeId).toBeNull();
  });
});

/* ── shouldEnterLargeSchemaFlow ───────────────────────────────────────────── */

describe("shouldEnterLargeSchemaFlow", () => {
  it("returns true for table count > 200 regardless of tier", () => {
    expect(shouldEnterLargeSchemaFlow(201, "M")).toBe(true);
    expect(shouldEnterLargeSchemaFlow(300, "S")).toBe(true);
    expect(shouldEnterLargeSchemaFlow(1000, "XS")).toBe(true);
  });

  it("returns true for L/XL tier regardless of table count", () => {
    expect(shouldEnterLargeSchemaFlow(50, "L")).toBe(true);
    expect(shouldEnterLargeSchemaFlow(100, "XL")).toBe(true);
  });

  it("returns false for small schemas below threshold", () => {
    expect(shouldEnterLargeSchemaFlow(100, "S")).toBe(false);
    expect(shouldEnterLargeSchemaFlow(200, "M")).toBe(false);
    expect(shouldEnterLargeSchemaFlow(150, "M")).toBe(false);
  });

  it("boundary: exactly 200 tables with M tier does not enter", () => {
    expect(shouldEnterLargeSchemaFlow(200, "M")).toBe(false);
  });

  it("boundary: 201 tables enters even with low tier", () => {
    expect(shouldEnterLargeSchemaFlow(201, "XS")).toBe(true);
  });
});

/* ── deriveNeighborhoodVisibleSet ─────────────────────────────────────────── */

/** Helper: build a knownTableKeys set from FK fixtures + extra isolated tables. */
function knownKeys(fks: ReturnType<typeof makeFk>[], extra: string[] = []): Set<string> {
  const keys = new Set<string>(extra);
  for (const fk of fks) {
    keys.add(`${fk.schema}.${fk.fromTable}`);
    keys.add(`${fk.toSchema}.${fk.toTable}`);
  }
  return keys;
}

describe("deriveNeighborhoodVisibleSet", () => {
  /* 1. search phase → [] */
  it("search phase returns empty visible set", () => {
    const adj = buildAdjacencyMap(chainFks());
    const result = deriveNeighborhoodVisibleSet(
      initialLargeSchemaState, // phase=search, seedTable=null
      adj,
      2,
      knownKeys(chainFks()),
    );

    expect(result.tableIds).toEqual([]);
    expect(result.truncated).toBe(false);
  });

  /* 2. neighborhood + valid seed */
  it("neighborhood phase returns bounded visible set with valid seed", () => {
    const fks = chainFks();
    const adj = buildAdjacencyMap(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.a",
      focusedNodeId: "public.a",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 10, knownKeys(fks));

    expect(result.tableIds).toContain("public.a");
    expect(result.tableIds.length).toBe(5); // full chain reachable
    expect(result.truncated).toBe(false);
  });

  /* 3. seed with zero relationships → [seed] */
  it("seed with zero relationships returns exactly [seed]", () => {
    const fks = chainFks();
    const adj = buildAdjacencyMap(fks);
    // "lonely" is a known table but has no FK edges
    const keys = knownKeys(fks, ["public.lonely"]);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.lonely",
      focusedNodeId: "public.lonely",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, keys);

    expect(result.tableIds).toEqual(["public.lonely"]);
    expect(result.truncated).toBe(false);
  });

  /* 4. missing seed */
  it("missing seed returns empty set", () => {
    const fks = chainFks();
    const adj = buildAdjacencyMap(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.nonexistent",
      focusedNodeId: "public.nonexistent",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, knownKeys(fks));

    expect(result.tableIds).toEqual([]);
    expect(result.truncated).toBe(false);
  });

  it("null seed returns empty set", () => {
    const adj = buildAdjacencyMap(chainFks());
    const state: LargeSchemaState = { phase: "neighborhood", seedTable: null, focusedNodeId: null };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, knownKeys(chainFks()));

    expect(result.tableIds).toEqual([]);
  });

  /* 5. cap boundary: exactly 100 reachable */
  it("exactly 100 reachable tables returns all 100, not truncated", () => {
    const fks = starFks(99); // center + 99 spokes = 100
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.center",
      focusedNodeId: "public.center",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 2, keys);

    expect(result.tableIds.length).toBe(100);
    expect(result.truncated).toBe(false);
  });

  /* 5b. cap boundary: > 100 reachable */
  it("> 100 reachable tables truncates to cap", () => {
    const fks = starFks(200); // center + 200 spokes = 201
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.center",
      focusedNodeId: "public.center",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 2, keys);

    expect(result.tableIds.length).toBe(100);
    expect(result.truncated).toBe(true);
  });

  /* 6. dense hub → <=100 */
  it("dense hub returns <= 100 tables", () => {
    const fks = starFks(500); // center + 500 spokes
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.center",
      focusedNodeId: "public.center",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, keys);

    expect(result.tableIds.length).toBeLessThanOrEqual(100);
    expect(result.truncated).toBe(true);
  });

  /* 7. seed survives truncation */
  it("seed survives truncation", () => {
    const fks = starFks(200);
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.center",
      focusedNodeId: "public.center",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 2, keys);

    expect(result.tableIds).toContain("public.center");
    expect(result.truncated).toBe(true);
  });

  /* 8. deterministic repeated runs */
  it("repeated runs produce identical ordered result", () => {
    const fks = starFks(200);
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.center",
      focusedNodeId: "public.center",
    };

    const a = deriveNeighborhoodVisibleSet(state, adj, 2, keys);
    const b = deriveNeighborhoodVisibleSet(state, adj, 2, keys);
    const c = deriveNeighborhoodVisibleSet(state, adj, 2, keys);

    expect(a.tableIds).toEqual(b.tableIds);
    expect(b.tableIds).toEqual(c.tableIds);
    expect(a.truncated).toBe(b.truncated);
    expect(b.truncated).toBe(c.truncated);
  });

  /* 9. 500-table fixture → <=100 */
  it("500-table fixture produces <= 100 visible tables", () => {
    const data = generateErFixture(500, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const keys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    // Pick the highest-degree hub as seed
    let seed = "";
    let maxDeg = -1;
    for (const [k, neighbors] of adj) {
      if (neighbors.size > maxDeg) {
        maxDeg = neighbors.size;
        seed = k;
      }
    }
    const state: LargeSchemaState = { phase: "neighborhood", seedTable: seed, focusedNodeId: seed };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, keys);

    expect(result.tableIds.length).toBeLessThanOrEqual(100);
    expect(result.tableIds).toContain(seed);
  });

  /* 10. 1000-table fixture → <=100 */
  it("1000-table fixture produces <= 100 visible tables", () => {
    const data = generateErFixture(1000, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const keys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    let seed = "";
    let maxDeg = -1;
    for (const [k, neighbors] of adj) {
      if (neighbors.size > maxDeg) {
        maxDeg = neighbors.size;
        seed = k;
      }
    }
    const state: LargeSchemaState = { phase: "neighborhood", seedTable: seed, focusedNodeId: seed };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, keys);

    expect(result.tableIds.length).toBeLessThanOrEqual(100);
    expect(result.tableIds).toContain(seed);
  });

  /* 11. overview phase → not applicable, returns empty */
  it("overview phase returns empty set (not this helper's responsibility)", () => {
    const fks = starFks(200); // 201 tables total
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "overview",
      seedTable: "public.center",
      focusedNodeId: null,
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 3, keys);

    expect(result.tableIds).toEqual([]);
    expect(result.truncated).toBe(false);
  });

  /* 12. tableIds are sorted (deterministic ordering) */
  it("tableIds are sorted alphabetically", () => {
    const fks = [makeFk("hub", "zebra"), makeFk("hub", "alpha"), makeFk("hub", "middle")];
    const adj = buildAdjacencyMap(fks);
    const keys = knownKeys(fks);
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.hub",
      focusedNodeId: "public.hub",
    };

    const result = deriveNeighborhoodVisibleSet(state, adj, 2, keys);

    // Verify sorted
    const sorted = [...result.tableIds].sort();
    expect(result.tableIds).toEqual(sorted);
  });
});

/* ── Bounded materialization pipeline (#38) ─────────────────────────────── */

describe("bounded materialization pipeline (#38)", () => {
  /** Helper: pick the highest-degree hub from an adjacency map. */
  function pickHub(adj: Map<string, Set<string>>): string {
    let seed = "";
    let maxDeg = -1;
    for (const [k, neighbors] of adj) {
      if (neighbors.size > maxDeg) {
        maxDeg = neighbors.size;
        seed = k;
      }
    }
    return seed;
  }

  /**
   * Mirror the er-diagram.tsx integration: resolve tables from #37 visible
   * set preserving deterministic order via Map lookup.
   */
  function resolveTables(visibleTableIds: string[], allTables: { name: string; schema: string }[]) {
    const tableByKey = new Map(allTables.map((t) => [`${t.schema}.${t.name}`, t]));
    return visibleTableIds
      .map((id) => tableByKey.get(id))
      .filter((t): t is (typeof allTables)[number] => t !== undefined);
  }

  it("only builds nodes for visible tables in neighborhood phase", () => {
    const data = generateErFixture(500, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const allTableKeys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    const seed = pickHub(adj);

    const state: LargeSchemaState = { phase: "neighborhood", seedTable: seed, focusedNodeId: seed };
    const visible = deriveNeighborhoodVisibleSet(state, adj, 3, allTableKeys);

    // Resolve tables using #37 ordering (mirrors er-diagram.tsx)
    const tables = resolveTables(visible.tableIds, data.tables);
    const indexes = buildErNodeIndexes(data);
    const nodes = buildTableNodes(tables, indexes, { compact: false });

    // Node count must match visible set and respect hard cap
    expect(nodes.length).toBe(visible.tableIds.length);
    expect(nodes.length).toBeLessThanOrEqual(NEIGHBORHOOD_NODE_CAP);

    // All node IDs must be in the visible set
    const visibleSet = new Set(visible.tableIds);
    for (const node of nodes) {
      expect(visibleSet.has(node.id)).toBe(true);
    }
  });

  it("preserves deterministic node ordering from #37", () => {
    const data = generateErFixture(500, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const allTableKeys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    const seed = pickHub(adj);

    const state: LargeSchemaState = { phase: "neighborhood", seedTable: seed, focusedNodeId: seed };
    const visible = deriveNeighborhoodVisibleSet(state, adj, 3, allTableKeys);

    const tables = resolveTables(visible.tableIds, data.tables);
    const indexes = buildErNodeIndexes(data);
    const nodes = buildTableNodes(tables, indexes, { compact: false });

    // Node IDs must match #37's canonical ordering exactly
    expect(nodes.map((n) => n.id)).toEqual(visible.tableIds);
  });

  it("empty visible set in neighborhood → 0 nodes (fail-safe, no fallback)", () => {
    const data = generateErFixture(500, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const allTableKeys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));

    // Stale seed that doesn't exist in the schema
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.nonexistent_table",
      focusedNodeId: "public.nonexistent_table",
    };
    const visible = deriveNeighborhoodVisibleSet(state, adj, 3, allTableKeys);
    expect(visible.tableIds).toEqual([]);

    // Mirror er-diagram.tsx: neighborhood phase with empty visible set
    const tables = resolveTables(visible.tableIds, data.tables);
    expect(tables.length).toBe(0);

    const indexes = buildErNodeIndexes(data);
    const nodes = buildTableNodes(tables, indexes, { compact: false });
    expect(nodes.length).toBe(0);

    const edgeGroups = groupForeignKeys(data.foreignKeys, new Set(visible.tableIds));
    expect(edgeGroups.length).toBe(0);
  });

  it("drops edges with hidden endpoints", () => {
    const data = generateErFixture(500, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const allTableKeys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    const seed = pickHub(adj);

    const state: LargeSchemaState = { phase: "neighborhood", seedTable: seed, focusedNodeId: seed };
    const visible = deriveNeighborhoodVisibleSet(state, adj, 3, allTableKeys);
    const visibleSet = new Set(visible.tableIds);

    const edgeGroups = groupForeignKeys(data.foreignKeys, visibleSet);

    for (const group of edgeGroups) {
      const fromKey = `${group.fk.schema}.${group.fk.fromTable}`;
      const toKey = `${group.fk.toSchema}.${group.fk.toTable}`;
      expect(visibleSet.has(fromKey)).toBe(true);
      expect(visibleSet.has(toKey)).toBe(true);
    }
  });

  it("search phase produces zero nodes and edges", () => {
    const data = generateErFixture(500, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const allTableKeys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));

    const visible = deriveNeighborhoodVisibleSet(initialLargeSchemaState, adj, 3, allTableKeys);
    expect(visible.tableIds).toEqual([]);

    const tables = resolveTables(visible.tableIds, data.tables);
    expect(tables.length).toBe(0);
  });

  it("1000-table fixture: nodes <= 100, all edges bounded", () => {
    const data = generateErFixture(1000, 42);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const allTableKeys = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    const seed = pickHub(adj);

    const state: LargeSchemaState = { phase: "neighborhood", seedTable: seed, focusedNodeId: seed };
    const visible = deriveNeighborhoodVisibleSet(state, adj, 3, allTableKeys);

    expect(visible.tableIds.length).toBeLessThanOrEqual(NEIGHBORHOOD_NODE_CAP);

    const visibleSet = new Set(visible.tableIds);
    const edgeGroups = groupForeignKeys(data.foreignKeys, visibleSet);
    for (const group of edgeGroups) {
      const fromKey = `${group.fk.schema}.${group.fk.fromTable}`;
      const toKey = `${group.fk.toSchema}.${group.fk.toTable}`;
      expect(visibleSet.has(fromKey)).toBe(true);
      expect(visibleSet.has(toKey)).toBe(true);
    }
  });
});
