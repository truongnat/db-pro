/**
 * P1.9 — renderer abstraction (locked P1 architecture).
 *
 * Domain/application layers never know which renderer is mounted. A host
 * (ErDiagram) owns the graph model + layout engine and drives whichever
 * `ErRenderer` the current scale/mode selects:
 *
 *   ErGraphModel → Layout Engine (Worker + cache) → ErRenderer
 *                                                    ├─ ReactFlowErRenderer (small/medium, neighborhood)
 *                                                    └─ CytoscapeErRenderer (large overview)
 */

/** Node identity — `schema.tableName`, stable across renderers. */
export type TableId = string;

/** A table's rendering-relevant view data (no backend DTOs leak here). */
export interface ErGraphTable {
  id: TableId;
  label: string;
  schema: string;
  columnCount: number;
  /** Number of outgoing FK relations (degree for hub ranking). */
  fkCount: number;
}

export interface ErGraphRelation {
  id: string;
  source: TableId;
  target: TableId;
  name: string;
}

/** Renderer-agnostic graph model produced by `buildErGraphModel`. */
export interface ErGraphModel {
  tables: ErGraphTable[];
  relations: ErGraphRelation[];
  /** adjacency: nodeId → 1-hop neighbors (undirected). */
  adjacency: Map<TableId, Set<TableId>>;
  stats: {
    tables: number;
    relations: number;
    columns: number;
  };
}

export interface ErPosition {
  x: number;
  y: number;
}

export interface ErViewport {
  x: number;
  y: number;
  zoom: number;
  width: number;
  height: number;
}

export interface ErSelection {
  nodeIds: TableId[];
  edgeIds?: string[];
}

/** Callbacks the host subscribes to; renderers fire them on user interaction. */
export interface ErRendererCallbacks {
  onNodeClick?: (nodeId: TableId) => void;
  onViewportChange?: (viewport: ErViewport) => void;
}

/**
 * The contract every renderer implements. `mount` is called with the model +
 * a full, stable position set (atomic commit from the layout engine) — a
 * renderer never receives partial positions.
 */
export interface ErRenderer {
  mount(model: ErGraphModel, positions: Map<TableId, ErPosition>): void;
  updateViewport(viewport: ErViewport): void;
  updateSelection(selection: ErSelection): void;
  focusNode(nodeId: TableId): void;
  dispose(): void;
}
