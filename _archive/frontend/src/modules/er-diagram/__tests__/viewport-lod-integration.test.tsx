import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Edge, Viewport } from "@xyflow/react";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { generateErFixture } from "./er-fixture";
import type {
  IntrospectResult,
  SchemaColumnDto,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

/* ── Mocks ───────────────────────────────────────────────────────────────── */

// Capture onViewportChange so tests can invoke it with controlled zoom values.
// This bridges jsdom (no real viewport events) to the production wiring:
//   ErDiagram → ReactFlow.onViewportChange → resolveLod →
//   currentLod/currentEdgeLod → tieredNodes → displayEdges → DOM.
const capturedCb = vi.hoisted(() => ({
  fn: null as ((vp: Viewport) => void) | null,
  edges: [] as Edge[],
}));

vi.mock("@xyflow/react", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@xyflow/react")>();
  const ActualRF = actual.ReactFlow;

  return {
    ...actual,
    ReactFlow: (props: Record<string, unknown>) => {
      const origCb = props.onViewportChange as ((vp: Viewport) => void) | undefined;
      if (origCb) {
        // Always update to the latest callback so tests can invoke
        // the production onViewportChange with the current compact state.
        capturedCb.fn = origCb;
      }
      // Capture edges so tests can assert handle-stripping on the actual
      // edge objects passed to React Flow (not just DOM inference).
      if (Array.isArray(props.edges)) {
        capturedCb.edges = props.edges as Edge[];
      }
      // Suppress onViewportChange forwarding to ActualRF. We capture the
      // production callback and invoke it manually via setZoom(), but prevent
      // React Flow from firing its own viewport events in jsdom.
      return <ActualRF {...props} onViewportChange={undefined} />;
    },
  };
});

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: () => ({
    status: "ready",
    positions: new Map(),
    layoutMs: 5,
    nodeCount: 0,
    fromCache: true,
    degraded: false,
    refining: false,
    error: null,
  }),
}));

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (k: string) => k,
    i18n: { language: "en" },
    currentLanguage: "en",
  }),
}));

const cytoscapeHarness = vi.hoisted(() => ({
  mounted: false,
  reset() {
    this.mounted = false;
  },
}));

vi.mock("../components/cytoscape-view", () => ({
  CytoscapeErView: () => {
    cytoscapeHarness.mounted = true;
    return <div data-testid="mock-cytoscape" />;
  },
}));

/* ── Helpers ─────────────────────────────────────────────────────────────── */

function setZoom(zoom: number) {
  expect(capturedCb.fn, "onViewportChange not captured").not.toBeNull();
  capturedCb.fn!({ x: 0, y: 0, zoom });
}

async function enterNeighborhood(container: HTMLElement) {
  const searchInput = screen.getByTestId("er-search-input");
  fireEvent.change(searchInput, { target: { value: "app" } });
  await waitFor(() => {
    expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
  });
  fireEvent.click(screen.getAllByTestId("er-search-result")[0]);
  await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
}

function buildSmallSchema(): IntrospectResult {
  const tables: TableDto[] = [];
  const columns: SchemaColumnDto[] = [];
  const foreignKeys: SchemaForeignKeyDto[] = [];
  const tableCount = 20;

  for (let i = 0; i < tableCount; i++) {
    const name = `tbl_${String(i).padStart(4, "0")}`;
    tables.push({ name, schema: "public", rowCount: 0 });
    for (let c = 0; c < 5; c++) {
      columns.push({
        name: c === 0 ? "id" : `col_${c}`,
        dataType: "text",
        nullable: true,
        defaultValue: null,
        isPrimaryKey: c === 0,
        tableName: name,
        schema: "public",
      });
    }
    if (i > 0) {
      foreignKeys.push({
        name: `fk_${name}_prev`,
        fromTable: name,
        fromColumns: ["id"],
        toTable: `tbl_${String(i - 1).padStart(4, "0")}`,
        toColumns: ["id"],
        schema: "public",
        toSchema: "public",
      });
    }
  }

  return {
    schemas: [{ name: "public" }],
    tables,
    columns,
    primaryKeys: [],
    indexes: [],
    foreignKeys,
    views: [],
    triggers: [],
  };
}

/* ── Tests ───────────────────────────────────────────────────────────────── */

