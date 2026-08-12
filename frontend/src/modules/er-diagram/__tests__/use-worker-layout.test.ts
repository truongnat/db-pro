import { describe, expect, it, vi } from "vitest";

import { resolveLayoutRunner } from "../hooks/use-worker-layout";
import { createFallbackLayoutRunner, type LayoutRunner } from "../utils/layout-runner";

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
