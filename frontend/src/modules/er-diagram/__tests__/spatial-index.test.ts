import { describe, it, expect } from "vitest";
import {
  SpatialIndex,
  computeVisibleNodeIds,
  nodeBounds,
  boxesIntersect,
  type SpatialNode,
} from "../utils/spatial-index";

const node = (id: string, x: number, y: number, w?: number, h?: number): SpatialNode => ({
  id,
  position: { x, y },
  ...(w !== undefined || h !== undefined
    ? { measured: { width: w ?? 220, height: h ?? 120 } }
    : {}),
});

const viewport = (x: number, y: number, zoom = 1, width = 800, height = 600) => ({
  x,
  y,
  zoom,
  width,
  height,
});

describe("nodeBounds", () => {
  it("uses measured dimensions when present", () => {
    const box = nodeBounds(node("a", 10, 20, 200, 100));
    expect(box).toEqual({ minX: 10, minY: 20, maxX: 210, maxY: 120 });
  });

  it("falls back to defaults pre-measurement", () => {
    const box = nodeBounds(node("a", 0, 0));
    expect(box.maxX - box.minX).toBe(220);
    expect(box.maxY - box.minY).toBe(120);
  });
});

describe("boxesIntersect", () => {
  it("detects overlap, adjacency and separation", () => {
    const a = { minX: 0, minY: 0, maxX: 10, maxY: 10 };
    expect(boxesIntersect(a, { minX: 5, minY: 5, maxX: 15, maxY: 15 })).toBe(true);
    expect(boxesIntersect(a, { minX: 10, minY: 0, maxX: 20, maxY: 10 })).toBe(true);
    expect(boxesIntersect(a, { minX: 11, minY: 0, maxX: 20, maxY: 10 })).toBe(false);
  });
});

describe("computeVisibleNodeIds (reference)", () => {
  it("returns nothing for empty viewport or nodes", () => {
    expect(computeVisibleNodeIds([], viewport(0, 0)).size).toBe(0);
    expect(computeVisibleNodeIds([node("a", 0, 0)], viewport(0, 0, 1, 0, 0)).size).toBe(0);
  });

  it("accounts for translation and zoom", () => {
    // Node at (300, 200) with default 220x120 box.
    const nodes = [node("a", 300, 200), node("b", 5000, 5000)];
    expect(computeVisibleNodeIds(nodes, viewport(-300, -200)).has("a")).toBe(true);
    expect(computeVisibleNodeIds(nodes, viewport(-300, -200)).has("b")).toBe(false);
    // At zoom 2 the visible world window is 400x300 starting at (400, 300).
    expect(computeVisibleNodeIds(nodes, viewport(-800, -600, 2)).has("a")).toBe(true);
  });
});

describe("SpatialIndex", () => {
  it("matches the brute-force reference on a grid query", () => {
    const nodes = [
      node("inside", 100, 100),
      node("left-out", -5000, 0),
      node("below-out", 0, 5000),
      node("partial", 750, 0),
      node("corner", 2000, 3000, 400, 300),
    ];
    const index = new SpatialIndex();
    index.build(nodes);

    const vp = viewport(0, 0);
    const gridResult = index.queryNodeIds(vp);
    const reference = computeVisibleNodeIds(nodes, vp);

    expect([...gridResult].sort()).toEqual([...reference].sort());
  });

  it("is stable across rebuilds", () => {
    const index = new SpatialIndex();
    index.build([node("a", 0, 0)]);
    index.build([node("a", 0, 0), node("b", 10, 10)]);
    expect(index.queryNodeIds(viewport(0, 0)).has("a")).toBe(true);
    expect(index.queryNodeIds(viewport(0, 0)).has("b")).toBe(true);
    expect(index.size).toBe(2);
  });

  it("returns edge ids only when both endpoints are visible", () => {
    const index = new SpatialIndex();
    index.build(
      [node("a", 0, 0), node("b", 200, 0), node("c", 5000, 5000)],
      [
        { id: "e1", source: "a", target: "b" },
        { id: "e2", source: "a", target: "c" },
      ],
    );

    const result = index.queryViewport(viewport(0, 0));
    expect(result.nodeIds.has("a")).toBe(true);
    expect(result.nodeIds.has("b")).toBe(true);
    expect(result.nodeIds.has("c")).toBe(false);
    expect(result.edgeIds.has("e1")).toBe(true);
    expect(result.edgeIds.has("e2")).toBe(false);
  });

  it("matches reference when viewport spans the whole graph (fallback path)", () => {
    const nodes = [
      node("a", 0, 0),
      node("b", 1000, 1000),
      node("c", 2000, 2000),
      node("d", -1000, -1000),
    ];
    const index = new SpatialIndex();
    index.build(nodes);

    const wide = viewport(0, 0, 0.1, 1000, 1000);
    const gridResult = index.queryNodeIds(wide);
    const reference = computeVisibleNodeIds(nodes, wide);
    expect([...gridResult].sort()).toEqual([...reference].sort());
  });

  it("handles nodes spanning multiple grid cells", () => {
    // A very large node crosses many cells; must still be found when visible.
    const big = node("big", -2000, -2000, 4000, 4000);
    const index = new SpatialIndex();
    index.build([big, node("small", 5000, 5000)]);

    const result = index.queryNodeIds(viewport(0, 0));
    expect(result.has("big")).toBe(true);
    expect(result.has("small")).toBe(false);
  });
});
