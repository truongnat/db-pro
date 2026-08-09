import dagre from "dagre";
import type { Node, Edge } from "@xyflow/react";

const NODE_WIDTH = 220;
const NODE_HEIGHT_ESTIMATE = 120; // base; adjusted per column count
const ROW_HEIGHT = 20;
const HEADER_HEIGHT = 32;

interface LayoutOptions {
  direction?: "LR" | "TB";
  nodeSep?: number;
  rankSep?: number;
}

/**
 * Compute node positions using dagre layout.
 * Returns a new array of nodes with `position` set.
 */
export function layoutGraph(
  nodes: Node[],
  _edges: Edge[],
  options: LayoutOptions = {},
): Node[] {
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

  for (const node of nodes) {
    const data = node.data as { columns?: unknown[]; compact?: boolean };
    const colCount = data?.columns?.length ?? 0;
    const height = data?.compact
      ? 50
      : HEADER_HEIGHT + colCount * ROW_HEIGHT + 8;
    g.setNode(node.id, { width: NODE_WIDTH, height });
  }

  for (const edge of _edges) {
    g.setEdge(edge.source, edge.target);
  }

  dagre.layout(g);

  return nodes.map((node) => {
    const pos = g.node(node.id);
    if (!pos) return node;
    return {
      ...node,
      position: {
        x: pos.x - NODE_WIDTH / 2,
        y: pos.y - (g.node(node.id)?.height ?? NODE_HEIGHT_ESTIMATE) / 2,
      },
    };
  });
}
