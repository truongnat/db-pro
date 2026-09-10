import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Edge, Viewport } from "@xyflow/react";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { generateErFixture } from "./er-fixture";

/**
 * #44 E1 — explicit "Show all N tables" transition from bounded neighborhood.
 *
 * Proves:
 *   A. Button only renders in neighborhood phase (absent in search)
 *   B. Click → SHOW_ALL → overview (Cytoscape mounts)
 *   C. No implicit trigger (zoom/pan/search/layout) enters overview
 *   D. Label uses full schema table count, not neighborhood count
 */

/* ── Mocks ───────────────────────────────────────────────────────────────── */

const cytoscapeHarness = vi.hoisted(() => ({
  mounted: false,
  reset() {
    this.mounted = false;
  },
}));

vi.mock("../components/cytoscape-view", () => ({
  CytoscapeErView: () => {
    cytoscapeHarness.mounted = true;
    return <div data-testid="mock-overview" />;
  },
}));

// Controllable layout harness — tests can transition computing → ready.
const layoutHarness = vi.hoisted(() => ({
  status: "ready" as "computing" | "ready",
  reset() {
    this.status = "ready";
  },
}));

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: () => ({
    status: layoutHarness.status,
    positions: layoutHarness.status === "ready" ? new Map() : new Map(),
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

// Capture onViewportChange so tests can invoke it with controlled zoom values.
// Suppress forwarding to ActualRF to eliminate jsdom viewport noise.
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
      if (origCb) capturedCb.fn = origCb;
      if (Array.isArray(props.edges)) capturedCb.edges = props.edges as Edge[];
      return <ActualRF {...props} onViewportChange={undefined} />;
    },
  };
});

/* ── Helpers ─────────────────────────────────────────────────────────────── */

function setZoom(zoom: number) {
  if (capturedCb.fn) capturedCb.fn({ x: 0, y: 0, zoom });
}

function simulatePan(x: number, y: number) {
  if (capturedCb.fn) capturedCb.fn({ x, y, zoom: 1.0 });
}

/** Navigate from search → neighborhood by selecting the first search result. */
async function enterNeighborhood(container: HTMLElement) {
  const searchInput = screen.getByTestId("er-search-input");
  fireEvent.change(searchInput, { target: { value: "app" } });
  await waitFor(() => {
    expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
  });
  fireEvent.click(screen.getAllByTestId("er-search-result")[0]);
  await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
}

/* ── Tests ───────────────────────────────────────────────────────────────── */

describe("#44 E1 — Show All transition", () => {
  beforeEach(() => {
    capturedCb.fn = null;
    capturedCb.edges = [];
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  /* A. Button only renders in neighborhood phase */

  it("button absent in search, present in neighborhood with correct label", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Search phase: button must NOT exist
    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());
    expect(screen.queryByTestId("er-show-all-tables")).not.toBeInTheDocument();

    // Select a table → neighborhood
    await enterNeighborhood(container);

    // Neighborhood phase: button must exist
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });
    // Cytoscape must NOT be mounted in neighborhood
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  /* B. Explicit click → SHOW_ALL → overview */

  it("click Show all → Cytoscape mounts (overview transition)", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Wait for the button
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // Click Show all → transitions to overview
    const showAllButton = screen.getByTestId("er-show-all-tables");
    act(() => {
      fireEvent.click(showAllButton);
    });

    // Overview: Cytoscape mounts
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(true);
    });
    // React Flow neighborhood is gone (overview uses Cytoscape, not React Flow)
    expect(container.querySelector(".react-flow")).toBeNull();
  });

  /* C. Negative triggers — no implicit overview transition */

  it("zoom does not trigger overview", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // Zoom low
    setZoom(0.3);
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(false);
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // Zoom high
    setZoom(1.2);
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(false);
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });
  });

  it("search input change does not trigger overview", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // The neighborhood search input uses the same placeholder as ErSearchEntry
    // but without the er-search-input testid. Find it by role + placeholder.
    const neighborhoodSearch = screen.getByPlaceholderText("shell.sidebar.searchObjects...");
    fireEvent.change(neighborhoodSearch, { target: { value: "billing" } });

    // Still in neighborhood — no overview
    expect(cytoscapeHarness.mounted).toBe(false);
    expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
  });

  it("pan/move does not trigger overview", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // Simulate pan (viewport x/y change, zoom stays constant)
    simulatePan(100, 200);
    simulatePan(-50, -100);

    expect(cytoscapeHarness.mounted).toBe(false);
    expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
  });

  it("layout computing → ready does not trigger overview", async () => {
    // Start layout in "computing" state — the mock returns status from the
    // controllable harness, so the component sees computing on first render.
    layoutHarness.status = "computing";

    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // While computing: no overview
    expect(cytoscapeHarness.mounted).toBe(false);

    // Transition layout: computing → ready.
    // The hook reads layoutHarness.status during render, so we need a
    // re-render to pick up the change. Changing the search input triggers
    // setSearchQuery → component re-renders → hook returns "ready".
    layoutHarness.status = "ready";
    const neighborhoodSearch = screen.getByPlaceholderText("shell.sidebar.searchObjects...");
    fireEvent.change(neighborhoodSearch, { target: { value: "x" } });

    // After layout transition: still in neighborhood, no overview
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(false);
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });
  });

  /* D. Full count — label uses tablesInSchema.length, not neighborhood count */

  it("label shows full schema table count (500), not neighborhood count", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Verify via role + name (accessibility semantics)
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Show all 500 tables" })).toBeInTheDocument();
    });

    // Also verify via data-testid
    const button = screen.getByTestId("er-show-all-tables");
    expect(button.textContent).toBe("Show all 500 tables");
  });
});
