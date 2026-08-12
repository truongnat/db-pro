import { describe, expect, it } from "vitest";

import type { ErGraphModel, TableId } from "../renderer/types";
import { findTableMatches, resolveHighlightSet } from "../utils/overview-search";

/**
 * UX pivot — search FOCUSES, never filters. These pure helpers drive the
 * opass-style search→ring + click→neighborhood-highlight behavior of the
 * canvas overview; the full graph is always mounted, search only computes
 * which tables to ring/center and which set to highlight.
 */

function buildModel(): ErGraphModel {
  // chain: a-b-c-d (public.*), plus a disconnected pair x-y (sales.*)
  const tables: ErGraphModel["tables"] = [
    { id: "public.a", label: "a", schema: "public", columnCount: 2, fkCount: 1 },
    { id: "public.b", label: "b", schema: "public", columnCount: 2, fkCount: 2 },
    { id: "public.c", label: "c", schema: "public", columnCount: 2, fkCount: 2 },
    { id: "public.d", label: "d", schema: "public", columnCount: 2, fkCount: 1 },
    { id: "sales.x", label: "x", schema: "sales", columnCount: 2, fkCount: 1 },
    { id: "sales.y", label: "y", schema: "sales", columnCount: 2, fkCount: 1 },
  ];
  const pairs: [TableId, TableId][] = [
    ["public.a", "public.b"],
    ["public.b", "public.c"],
    ["public.c", "public.d"],
    ["sales.x", "sales.y"],
  ];
  const relations = pairs.map(([source, target], i) => ({
    id: `r${i}`,
    source,
    target,
    name: `fk${i}`,
  }));
  const adjacency = new Map<TableId, Set<TableId>>();
  for (const rel of relations) {
    let from = adjacency.get(rel.source);
    if (!from) adjacency.set(rel.source, (from = new Set()));
    from.add(rel.target);
    let to = adjacency.get(rel.target);
    if (!to) adjacency.set(rel.target, (to = new Set()));
    to.add(rel.source);
  }
  return { tables, relations, adjacency, stats: { tables: 6, relations: 4, columns: 12 } };
}

describe("findTableMatches", () => {
  const model = buildModel();

  it("matches table ids case-insensitively", () => {
    expect(findTableMatches(model, "PUBLIC.B")).toEqual(["public.b"]);
    expect(findTableMatches(model, "public.b")).toEqual(["public.b"]);
  });

  it("matches by label too", () => {
    expect(findTableMatches(model, "x")).toEqual(["sales.x"]);
  });

  it("returns multiple matches in model order", () => {
    expect(findTableMatches(model, "s")).toEqual(["sales.x", "sales.y"]);
  });

  it("returns [] for an empty or whitespace query", () => {
    expect(findTableMatches(model, "")).toEqual([]);
    expect(findTableMatches(model, "   ")).toEqual([]);
  });

  it("returns [] when nothing matches", () => {
    expect(findTableMatches(model, "zzz")).toEqual([]);
  });
});

describe("resolveHighlightSet", () => {
  const model = buildModel();

  it("includes the seed at every scope (neighborhood semantics)", () => {
    for (const hops of [1, 2, 3, "domain"] as const) {
      expect(resolveHighlightSet(model, "public.b", hops).has("public.b")).toBe(true);
    }
  });

  it("1 hop = seed + direct neighbors", () => {
    const hl = resolveHighlightSet(model, "public.b", 1);
    expect([...hl].sort()).toEqual(["public.a", "public.b", "public.c"]);
  });

  it("2 hops expands to distance-2", () => {
    const hl = resolveHighlightSet(model, "public.b", 2);
    expect([...hl].sort()).toEqual(["public.a", "public.b", "public.c", "public.d"]);
  });

  it("domain = the full connected component only", () => {
    const hl = resolveHighlightSet(model, "public.a", "domain");
    expect([...hl].sort()).toEqual(["public.a", "public.b", "public.c", "public.d"]);
    // Disconnected pair stays out.
    expect(hl.has("sales.x")).toBe(false);
  });

  it("hops beyond the component clamp to the component", () => {
    const hl = resolveHighlightSet(model, "sales.x", 3);
    expect([...hl].sort()).toEqual(["sales.x", "sales.y"]);
  });
});
