import type { ErGraphModel, TableId } from "../renderer/types";
import { getConnectedComponent, getNeighborhood, type NeighborhoodScope } from "./neighborhood";

/**
 * Overview search (UX pivot — opass-style focus, NOT filter).
 *
 * The large-schema overview always shows the FULL graph. Search never removes
 * nodes from the canvas; it focuses the viewport on the matches and rings
 * them. These pure helpers drive the renderer calls from `cytoscape-view`,
 * keeping the search→highlight semantics unit-testable without a DOM.
 */

/** Table ids whose id or label contains the query (case-insensitive, order = model order). */
export function findTableMatches(model: ErGraphModel, query: string): TableId[] {
  const q = query.trim().toLowerCase();
  if (!q) return [];
  return model.tables
    .filter((t) => t.id.toLowerCase().includes(q) || t.label.toLowerCase().includes(q))
    .map((t) => t.id);
}

/**
 * The hop-scoped highlight set for `seed` (includes the seed itself). Uses the
 * model's pre-built adjacency (undirected FK graph). `hops` 1/2/3 → BFS radius;
 * `"domain"` → the full connected component (opass "related tables" semantics
 * at maximum scope). The caller separates seed vs. neighbors for rendering.
 */
export function resolveHighlightSet(
  model: ErGraphModel,
  seed: TableId,
  hops: NeighborhoodScope,
): Set<TableId> {
  if (hops === "domain") {
    return getConnectedComponent(model.adjacency, seed);
  }
  return getNeighborhood(model.adjacency, seed, hops);
}
