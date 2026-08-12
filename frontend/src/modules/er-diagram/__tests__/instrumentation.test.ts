import { describe, it, expect } from "vitest";
import { computeFrameStats, computeVisibleNodeIds } from "../utils/instrumentation";

describe("computeFrameStats", () => {
  it("returns zeros for an empty sample set", () => {
    expect(computeFrameStats([])).toEqual({ samples: 0, avgMs: 0, maxMs: 0, p95Ms: 0 });
  });

  it("computes avg, max and p95 from samples", () => {
    const stats = computeFrameStats([10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    expect(stats.samples).toBe(10);
    expect(stats.avgMs).toBe(55);
    expect(stats.maxMs).toBe(100);
    // p95 index = floor(10 * 0.95) = 9 → 100
    expect(stats.p95Ms).toBe(100);
  });

  it("handles a single sample", () => {
    const stats = computeFrameStats([16]);
    expect(stats.avgMs).toBe(16);
    expect(stats.maxMs).toBe(16);
    expect(stats.p95Ms).toBe(16);
  });

  it("does not mutate the input", () => {
    const input = [5, 1, 3];
    computeFrameStats(input);
    expect(input).toEqual([5, 1, 3]);
  });
});

describe("computeVisibleNodeIds", () => {
  const node = (id: string, x: number, y: number, w = 200, h = 100) => ({
    id,
    position: { x, y },
    measured: { width: w, height: h },
  });

  it("returns nothing for an empty node list", () => {
    const visible = computeVisibleNodeIds([], { x: 0, y: 0, zoom: 1, width: 800, height: 600 });
    expect(visible.size).toBe(0);
  });

  it("returns nothing for an empty viewport", () => {
    const visible = computeVisibleNodeIds([node("a", 0, 0)], {
      x: 0,
      y: 0,
      zoom: 1,
      width: 0,
      height: 0,
    });
    expect(visible.size).toBe(0);
  });

  it("includes nodes intersecting the viewport at zoom 1", () => {
    const nodes = [
      node("inside", 100, 100),
      node("left-out", -500, 0),
      node("below-out", 0, 5000),
      node("partial", 750, 0), // left edge at 750 < 800 → intersects
    ];
    const visible = computeVisibleNodeIds(nodes, { x: 0, y: 0, zoom: 1, width: 800, height: 600 });
    expect(visible.has("inside")).toBe(true);
    expect(visible.has("left-out")).toBe(false);
    expect(visible.has("below-out")).toBe(false);
    expect(visible.has("partial")).toBe(true);
  });

  it("accounts for viewport translation (pan)", () => {
    // Viewport translated so that world (100, 100) lands at screen center.
    const nodes = [node("a", 100, 100), node("b", 5000, 5000)];
    const visible = computeVisibleNodeIds(nodes, {
      x: -300,
      y: -200,
      zoom: 1,
      width: 800,
      height: 600,
    });
    expect(visible.has("a")).toBe(true);
    expect(visible.has("b")).toBe(false);
  });

  it("accounts for zoom (world window shrinks as zoom grows)", () => {
    const nodes = [node("near", 300, 200), node("far", 1500, 1200)];
    // At zoom 2 the visible world window is 400x300 centered on the viewport center.
    const visible = computeVisibleNodeIds(nodes, {
      x: -800,
      y: -600,
      zoom: 2,
      width: 800,
      height: 600,
    });
    expect(visible.has("near")).toBe(true);
    expect(visible.has("far")).toBe(false);
  });

  it("falls back to default node size when measured is missing", () => {
    const nodes = [{ id: "unmeasured", position: { x: 0, y: 0 } }];
    const visible = computeVisibleNodeIds(nodes, { x: 0, y: 0, zoom: 1, width: 800, height: 600 });
    expect(visible.has("unmeasured")).toBe(true);
  });
});
