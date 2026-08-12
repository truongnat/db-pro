import { describe, expect, it, vi } from "vitest";

import { SYNC_FALLBACK_MAX_NODES, resolveLayoutRunner } from "../hooks/use-worker-layout";
import {
  createApproximateLayoutRunner,
  createFallbackLayoutRunner,
  createWorkerLayoutRunner,
  type LayoutRunner,
} from "../utils/layout-runner";

/**
 * F-MR-2 — a synchronous worker-creation failure must not permanently
 * downgrade the session to the main-thread fallback: the fallback is used for
 * that attempt only, and the next run retries worker creation.
 */
describe("resolveLayoutRunner (F-MR-2)", () => {
  it("returns the current runner when one is already cached (no re-creation)", () => {
    const worker = createFallbackLayoutRunner();
    const create = vi.fn(() => worker);
    const fallback = vi.fn(() => createFallbackLayoutRunner());

    const first = resolveLayoutRunner(null, create, fallback);
    expect(first.runner).toBe(worker);
    expect(first.sticky).toBe(true);
    expect(create).toHaveBeenCalledTimes(1);

    const second = resolveLayoutRunner(worker, create, fallback);
    expect(second.runner).toBe(worker);
    expect(second.sticky).toBe(true);
    expect(create).toHaveBeenCalledTimes(1); // cached — no second creation
    expect(fallback).not.toHaveBeenCalled();
  });

  it("uses the fallback for a failed creation but is NOT sticky — next call retries", () => {
    const fallbackRunner = createFallbackLayoutRunner();
    let attempts = 0;
    const create = () => {
      attempts++;
      throw new Error("module worker unsupported");
    };
    const fallback = () => fallbackRunner;

    const failed = resolveLayoutRunner(null, create, fallback);
    expect(failed.runner).toBe(fallbackRunner);
    expect(failed.sticky).toBe(false); // caller must NOT cache this
    expect(attempts).toBe(1);

    // A subsequent call tries worker creation again instead of reusing the fallback.
    const retried = resolveLayoutRunner(null, create, fallback);
    expect(retried.sticky).toBe(false);
    expect(attempts).toBe(2);
  });

  it("recovery: a successful creation after failures is cached", () => {
    let attempts = 0;
    const worker = createFallbackLayoutRunner();
    const create = () => {
      attempts++;
      if (attempts === 1) throw new Error("transient failure");
      return worker;
    };
    const fallback = () => createFallbackLayoutRunner();

    expect(resolveLayoutRunner(null, create, fallback).sticky).toBe(false);
    expect(attempts).toBe(1);

    const recovered = resolveLayoutRunner(null, create, fallback);
    expect(recovered.runner).toBe(worker);
    expect(recovered.sticky).toBe(true);
    expect(attempts).toBe(2);
  });

  it("falls back and reports sticky only for the runner actually usable", () => {
    // A worker-shaped runner that is not the fallback is treated as sticky once created.
    const runner: LayoutRunner = {
      kind: "worker",
      run: async () => ({ requestId: 0, positions: {}, layoutMs: 0 }),
      dispose: () => undefined,
    };
    const selection = resolveLayoutRunner(
      null,
      () => runner,
      () => createFallbackLayoutRunner(),
    );
    expect(selection.runner).toBe(runner);
    expect(selection.sticky).toBe(true);
  });
});

/**
 * P1-3 (review F-REV-3) — large graphs must NEVER run synchronous dagre on the
 * main thread. The sync fallback is size-gated by `SYNC_FALLBACK_MAX_NODES`;
 * above it, the fallback is the O(N log N) approximate runner instead.
 */
describe("getFallbackRunner size gate (P1-3)", () => {
  it("defines a conservative sync-fallback node cap above the benchmark's 100-table line", () => {
    // P1.8 measured dagre at 151 ms @100 and 8,110 ms @500; 250 keeps the sync
    // fallback sub-second while guaranteeing 500/1000-table graphs never block.
    expect(SYNC_FALLBACK_MAX_NODES).toBeGreaterThan(100);
    expect(SYNC_FALLBACK_MAX_NODES).toBeLessThan(500);
  });

  it("the approximate fallback is fast and deterministic for large inputs", async () => {
    const runner = createApproximateLayoutRunner();
    const nodes = Array.from({ length: 1000 }, (_, i) => ({
      id: `n${i}`,
      height: 28,
      width: 160,
    }));
    const edges = nodes.slice(1).map((n, i) => ({ source: `n${i}`, target: n.id }));

    const t0 = performance.now();
    const result = await runner.run({ requestId: 1, input: { nodes, edges }, options: {} });
    const elapsed = performance.now() - t0;

    expect(result.positions).toHaveProperty("n0");
    expect(Object.keys(result.positions).length).toBe(1000);
    // Sub-second guaranteed — it's a circle, not dagre. Generous bound vs the
    // 8 s / 122 s synchronous dagre it replaces.
    expect(elapsed).toBeLessThan(1000);
    expect(Number.isFinite(result.positions.n0.x)).toBe(true);
    expect(Number.isFinite(result.positions.n0.y)).toBe(true);
  });

  it("large-graph fallback is deterministic across runs", async () => {
    const runner = createApproximateLayoutRunner();
    const input = {
      nodes: [
        { id: "a", height: 28, width: 160 },
        { id: "b", height: 28, width: 160 },
        { id: "c", height: 28, width: 160 },
      ],
      edges: [
        { source: "a", target: "b" },
        { source: "b", target: "c" },
      ],
    };
    const r1 = await runner.run({ requestId: 1, input, options: {} });
    const r2 = await runner.run({ requestId: 2, input, options: {} });
    expect(r1.positions).toEqual(r2.positions);
  });

  it("runners are tagged with their kind so degraded state never needs re-derivation", () => {
    expect(createApproximateLayoutRunner().kind).toBe("approximate");
    expect(createFallbackLayoutRunner().kind).toBe("dagre-sync");
  });
});

describe("createWorkerLayoutRunner (P1-3 hard gate)", () => {
  it("throws when Worker is unavailable instead of silently returning sync dagre", () => {
    // P1-3 — before this fix, a Worker-less environment silently got the sync
    // dagre fallback, bypassing the size gate: a 500/1000-table graph would run
    // dagre on the main thread (8 s / 122 s per P1.8). It must throw so
    // resolveLayoutRunner routes to the size-gated getFallbackRunner.
    const workerGlobal = globalThis.Worker;
    try {
      // @ts-expect-error — deliberately removing the global for the test
      delete globalThis.Worker;
      expect(() => createWorkerLayoutRunner()).toThrow(/Worker is unavailable/);
    } finally {
      if (workerGlobal) globalThis.Worker = workerGlobal;
    }
  });
});
