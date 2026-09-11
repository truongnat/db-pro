import type { Edge } from "@xyflow/react";

/**
 * P1.4 — Edge level-of-detail.
 *
 * The locked architecture gives edges their own LOD bands, independent of node
 * LOD (0.25 / 0.6 per the locked spec):
 *   aggregate (< 0.25) → straight lines, no markers, no FK labels, relations
 *                        between the same pair of tables merged into one edge
 *                        carrying the relation count (e.g. `A ─31─ B`)
 *   simple  (0.25–0.6)  → individual relation edges, straight, no markers/labels
 *   full    (> 0.6)     → normal FK edges (smoothstep, markers, FK name labels)
 */

export type EdgeLodLevel = "aggregate" | "simple" | "full";

export const EDGE_LOD_THRESHOLDS = {
  aggregate: 0.25,
  simple: 0.6,
} as const;

export function resolveEdgeLod(zoom: number): EdgeLodLevel {
  if (zoom < EDGE_LOD_THRESHOLDS.aggregate) return "aggregate";
  if (zoom < EDGE_LOD_THRESHOLDS.simple) return "simple";
  return "full";
}

export interface AggregatedRelation {
  source: string;
  target: string;
  count: number;
}

/**
 * Merge all edges between the same unordered pair of nodes into a single
 * relation carrying the total count. Direction-insensitive: `A→B` and `B→A`
 * count as the same relation at this zoom level.
 */
export function aggregateRelations(edges: Edge[]): AggregatedRelation[] {
  const byPair = new Map<string, AggregatedRelation>();
  for (const edge of edges) {
    const key = [edge.source, edge.target].sort().join("\u0000");
    const existing = byPair.get(key);
    if (existing) {
      existing.count += 1;
    } else {
      byPair.set(key, { source: edge.source, target: edge.target, count: 1 });
    }
  }
  return [...byPair.values()];
}
