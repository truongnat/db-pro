import cytoscape, { type Core, type ElementDefinition } from "cytoscape";
import type {
  ErGraphModel,
  ErPosition,
  ErRenderer,
  ErRendererCallbacks,
  ErSelection,
  ErViewport,
  TableId,
} from "./types";

export interface CytoscapeErRendererOptions extends ErRendererCallbacks {
  container: HTMLElement;
}

/**
 * Canvas ERD renderer (P1.9) for large-schema overview/exploration.
 *
 * - Renders the full graph as canvas primitives (≈ a handful of DOM elements
 *   regardless of graph size — the P1.8 benchmark's 31-vs-6,141 DOM result).
 * - Layout is NOT computed here: positions come from the shared layout engine
 *   (dagre in the P1.7 Worker), so renderer and layout stay decoupled.
 * - `updateSelection` highlights the selected node + its 1-hop neighborhood.
 *
 * Colors are hardcoded for the dark theme (canvas paints don't resolve CSS
 * vars). Relative imports only — this file can be bundled standalone (the
 * benchmark harness imports it directly).
 */
export class CytoscapeErRenderer implements ErRenderer {
  private cy: Core;
  private onNodeClick?: (nodeId: TableId) => void;
  private onViewportChange?: (viewport: ErViewport) => void;
  private viewportRaf: number | null = null;

  constructor(options: CytoscapeErRendererOptions) {
    this.onNodeClick = options.onNodeClick;
    this.onViewportChange = options.onViewportChange;

    this.cy = cytoscape({
      container: options.container,
      minZoom: 0.02,
      maxZoom: 4,
      wheelSensitivity: 0.3,
      style: [
        {
          selector: "node",
          style: {
            "background-color": "#1e293b",
            "border-color": "#475569",
            "border-width": "1px",
            "border-opacity": 0.9,
            width: 160,
            height: 28,
            shape: "rectangle",
            label: "data(label)",
            "font-size": "9px",
            "font-family": "ui-sans-serif, system-ui, sans-serif",
            color: "#cbd5e1",
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
            "border-color": "#7dd3fc",
            "border-width": "2px",
            "background-color": "#1e3a5f",
          },
        },
        {
          selector: "node.neighbor",
          style: {
            "border-color": "#38bdf8",
            "border-opacity": 0.8,
          },
        },
        {
          selector: "edge",
          style: {
            "curve-style": "bezier",
            width: "1px",
            "line-color": "#334155",
            "target-arrow-color": "#475569",
            "target-arrow-shape": "triangle",
            "arrow-scale": 0.5,
            "overlay-opacity": 0,
          },
        },
        {
          selector: "edge.neighbor",
          style: {
            "line-color": "#475569",
            width: "1.5px",
          },
        },
      ],
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
}
