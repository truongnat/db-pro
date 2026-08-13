import { describe, expect, it } from "vitest";

import {
  getBoundedNeighborhood,
  largeSchemaReducer,
  initialLargeSchemaState,
  shouldEnterLargeSchemaFlow,
  NEIGHBORHOOD_NODE_CAP,
  type LargeSchemaState,
} from "../utils/large-schema";
import { buildAdjacencyMap } from "../utils/neighborhood";

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
  return [
    makeFk("a", "b"),
    makeFk("b", "c"),
    makeFk("c", "d"),
    makeFk("d", "e"),
  ];
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
    const fks = [
      makeFk("hub", "zebra"),
      makeFk("hub", "alpha"),
      makeFk("hub", "middle"),
    ];
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

  it("SELECT_TABLE transitions to neighborhood with seed and focus", () => {
    const state = largeSchemaReducer(initialLargeSchemaState, {
      type: "SELECT_TABLE",
      tableKey: "public.users",
    });

    expect(state.phase).toBe("neighborhood");
    expect(state.seedTable).toBe("public.users");
    expect(state.focusedNodeId).toBe("public.users");
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

  it("BACK_TO_NEIGHBORHOOD returns from overview preserving seed", () => {
    const overview: LargeSchemaState = {
      phase: "overview",
      seedTable: "public.users",
      focusedNodeId: null,
    };

    const state = largeSchemaReducer(overview, { type: "BACK_TO_NEIGHBORHOOD" });

    expect(state.phase).toBe("neighborhood");
    expect(state.seedTable).toBe("public.users");
    expect(state.focusedNodeId).toBe("public.users");
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
      focusedNodeId: "public.users",
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

  it("CLEAR_FOCUS resets focus to seed in neighborhood phase", () => {
    const neighborhood: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: "public.users",
      focusedNodeId: "public.orders",
    };

    const state = largeSchemaReducer(neighborhood, { type: "CLEAR_FOCUS" });

    expect(state.focusedNodeId).toBe("public.users");
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
