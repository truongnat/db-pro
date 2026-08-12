/**
 * P1.5 — Spatial Index for the ER diagram.
 *
 * The locked architecture's third layer. Receives node bounding boxes (from
 * the layout engine) and answers viewport-intersection queries in sub-linear
 * time via a uniform-grid spatial hash, returning the visible node IDs and
 * the edge IDs whose both endpoints are visible.
 *
 * This replaces the ad-hoc O(N) `computeVisibleNodeIds` scan used by the P1.1
 * HUD. It is the precursor to the full Viewport Engine (P1.6) and keeps the
 * React Flow renderer decoupled from culling decisions (see `ErRenderer`).
 */

export interface ErViewport {
  x: number;
  y: number;
  zoom: number;
  width: number;
  height: number;
}

export interface SpatialNode {
  id: string;
  position: { x: number; y: number };
  measured?: { width?: number; height?: number };
}

export interface SpatialEdge {
  id: string;
  source: string;
  target: string;
}

export interface BoundingBox {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface ViewportQueryResult {
  nodeIds: Set<string>;
  edgeIds: Set<string>;
}

const DEFAULT_NODE_WIDTH = 220;
const DEFAULT_NODE_HEIGHT = 120;

/** Grid cell size in world units — tuned for typical table node sizes. */
const CELL_SIZE = 512;

function cellKey(cx: number, cy: number): string {
  return `${cx},${cy}`;
}

/** Node bounding box, falling back to default dimensions pre-measurement. */
export function nodeBounds(node: SpatialNode): BoundingBox {
  const w = node.measured?.width ?? DEFAULT_NODE_WIDTH;
  const h = node.measured?.height ?? DEFAULT_NODE_HEIGHT;
  return {
    minX: node.position.x,
    minY: node.position.y,
    maxX: node.position.x + w,
    maxY: node.position.y + h,
  };
}

export function boxesIntersect(a: BoundingBox, b: BoundingBox): boolean {
  return a.minX <= b.maxX && a.maxX >= b.minX && a.minY <= b.maxY && a.maxY >= b.minY;
}

/** Convert a screen-space viewport to a world-space query rectangle. */
function viewportToWorldBox(viewport: ErViewport): BoundingBox | null {
  if (viewport.width <= 0 || viewport.height <= 0 || viewport.zoom <= 0) return null;
  const minX = -viewport.x / viewport.zoom;
  const minY = -viewport.y / viewport.zoom;
  return {
    minX,
    minY,
    maxX: minX + viewport.width / viewport.zoom,
    maxY: minY + viewport.height / viewport.zoom,
  };
}

/**
 * Reference implementation — brute-force O(N) viewport intersection, kept
 * for tests and as the fallback when the grid would not pay off. Returns the
 * visible node IDs. Shares `viewportToWorldBox` with the grid query so the
 * two implementations can only differ in traversal strategy, never in the
 * transform.
 */
export function computeVisibleNodeIds(nodes: SpatialNode[], viewport: ErViewport): Set<string> {
  const viewBox = viewportToWorldBox(viewport);
  if (!viewBox) return new Set();

  const visible = new Set<string>();
  for (const node of nodes) {
    if (boxesIntersect(nodeBounds(node), viewBox)) visible.add(node.id);
  }
  return visible;
}

/**
 * Uniform-grid spatial hash over node bounding boxes. Queries return the
 * subset of nodes whose boxes intersect the viewport in O(cells + overlaps)
 * instead of O(N).
 */
export class SpatialIndex {
  private grid = new Map<string, string[]>();
  private bounds = new Map<string, BoundingBox>();
  private nodes: SpatialNode[] = [];
  private edges: SpatialEdge[] = [];

  /** (Re)build the index from the current graph state. */
  build(nodes: SpatialNode[], edges: SpatialEdge[] = []): void {
    this.grid.clear();
    this.bounds.clear();
    this.nodes = nodes;
    this.edges = edges;

    for (const node of nodes) {
      const box = nodeBounds(node);
      this.bounds.set(node.id, box);

      const minCX = Math.floor(box.minX / CELL_SIZE);
      const maxCX = Math.floor(box.maxX / CELL_SIZE);
      const minCY = Math.floor(box.minY / CELL_SIZE);
      const maxCY = Math.floor(box.maxY / CELL_SIZE);

      for (let cx = minCX; cx <= maxCX; cx++) {
        for (let cy = minCY; cy <= maxCY; cy++) {
          const key = cellKey(cx, cy);
          const list = this.grid.get(key);
          if (list) list.push(node.id);
          else this.grid.set(key, [node.id]);
        }
      }
    }
  }

  get size(): number {
    return this.nodes.length;
  }

  /** IDs of nodes intersecting the current viewport. */
  queryNodeIds(viewport: ErViewport): Set<string> {
    const viewBox = viewportToWorldBox(viewport);
    if (!viewBox) return new Set();

    const minCX = Math.floor(viewBox.minX / CELL_SIZE);
    const maxCX = Math.floor(viewBox.maxX / CELL_SIZE);
    const minCY = Math.floor(viewBox.minY / CELL_SIZE);
    const maxCY = Math.floor(viewBox.maxY / CELL_SIZE);

    // When the viewport spans most of the graph, iterating the grid costs more
    // than the reference O(N) scan — fall back to it. This threshold is a pure
    // performance heuristic: both paths produce IDENTICAL results (same
    // transform, same intersection predicate), so the choice only affects
    // which traversal is cheaper — never correctness.
    const spannedCells = (maxCX - minCX + 1) * (maxCY - minCY + 1);
    if (spannedCells > this.nodes.length * 2) {
      return computeVisibleNodeIds(this.nodes, viewport);
    }

    const seen = new Set<string>();
    const visible = new Set<string>();
    for (let cx = minCX; cx <= maxCX; cx++) {
      for (let cy = minCY; cy <= maxCY; cy++) {
        const ids = this.grid.get(cellKey(cx, cy));
        if (!ids) continue;
        for (const id of ids) {
          if (seen.has(id)) continue;
          seen.add(id);
          const box = this.bounds.get(id);
          if (box && boxesIntersect(box, viewBox)) visible.add(id);
        }
      }
    }
    return visible;
  }

  /**
   * Node IDs plus the edge IDs whose both endpoints are visible — the set a
   * culled renderer (React Flow `onlyRenderVisibleElements` or the future
   * Canvas renderer) should draw.
   */
  queryViewport(viewport: ErViewport): ViewportQueryResult {
    const nodeIds = this.queryNodeIds(viewport);
    const edgeIds = new Set<string>();
    for (const edge of this.edges) {
      if (nodeIds.has(edge.source) && nodeIds.has(edge.target)) edgeIds.add(edge.id);
    }
    return { nodeIds, edgeIds };
  }
}
