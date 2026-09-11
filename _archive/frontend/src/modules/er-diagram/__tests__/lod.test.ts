import { describe, it, expect } from "vitest";
import { resolveLod, lodTier, LOD_ORDER, LOD_THRESHOLDS } from "../utils/lod";

describe("resolveLod", () => {
  it("returns dot at very low zoom", () => {
    expect(resolveLod(0.05)).toBe("dot");
    expect(resolveLod(LOD_THRESHOLDS.dot - 0.001)).toBe("dot");
  });

  it("returns compact in the second band", () => {
    expect(resolveLod(0.2)).toBe("compact");
    expect(resolveLod(0.3)).toBe("compact");
    expect(resolveLod(LOD_THRESHOLDS.compact - 0.001)).toBe("compact");
  });

  it("returns summary in the third band", () => {
    expect(resolveLod(0.45)).toBe("summary");
    expect(resolveLod(0.6)).toBe("summary");
    expect(resolveLod(LOD_THRESHOLDS.summary - 0.001)).toBe("summary");
  });

  it("returns detail at high zoom", () => {
    expect(resolveLod(0.7)).toBe("detail");
    expect(resolveLod(1)).toBe("detail");
    expect(resolveLod(2)).toBe("detail");
  });

  it("caps detail at summary when compact is enabled", () => {
    expect(resolveLod(1, true)).toBe("summary");
    expect(resolveLod(0.6, true)).toBe("summary");
  });

  it("does not elevate low levels when compact is enabled", () => {
    expect(resolveLod(0.1, true)).toBe("dot");
    expect(resolveLod(0.3, true)).toBe("compact");
  });

  it("thresholds partition the zoom axis monotonically", () => {
    const zooms = [0.01, 0.19, 0.2, 0.44, 0.45, 0.69, 0.7, 1, 2];
    const lods = zooms.map((z) => resolveLod(z));
    // Values must be non-decreasing in LOD_ORDER index.
    for (let i = 1; i < lods.length; i++) {
      expect(LOD_ORDER.indexOf(lods[i])).toBeGreaterThanOrEqual(LOD_ORDER.indexOf(lods[i - 1]));
    }
  });
});

describe("lodTier", () => {
  it("maps levels to numeric tiers", () => {
    expect(lodTier("dot")).toBe(0);
    expect(lodTier("compact")).toBe(1);
    expect(lodTier("summary")).toBe(2);
    expect(lodTier("detail")).toBe(3);
  });
});
