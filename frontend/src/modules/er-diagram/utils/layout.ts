import dagre from "dagre";
import type { Node, Edge } from "@xyflow/react";

/** Shared node geometry — main thread and worker must agree on these. */
export const LAYOUT_NODE_WIDTH = 220;
export const LAYOUT_ROW_HEIGHT = 20;
export const LAYOUT_HEADER_HEIGHT = 32;
export interface LayoutOptions {
  direction?: "LR" | "TB";
  nodeSep?: number;
  rankSep?: number;
}

/** Minimal plain-node input that can cross the worker boundary. */
export interface LayoutNodeInput {
  id: string;
  width?: number;
  height: number;
}

export interface LayoutEdgeInput {
  source: string;
  target: string;
}

export interface LayoutInput {
  nodes: LayoutNodeInput[];
  edges: LayoutEdgeInput[];
}

export interface LayoutPosition {
  x: number;
  y: number;
}

/** Node height used by dagre, derived from column count / compact flag. */
export function layoutNodeHeight(columnCount: number, compact: boolean): number {
  return compact ? 50 : LAYOUT_HEADER_HEIGHT + columnCount * LAYOUT_ROW_HEIGHT + 8;
}

/**
 * Pure dagre layout over a plain `LayoutInput` — this is what the P1.7 layout
 * Worker executes (and the sync main-thread fallback). No React Flow types
 * cross the worker boundary. Returns a Map keyed by node id.
 *
 * Note: heights are estimated (layout runs before measurement), exactly as the
 * pre-P1.7 main-thread path did.
 */
export function computeLayoutPositions(
  input: LayoutInput,
  options: LayoutOptions = {},
): Map<string, LayoutPosition> {
  const { direction = "LR", nodeSep = 60, rankSep = 100 } = options;

  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({
    rankdir: direction,
    nodesep: nodeSep,
    ranksep: rankSep,
    marginx: 40,
    marginy: 40,
  });

  for (const node of input.nodes) {
    const width = node.width ?? LAYOUT_NODE_WIDTH;
    g.setNode(node.id, { width, height: node.height });
  }
  for (const edge of input.edges) {
    g.setEdge(edge.source, edge.target);
  }

  dagre.layout(g);

  const positions = new Map<string, LayoutPosition>();
  for (const node of input.nodes) {
    const pos = g.node(node.id);
    if (!pos) continue;
    positions.set(node.id, {
      x: pos.x - (node.width ?? LAYOUT_NODE_WIDTH) / 2,
      y: pos.y - pos.height / 2,
    });
  }
  return positions;
}

/**
 * Main-thread convenience wrapper (fallback path + existing tests): lays out
 * React Flow nodes in place. P1.7 moves the real layout into a Worker; this
 * stays only as the sync fallback (non-worker environments).
 */
export function layoutGraph(nodes: Node[], edges: Edge[], options: LayoutOptions = {}): Node[] {
  const input: LayoutInput = {
    nodes: nodes.map((node) => {
      const data = node.data as { columns?: unknown[]; compact?: boolean };
      return {
        id: node.id,
        height: layoutNodeHeight(data?.columns?.length ?? 0, data?.compact ?? false),
      };
    }),
    edges: edges.map((edge) => ({ source: edge.source, target: edge.target })),
  };
  const positions = computeLayoutPositions(input, options);
  return nodes.map((node) => {
    const pos = positions.get(node.id);
    if (!pos) return node;
    return { ...node, position: pos };
  });
}
