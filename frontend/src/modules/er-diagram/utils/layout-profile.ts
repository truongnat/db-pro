import type { ErGraphModel } from "../renderer/types";
import { LAYOUT_NODE_WIDTH, layoutNodeHeight, type LayoutInput } from "./layout";
import { OVERVIEW_NODE_HEIGHT, OVERVIEW_NODE_WIDTH } from "./overview-geometry";

/**
 * P1-2 (review F-REV-2) — renderer-specific layout geometry.
 *
 * Dagre's input node sizes drive rank separation, edge routing and the final
 * graph bounds, so the geometry fed to the layout engine MUST match what the
 * target renderer actually paints. Before this profile split, the overview
 * canvas (nodes painted 160×28) was laid out with React Flow card geometry
 * (220 × dynamic column height) — the graph was stretched, edges unnecessarily
 * long, and `fit()` had to zoom way out.
 *
 * Each profile has a stable `id` that participates in the layout cache hash
 * (see `computeLayoutHash`), so positions computed for one renderer's geometry
 * can never be served to the other's.
 */
export interface LayoutProfile {
  /** Stable identity — part of the cache key; never rename across renders. */
  id: string;
  nodeWidth: number;
  nodeHeight(node: { columnCount: number; compact: boolean }): number;
}

/** React Flow detail cards — full column-aware height. */
export const REACT_FLOW_LAYOUT_PROFILE: LayoutProfile = {
  id: "react-flow",
  nodeWidth: LAYOUT_NODE_WIDTH,
  nodeHeight: ({ columnCount, compact }) => layoutNodeHeight(columnCount, compact),
};

export { OVERVIEW_NODE_HEIGHT, OVERVIEW_NODE_WIDTH } from "./overview-geometry";

/**
 * Cytoscape overview — compact fixed geometry, matching what the canvas
 * renderer paints (OVERVIEW_NODE_WIDTH×OVERVIEW_NODE_HEIGHT node style in
 * CytoscapeErRenderer). The overview does not need column-aware geometry:
 * every node is a flat label chip.
 */
export const OVERVIEW_LAYOUT_PROFILE: LayoutProfile = {
  id: "overview",
  nodeWidth: OVERVIEW_NODE_WIDTH,
  nodeHeight: () => OVERVIEW_NODE_HEIGHT,
};

/**
 * Build a worker-serializable layout input for a renderer-agnostic graph model
 * using the given profile's geometry. Used by the overview path (P1-2) so
 * dagre lays out compact nodes instead of full React Flow cards.
 */
export function buildLayoutInputFromModel(
  model: ErGraphModel,
  profile: LayoutProfile,
): LayoutInput {
  return {
    nodes: model.tables.map((t) => ({
      id: t.id,
      width: profile.nodeWidth,
      height: profile.nodeHeight({ columnCount: t.columnCount, compact: false }),
    })),
    edges: model.relations.map((r) => ({ source: r.source, target: r.target })),
  };
}
