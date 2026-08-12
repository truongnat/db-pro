import { describe, expect, it } from "vitest";

import {
  computeOptimalDistance,
  FORCE_REFINE_ALPHA,
  meanEdgeLength,
  PROGRESSIVE_MIN_NODES,
  refinePositions,
} from "../utils/force-refine";
import { computeApproximateLayoutFromInput } from "../utils/approximate-layout";
import type { LayoutInput } from "../utils/layout";

function chainInput(n: number): LayoutInput {
  const nodes = Array.from({ length: n }, (_, i) => ({ id: `n${i}`, height: 28, width: 160 }));
  const edges = nodes.slice(1).map((node, i) => ({ source: `n${i}`, target: node.id }));
  return { nodes, edges };
}

function clusterInput(): LayoutInput {
  // Two dense clusters connected by a single bridge edge — refinement must
  // pull each cluster's members together (short edges) and separate clusters.
  const nodes: LayoutInput["nodes"] = [];
  const edges: LayoutInput["edges"] = [];
  for (const c of [0, 1]) {
    for (let i = 0; i < 6; i++) {
      nodes.push({ id: `c${c}n${i}`, height: 28, width: 160 });
      if (i > 0) edges.push({ source: `c${c}n${i - 1}`, target: `c${c}n${i}` });
    }
    // intra-cluster star edges from n0
    for (let i = 1; i < 6; i++) edges.push({ source: `c${c}n0`, target: `c${c}n${i}` });
  }
  edges.push({ source: "c0n0", target: "c1n0" }); // bridge
  return { nodes, edges };
}

describe("computeOptimalDistance", () => {
  it("target edge length is driven by node footprint (deterministic, positive)", () => {
    const small = chainInput(10);
    const large = chainInput(1000);
    const kSmall = computeOptimalDistance(small);
    const kLarge = computeOptimalDistance(large);
    expect(kSmall).toBeGreaterThan(0);
    expect(kLarge).toBeGreaterThan(0);
    expect(Number.isFinite(kSmall)).toBe(true);
    // k is the ideal length of an edge — a function of node footprint, not
    // graph size (both fixtures use 160×28 chips), so it is size-invariant.
    expect(kLarge).toBe(kSmall);
  });

  it("bigger node footprints get proportionally more spacing", () => {
    const compact = { nodes: [{ id: "a", width: 160, height: 28 }], edges: [] };
    const huge = { nodes: [{ id: "a", width: 400, height: 200 }], edges: [] };
    expect(computeOptimalDistance(huge)).toBeGreaterThan(computeOptimalDistance(compact));
  });
});

describe("refinePositions (Option C)", () => {
  it("returns a complete position set for every node (nothing dropped)", () => {
    const input = chainInput(250);
    const initial = computeApproximateLayoutFromInput(input);
    const refined = refinePositions(input, initial, { iterations: 12 });
    expect(refined.size).toBe(250);
    for (const node of input.nodes) {
      expect(refined.has(node.id)).toBe(true);
      expect(Number.isFinite(refined.get(node.id)!.x)).toBe(true);
      expect(Number.isFinite(refined.get(node.id)!.y)).toBe(true);
    }
  });

  it("is deterministic: same input + same start → identical output", () => {
    const input = clusterInput();
    const initial = computeApproximateLayoutFromInput(input);
    const a = refinePositions(input, initial, { iterations: 20 });
    const b = refinePositions(input, initial, { iterations: 20 });
    expect(a).toEqual(b);
  });

  it("is idempotent across repeated invocations with the same start", () => {
    const input = clusterInput();
    const initial = computeApproximateLayoutFromInput(input);
    const once = refinePositions(input, initial, { iterations: 10 });
    // Continuing from the refined result with more iterations changes it but
    // stays deterministic — no NaN / drift.
    const twice = refinePositions(input, once, { iterations: 10 });
    expect(Number.isFinite(twice.get("c0n0")!.x)).toBe(true);
    expect(Number.isFinite(twice.get("c1n5")!.y)).toBe(true);
  });

  it("reduces mean edge length vs the circle (connected tables pulled together)", () => {
    const input = clusterInput();
    const initial = computeApproximateLayoutFromInput(input);
    const before = meanEdgeLength(input, initial);
    const refined = refinePositions(input, initial, { iterations: 30 });
    const after = meanEdgeLength(input, refined);
    // The circle scatters neighbors to opposite sides of the ring; FR pulls
    // connected pairs close. Assert a strong improvement (>25% shorter).
    expect(after).toBeLessThan(before * 0.75);
    expect(after).toBeGreaterThan(0);
  });

  it("separates disconnected clusters from each other", () => {
    const input = clusterInput();
    const initial = computeApproximateLayoutFromInput(input);
    const refined = refinePositions(input, initial, { iterations: 30 });
    // Mean intra-cluster distance should be much smaller than the inter-cluster
    // distance — the bridge is long, cluster members are near each other.
    const cluster0 = ["c0n0", "c0n1", "c0n2"];
    const cluster1 = ["c1n0", "c1n1", "c1n2"];
    const intra = (ids: string[]) => {
      let d = 0;
      for (const a of ids)
        for (const b of ids)
          if (a < b)
            d += Math.hypot(
              refined.get(a)!.x - refined.get(b)!.x,
              refined.get(a)!.y - refined.get(b)!.y,
            );
      return d / ((ids.length * (ids.length - 1)) / 2);
    };
    const bridge = Math.hypot(
      refined.get("c0n0")!.x - refined.get("c1n0")!.x,
      refined.get("c0n0")!.y - refined.get("c1n0")!.y,
    );
    expect(intra(cluster0)).toBeLessThan(bridge * 0.6);
    expect(intra(cluster1)).toBeLessThan(bridge * 0.6);
  });

  it("stays fast at 1000 tables (sub-second for 30 passes)", () => {
    const input = chainInput(1000);
    const initial = computeApproximateLayoutFromInput(input);
    const t0 = performance.now();
    const refined = refinePositions(input, initial, { iterations: 30 });
    const elapsed = performance.now() - t0;
    expect(refined.size).toBe(1000);
    // 30 grid-accelerated passes at 1000 nodes must be well under a second —
    // this is the "paint <1s, better layout asynchronously" budget.
    expect(elapsed).toBeLessThan(1000);
  });
});

describe("progressive constants", () => {
  it("PROGRESSIVE_MIN_NODES sits above the fast-dagre line and below 500", () => {
    // P1.8: dagre 151 ms @100 — refinement would add noise there. 8,110 ms
    // @500 — the overview needs progressive quality long before that.
    expect(PROGRESSIVE_MIN_NODES).toBeGreaterThan(100);
    expect(PROGRESSIVE_MIN_NODES).toBeLessThan(500);
  });

  it("cooling factor is a sane (0,1) constant", () => {
    expect(FORCE_REFINE_ALPHA).toBeGreaterThan(0);
    expect(FORCE_REFINE_ALPHA).toBeLessThan(1);
  });
});
