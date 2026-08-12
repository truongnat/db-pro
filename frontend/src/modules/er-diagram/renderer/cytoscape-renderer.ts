import cytoscape, { type Core, type ElementDefinition } from "cytoscape";
import { OVERVIEW_NODE_HEIGHT, OVERVIEW_NODE_WIDTH } from "../utils/overview-geometry";
import type {
  ErGraphModel,
  ErPosition,
  ErRenderer,
  ErRendererCallbacks,
  ErSelection,
  ErThemeTokens,
  ErViewport,
  TableId,
} from "./types";

export interface CytoscapeErRendererOptions extends ErRendererCallbacks {
  container: HTMLElement;
  /**
   * P2-1 — resolved theme colors (light or dark). If omitted, the dark
   * defaults are used (standalone harness / tests). The host should resolve
   * canonical CSS tokens via getComputedStyle and pass concrete values.
   */
  theme?: Partial<ErThemeTokens>;
  /**
   * Headless mode (P2-2 runtime tests in jsdom, which has no canvas): run
   * cytoscape without a container/renderer. Positions, classes, theme storage
   * and bounding boxes all work; painting does not (nothing to paint).
   */
  headless?: boolean;
}

/**
 * Overview node paint geometry — imported from the layout profile (P1-2): the
 * renderer and the layout engine share one source of truth, so the P1-2
 * failure class (renderer paints 160×28 while dagre lays out 220×640) cannot
 * silently recur.
 */
const NODE_WIDTH = OVERVIEW_NODE_WIDTH;
const NODE_HEIGHT = OVERVIEW_NODE_HEIGHT;

/** Dark-theme defaults — used when no `theme` is supplied (harness/tests). */
const DARK_THEME: ErThemeTokens = {
  nodeBg: "#1e293b",
  nodeBorder: "#475569",
  nodeLabel: "#cbd5e1",
  selectedNodeBorder: "#7dd3fc",
  selectedNodeBg: "#1e3a5f",
  neighborNodeBorder: "#38bdf8",
  edgeColor: "#334155",
  edgeArrowColor: "#475569",
  neighborEdgeColor: "#475569",
};

/**
 * Canvas ERD renderer (P1.9) for large-schema overview/exploration.
 *
 * - Renders the full graph as canvas primitives (≈ a handful of DOM elements
 *   regardless of graph size — the P1.8 benchmark's 31-vs-6,141 DOM result).
 * - Layout is NOT computed here: positions come from the shared layout engine
 *   (dagre in the P1.7 Worker), so renderer and layout stay decoupled.
 * - `updateSelection` highlights the selected node + its 1-hop neighborhood.
 * - P1-1: `updatePositions` applies a new full position set in place (async
 *   layout upgrade — the overview paints an approximate layout immediately and
 *   swaps in dagre positions when the worker finishes).
 * - P2-1: colors come from `ErThemeTokens` (resolved canonical CSS tokens),
 *   `updateTheme` swaps them at runtime without destroying the graph.
 *
 * Relative imports only — this file can be bundled standalone (the benchmark
 * harness imports it directly).
 */
export class CytoscapeErRenderer implements ErRenderer {
  private cy: Core;
  private theme: ErThemeTokens;
  private onNodeClick?: (nodeId: TableId) => void;
  private onViewportChange?: (viewport: ErViewport) => void;
  private viewportRaf: number | null = null;

  constructor(options: CytoscapeErRendererOptions) {
    this.onNodeClick = options.onNodeClick;
    this.onViewportChange = options.onViewportChange;
    this.theme = { ...DARK_THEME, ...options.theme };

    this.cy = cytoscape({
      ...(options.headless ? {} : { container: options.container }),
      headless: options.headless ?? false,
      minZoom: 0.02,
      maxZoom: 4,
      wheelSensitivity: 0.3,
      style: this.buildStylesheet(),
    });

    this.cy.on("tap", "node", (event) => {
      this.onNodeClick?.(event.target.id());
    });

    // pan/zoom fire per gesture frame — coalesce into one callback per rAF.
    this.cy.on("pan zoom", () => {
      if (this.viewportRaf !== null) return;
      this.viewportRaf = requestAnimationFrame(() => {
        this.viewportRaf = null;
        this.onViewportChange?.(this.readViewport());
      });
    });
  }

