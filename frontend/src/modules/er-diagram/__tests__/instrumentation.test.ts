import { describe, it, expect } from "vitest";
import { computeFrameStats } from "../utils/instrumentation";

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
