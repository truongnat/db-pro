import type { ErGraphModel, ErPosition, TableId } from "../renderer/types";
import type { LayoutInput, LayoutPosition } from "./layout";

/**
 * P1-1 (review F-REV-1) — fast approximate layout for the canvas overview.
 *
 * Cold dagre on 1000 tables takes ~122 s (P1.8 evidence). The overview must
 * paint something usable immediately instead of an empty canvas: a
 * deterministic, O(N log N) degree-ordered circle places every node in a
 * single pass (sub-millisecond even at 1000 tables). The real dagre layout
 * arrives later via `ErRenderer.updatePositions` (async upgrade), so the user
 * gets a diagram in well under 1 s and a better one whenever the worker
 * finishes.
 *
 * Also used as the large-graph fallback when the layout worker is unavailable
 * (P1-3) — never run synchronous dagre on the main thread for big graphs.
 *
 * Deterministic: same graph → same positions (tables sorted by FK degree,
 * stable tiebreak by id). Not persisted — recomputed per mount.
 */
export function computeApproximateOverviewLayout(model: ErGraphModel): Map<TableId, ErPosition> {
  const tables = [...model.tables].sort((a, b) => {
    if (b.fkCount !== a.fkCount) return b.fkCount - a.fkCount;
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });

  const n = tables.length;
  // Radius grows with √N so node density stays roughly constant; the 0.9
  // factor leaves a little margin for fit padding.
  const radius = Math.max(200, Math.sqrt(n) * 90 * 0.9);
  const positions = new Map<TableId, ErPosition>();

  for (let i = 0; i < n; i++) {
    const angle = (i / Math.max(n, 1)) * Math.PI * 2;
    positions.set(tables[i].id, {
      x: radius * Math.cos(angle),
      y: radius * Math.sin(angle),
    });
  }
  return positions;
}

/**
 * Same degree-ordered circle, but over a worker-serializable `LayoutInput` —
 * used by the P1-3 approximate fallback runner (no graph model on hand).
 * Degree = number of incident edges in the input. Deterministic.
 */
export function computeApproximateLayoutFromInput(input: LayoutInput): Map<string, LayoutPosition> {
  const degree = new Map<string, number>();
  for (const edge of input.edges) {
    degree.set(edge.source, (degree.get(edge.source) ?? 0) + 1);
    degree.set(edge.target, (degree.get(edge.target) ?? 0) + 1);
  }

  const nodes = [...input.nodes].sort((a, b) => {
    const da = degree.get(a.id) ?? 0;
    const db = degree.get(b.id) ?? 0;
    if (db !== da) return db - da;
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });

  const n = nodes.length;
  const radius = Math.max(200, Math.sqrt(n) * 90 * 0.9);
  const positions = new Map<string, LayoutPosition>();

  for (let i = 0; i < n; i++) {
    const angle = (i / Math.max(n, 1)) * Math.PI * 2;
    positions.set(nodes[i].id, {
      x: radius * Math.cos(angle),
      y: radius * Math.sin(angle),
    });
  }
  return positions;
}