  private buildStylesheet(): { selector: string; style: Record<string, string | number> }[] {
    const t = this.theme;
    return [
      {
        selector: "node",
        style: {
          "background-color": t.nodeBg,
          "border-color": t.nodeBorder,
          "border-width": "1px",
          "border-opacity": 0.9,
          width: NODE_WIDTH,
          height: NODE_HEIGHT,
          shape: "rectangle",
          label: "data(label)",
          "font-size": "9px",
          "font-family": "ui-sans-serif, system-ui, sans-serif",
          color: t.nodeLabel,
          "text-valign": "center",
          "text-halign": "center",
          "text-wrap": "ellipsis",
          "text-max-width": "150px",
          "overlay-opacity": 0,
        },
      },
      {
        selector: "node.selected",
        style: {
          "border-color": t.selectedNodeBorder,
          "border-width": "2px",
          "background-color": t.selectedNodeBg,
        },
      },
      {
        selector: "node.neighbor",
        style: {
          "border-color": t.neighborNodeBorder,
          "border-opacity": 0.8,
        },
      },
      {
        selector: "edge",
        style: {
          "curve-style": "bezier",
          width: "1px",
          "line-color": t.edgeColor,
          "target-arrow-color": t.edgeArrowColor,
          "target-arrow-shape": "triangle",
          "arrow-scale": 0.5,
          "overlay-opacity": 0,
        },
      },
      {
        selector: "edge.neighbor",
        style: {
          "line-color": t.neighborEdgeColor,
          width: "1.5px",
        },
      },
    ];
  }

  mount(model: ErGraphModel, positions: Map<TableId, ErPosition>): void {
    this.cy.elements().remove();

    const elements: ElementDefinition[] = [];
    for (const table of model.tables) {
      const position = positions.get(table.id);
      if (!position) continue; // mount is only called with a stable full set
      elements.push({
        data: { id: table.id, label: table.label },
        position: { x: position.x, y: position.y },
      });
    }
    for (const rel of model.relations) {
      elements.push({ data: { id: rel.id, source: rel.source, target: rel.target } });
    }
    this.cy.add(elements);
    this.cy.fit(undefined, 40);

    // Emit the initial viewport so the host/HUD sees a valid state.
    this.onViewportChange?.(this.readViewport());
  }

  /**
   * P1-1 — swap in a new full position set without re-mounting (async layout
   * upgrade). Only nodes that already exist are moved; the graph structure is
   * untouched, then we re-fit so the new layout fills the view.
   */
  updatePositions(positions: Map<TableId, ErPosition>): void {
    // Batch: a per-node `position()` loop at 1000 tables would trigger one
    // layout/render pass per mutation; cy.batch() coalesces them into one.
    this.cy.batch(() => {
      for (const [id, pos] of positions) {
        const node = this.cy.getElementById(id);
        if (node.nonempty()) node.position({ x: pos.x, y: pos.y });
      }
    });
    this.cy.fit(undefined, 40);
    this.onViewportChange?.(this.readViewport());
  }

  updateViewport(viewport: ErViewport): void {
    this.cy.pan({ x: viewport.x, y: viewport.y });
    this.cy.zoom(viewport.zoom);
  }

  updateSelection(selection: ErSelection): void {
    this.cy.elements().removeClass("selected neighbor");

    for (const id of selection.nodeIds) {
      const node = this.cy.getElementById(id);
      if (node.nonempty()) node.addClass("selected");
      const neighborhood = node.closedNeighborhood();
      if (neighborhood.length) neighborhood.addClass("neighbor");
    }
  }

  /** P2-1 — swap theme colors at runtime without destroying the graph. */
  updateTheme(tokens: ErThemeTokens): void {
    this.theme = { ...DARK_THEME, ...tokens };
    // In headless mode (unit tests) `cy.style()` is unavailable — the graph is
    // not painted, so there is nothing to re-style; the theme is still stored
    // for `getTheme()` and the next non-headless instance.
    const style = this.cy.style();
    if (style && typeof style.fromJson === "function") {
      // @types/cytoscape models the runtime stylesheet shape as `css`, but the
      // runtime consumes `style` — the local shape above is the real contract.
      style.fromJson(this.buildStylesheet() as never).update();
    }
  }

  /** Current resolved theme — used by tests/instrumentation. */
  getTheme(): ErThemeTokens {
    return { ...this.theme };
  }

  focusNode(nodeId: TableId): void {
    const node = this.cy.getElementById(nodeId);
    if (node.nonempty()) {
      this.cy.animate({ fit: { eles: node, padding: 80 }, duration: 300 });
    }
  }

  /** Programmatic fit-to-content (used by the host's "fit view" action). */
  fit(): void {
    this.cy.fit(undefined, 40);
    this.onViewportChange?.(this.readViewport());
  }

  dispose(): void {
    if (this.viewportRaf !== null) {
      cancelAnimationFrame(this.viewportRaf);
      this.viewportRaf = null;
    }
    this.cy.destroy();
  }

  /** Current graph bounds (screen coords) — used for bench/instrumentation. */
  getBoundingBox(): { x1: number; y1: number; x2: number; y2: number } {
    const bb = this.cy.elements().boundingBox();
    return { x1: bb.x1, y1: bb.y1, x2: bb.x2, y2: bb.y2 };
  }

  private readViewport(): ErViewport {
    const container = this.cy.container();
    const pan = this.cy.pan();
    return {
      x: pan.x,
      y: pan.y,
      zoom: this.cy.zoom(),
      width: container?.clientWidth ?? 0,
      height: container?.clientHeight ?? 0,
    };
  }

  /** Underlying cytoscape core — used by tests to assert graph state. */
  getCy(): Core {
    return this.cy;
  }
}