describe("#43 D3 — viewport-driven LOD integration", () => {
  beforeEach(() => {
    capturedCb.fn = null;
    capturedCb.edges = [];
    cytoscapeHarness.reset();
    localStorage.clear();
  });

  /* 1. Large neighborhood → zoom 0.50 → siblings summary (tier 2) */

  it("zoom 0.50 in neighborhood → non-focused nodes become summary/tier=2", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    setZoom(0.5);

    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="2"]').length).toBeGreaterThan(0);
    });
    expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  /* 2. Focus B → then zoom to 0.30 → B stays tier 3, siblings tier 1 */

  it("focus B first, then zoom 0.30 → B stays detail, siblings compact", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Focus a node FIRST (at default zoom)
    const rfNodes = container.querySelectorAll(".react-flow__node");
    expect(rfNodes.length).toBeGreaterThan(0);
    fireEvent.click(rfNodes[rfNodes.length - 1]);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(1);
    });

    // NOW change zoom to compact (0.30 → resolveLod = "compact")
    setZoom(0.3);

    // Focused node STILL detail (tier 3) — focus override survives zoom change
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(1);
    });
    // Siblings at compact tier (tier 1)
    expect(container.querySelectorAll('[data-tier="1"]').length).toBeGreaterThan(0);
  });

  /* 3. Pane click → clear focus → detail count = 0 */

  it("pane click clears focus → detail count returns to 0", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Focus a node first
    const rfNodes = container.querySelectorAll(".react-flow__node");
    fireEvent.click(rfNodes[rfNodes.length - 1]);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(1);
    });

    // Click the pane background → CLEAR_FOCUS
    const pane = container.querySelector(".react-flow__pane");
    expect(pane, "react-flow__pane must exist for pane-click test").not.toBeNull();
    fireEvent.click(pane!);

    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);
    });
  });

  /* 4. Zoom < 0.20 → no [data-column] DOM */

  it("zoom < 0.20 → dot tier, no column rows in DOM", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    setZoom(0.1);

    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="0"]').length).toBeGreaterThan(0);
    });
    expect(container.querySelectorAll("[data-column]").length).toBe(0);
    expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);
  });

  /* 5. Critical band 0.60–0.70: edge full, node summary → handles stripped */

  it("zoom 0.65 → edge LOD full but node summary → edges have no sourceHandle/targetHandle", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Zoom to the critical band: node=summary (0.65 < 0.70), edge=full (0.65 ≥ 0.60)
    setZoom(0.65);

    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="2"]').length).toBeGreaterThan(0);
    });
    // Node LOD: summary — no per-column handles exist on any node
    expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);
    expect(container.querySelectorAll("[data-column]").length).toBe(0);

    // Capture the actual edge objects passed to React Flow.
    // At this zoom, currentLod = "summary" ≠ "detail", so displayEdges
    // must strip sourceHandle/targetHandle to prevent error 008.
    const capturedEdges = capturedCb.edges;
    expect(capturedEdges.length).toBeGreaterThan(0);
    for (const edge of capturedEdges) {
      expect(edge.sourceHandle).toBeUndefined();
      expect(edge.targetHandle).toBeUndefined();
    }
  });

  /* 6. Compact toggle + focus: deterministic LOD transitions */

  it("setZoom(0.80) → detail>0; click compact → detail=0, summary>0; focus B → detail=1", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Step 1: setZoom(0.80) → detail nodes
    setZoom(0.8);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBeGreaterThan(0);
    });

    // Step 2: click compact toggle → base LOD capped at summary.
    // The compact button contains the Columns2 icon (lucide-columns2).
    // We must NOT rely on panel/button index because React Flow's built-in
    // Controls component also renders buttons in a bottom-left panel.
    const compactButton = container.querySelector(".lucide-columns-2")?.closest("button");
    expect(compactButton, "compact button not found").not.toBeNull();
    await act(async () => {
      fireEvent.click(compactButton!);
    });

    // After compact: resolveLod(0.80, true) = "summary" → detail=0, summary>0
    await waitFor(
      () => {
        expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);
        expect(container.querySelectorAll('[data-tier="2"]').length).toBeGreaterThan(0);
      },
      { timeout: 3000 },
    );

    // Step 3: focus B → it becomes detail (tier=3) despite compact cap
    const rfNodes = container.querySelectorAll(".react-flow__node");
    expect(rfNodes.length).toBeGreaterThan(0);
    fireEvent.click(rfNodes[rfNodes.length - 1]);

    // Exactly 1 detail node (focused), siblings remain summary
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(1);
    });
    expect(container.querySelectorAll('[data-tier="2"]').length).toBeGreaterThan(0);
  });

  /* 7. Small/medium schema regression — viewport events change LOD normally */

  it("small/medium schema: viewport events change LOD without regression", async () => {
    const data = buildSmallSchema();
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Small schema goes directly to React Flow (no search-first)
    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
    await waitFor(() => {
      expect(container.querySelectorAll(".react-flow__node").length).toBeGreaterThan(0);
    });

    // Zoom to summary
    setZoom(0.5);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="2"]').length).toBeGreaterThan(0);
    });

    // Zoom to dot
    setZoom(0.1);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="0"]').length).toBeGreaterThan(0);
    });

    // Zoom back to detail
    setZoom(1.0);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBeGreaterThan(0);
    });

    expect(cytoscapeHarness.mounted).toBe(false);
  });
});
