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
  /**
   * Full node set to mark as highlighted (neighbors). Computed by the host
   * with the neighborhood utils (hop-scoped BFS), NOT re-derived from a
   * hardcoded 1-hop inside the renderer. Omit to keep the legacy behavior
   * (closedNeighborhood of each selected node).
   */
  highlightNodeIds?: TableId[];
  /**
   * opass-style focus: fade every non-selected, non-highlighted element to
   * near-invisible so the neighborhood reads at a glance (large graphs).
   */
  fadeRest?: boolean;
}

/** Callbacks the host subscribes to; renderers fire them on user interaction. */
export interface ErRendererCallbacks {
  onNodeClick?: (nodeId: TableId) => void;
  /** Empty-canvas tap — clears focus/fade (opass behavior). */
  onBackgroundTap?: () => void;
  onViewportChange?: (viewport: ErViewport) => void;
}

/**
 * P2-1 (review F-REV-4) — concrete color values a canvas renderer can paint.
 * Canvas paints don't resolve CSS `var()` references, so the host resolves the
 * canonical design tokens via `getComputedStyle` and hands concrete values to
 * the renderer. `updateTheme` lets the host swap themes at runtime without
 * destroying the graph.
 */
export interface ErThemeTokens {
  nodeBg: string;
  nodeBorder: string;
  nodeLabel: string;
  selectedNodeBorder: string;
  selectedNodeBg: string;
  neighborNodeBorder: string;
  edgeColor: string;
  edgeArrowColor: string;
  neighborEdgeColor: string;
  /** Search-match marker border (opass-style red ring around matches). */
  searchNodeBorder: string;
}

/**
 * The contract every renderer implements. `mount` is called with the model +
 * a full, stable position set (atomic commit from the layout engine) — a
 * renderer never receives partial positions.
 */
export interface ErRenderer {
  mount(model: ErGraphModel, positions: Map<TableId, ErPosition>): void;
  /**
   * P1-1 — apply a NEW full position set to the already-mounted graph without
   * re-mounting (async layout upgrade: approximate → dagre). A renderer that
   * cannot move nodes in place may treat this as a re-mount.
   */
  updatePositions(positions: Map<TableId, ErPosition>): void;
  updateViewport(viewport: ErViewport): void;
  updateSelection(selection: ErSelection): void;
  /** P2-1 — swap theme colors without destroying the graph. */
  updateTheme(tokens: ErThemeTokens): void;
  focusNode(nodeId: TableId): void;
  dispose(): void;
}
