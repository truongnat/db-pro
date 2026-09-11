import { describe, it, expect } from "vitest";
import type { Edge } from "@xyflow/react";
import { resolveEdgeLod, aggregateRelations, EDGE_LOD_THRESHOLDS } from "../utils/edge-lod";

const edge = (id: string, source: string, target: string): Edge => ({
  id,
  source,
  target,
});

describe("resolveEdgeLod", () => {
  it("returns aggregate below 0.25", () => {
    expect(resolveEdgeLod(0.1)).toBe("aggregate");
    expect(resolveEdgeLod(EDGE_LOD_THRESHOLDS.aggregate - 0.001)).toBe("aggregate");
  });

  it("returns simple between 0.25 and 0.6", () => {
    expect(resolveEdgeLod(0.25)).toBe("simple");
    expect(resolveEdgeLod(0.4)).toBe("simple");
    expect(resolveEdgeLod(EDGE_LOD_THRESHOLDS.simple - 0.001)).toBe("simple");
  });

  it("returns full at or above 0.6", () => {
    expect(resolveEdgeLod(0.6)).toBe("full");
    expect(resolveEdgeLod(1)).toBe("full");
  });
});

describe("aggregateRelations", () => {
  it("returns nothing for an empty edge list", () => {
    expect(aggregateRelations([])).toEqual([]);
  });

  it("passes a single relation through with count 1", () => {
    const relations = aggregateRelations([edge("e1", "a", "b")]);
    expect(relations).toHaveLength(1);
    expect(relations[0]).toEqual({ source: "a", target: "b", count: 1 });
  });

  it("merges multiple edges between the same pair", () => {
    const relations = aggregateRelations([
      edge("e1", "a", "b"),
      edge("e2", "a", "b"),
      edge("e3", "a", "b"),
    ]);
    expect(relations).toHaveLength(1);
    expect(relations[0].count).toBe(3);
  });

  it("merges reverse-direction edges into one relation", () => {
    const relations = aggregateRelations([edge("e1", "a", "b"), edge("e2", "b", "a")]);
    expect(relations).toHaveLength(1);
    expect(relations[0].count).toBe(2);
  });

  it("keeps distinct pairs separate", () => {
    const relations = aggregateRelations([
      edge("e1", "a", "b"),
      edge("e2", "a", "c"),
      edge("e3", "b", "c"),
    ]);
    expect(relations).toHaveLength(3);
  });
});
