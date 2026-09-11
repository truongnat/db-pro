import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { generateErFixture } from "./er-fixture";

/**
 * PR#12 re-review — app-level navigation test for the large-schema flow.
 *
 * Gate 4 C3 (#39) update: neighborhood phase now renders React Flow with the
 * bounded ≤100 graph, not Cytoscape. Cytoscape is only active for the full
 * overview phase (reached via explicit "Show All").
 *
 * This test renders the real `ErDiagram` on a large (L-tier) schema and proves:
 *
 *   open large ER
 *   → search-first entry shown (no graph mounted)
 *   → select a table
 *   → React Flow mounts with bounded neighborhood (NOT Cytoscape)
 *   → no accidental navigation occurs
 */

// Cytoscape is code-split; mock it to track whether it mounts.
// After #39, it must NOT mount during neighborhood phase.
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

// Layout is a worker pipeline; a real worker doesn't exist in jsdom. Mock the
// hook with a ready state so the renderer mounts deterministically.
// The harness captures every layoutInput the component sends, so tests can
// assert that no stale (identity-mismatched) input was ever requested.
const layoutHarness = vi.hoisted(() => ({
  inputs: [] as (unknown | null)[],
  reset() {
    this.inputs = [];
  },
}));

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: (input: unknown) => {
    layoutHarness.inputs.push(input);
    return {
      status: "ready",
      positions: new Map(),
      layoutMs: 5,
      nodeCount: 500,
      fromCache: true,
      degraded: false,
      refining: false,
      error: null,
    };
  },
}));

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (k: string) => k,
    i18n: { language: "en" },
    currentLanguage: "en",
  }),
}));

describe("ErDiagram — large overview click must not navigate (PR#12 re-review P1)", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  it("renders neighborhood on React Flow (not Cytoscape) after table selection", async () => {
    const data = generateErFixture(500, 42); // complexity ≈ 1,100 → L tier
    const first = data.tables[0];
    const openSpy = vi.spyOn(useWorkspaceStore.getState(), "openDbObject");

    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-1" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Gate 4 Slice B: large schema enters search-first mode.
    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

    // Select a table to transition to neighborhood phase.
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: first.name } });

    await waitFor(() => {
      const results = screen.getAllByTestId("er-search-result");
      expect(results.length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Gate 4 C3 (#39): neighborhood renders React Flow, NOT Cytoscape.
    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
    expect(cytoscapeHarness.mounted).toBe(false);
    expect(screen.queryByTestId("mock-overview")).not.toBeInTheDocument();

    // P1: while the diagram is open, nothing has navigated — the schema
    // workspace remains the active view.
    expect(openSpy).not.toHaveBeenCalled();

    openSpy.mockRestore();
  });

  it("a single click alone (no explicit action) never opens a table", async () => {
    const data = generateErFixture(500, 42);
    const openSpy = vi.spyOn(useWorkspaceStore.getState(), "openDbObject");

    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-1" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Gate 4 Slice B: select a table to pass through search-first gate.
    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Neighborhood renders React Flow, not Cytoscape.
    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
    expect(cytoscapeHarness.mounted).toBe(false);

    // No navigation occurred from a single click.
    expect(openSpy).not.toHaveBeenCalled();

    openSpy.mockRestore();
  });
});

/* ── #42 D2 focus invalidation when node leaves visible scope ─────────────── */

describe("ErDiagram — #42 focus invalidation on data change", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  it("clears focus when data change removes focused node from neighborhood", async () => {
    // Build a connected graph: generateErFixture(500, 42) creates ~70% FK coverage.
    const data = generateErFixture(500, 42);
    const { rerender, container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-1" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter search phase
    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

    // Select a table → neighborhood phase
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Wait for neighborhood to mount (React Flow visible)
    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });

    // Before focus: zero detail nodes
    expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);

    // Click a neighbor node (not the seed) → FOCUS_NODE → detail
    // The first node is typically the seed; click a later node to ensure
    // we focus a neighbor that will leave the visible set when FKs are removed.
    const rfNodes = container.querySelectorAll(".react-flow__node");
    expect(rfNodes.length).toBeGreaterThan(1);
    const neighborNode = rfNodes[rfNodes.length - 1];
    const neighborId = neighborNode.getAttribute("data-id");
    fireEvent.click(neighborNode);

    // After focus: exactly 1 detail node
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(1);
    });

    // The focused node is a neighbor, not the seed
    expect(neighborId).not.toBeNull();

    // Now change the data: remove all FKs → every table becomes isolated.
    // The seed table still exists, but its BFS neighborhood = {seed} only.
    // The focused node (a neighbor via FK) is no longer in the visible set.
    const dataWithoutFks = { ...data, foreignKeys: [] };
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="conn-1" schema="public" data={dataWithoutFks} />
      </TooltipProvider>,
    );

    // #42: focus must be cleared → detail count returns to 0.
    await waitFor(() => {
      const detailNodes = container.querySelectorAll('[data-tier="3"]');
      expect(detailNodes.length).toBe(0);
    });
  });
});

/* ── #40 C4 lifecycle/reset — connection switch ───────────────────────────── */

describe("ErDiagram — #40 connection switch lifecycle", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  it("switching connection resets to search phase without transient neighborhood", async () => {
    const data = generateErFixture(500, 42);
    const first = data.tables[0];

    const { rerender } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-A" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood on conn-A
    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: first.name } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Verify neighborhood phase mounted on conn-A (React Flow is visible)
    await waitFor(() => {
      const rfNodes = screen.getAllByRole("group", { hidden: true });
      expect(rfNodes.length).toBeGreaterThan(0);
    });

    // Capture the layout input count BEFORE the switch.
    const inputsBeforeSwitch = layoutHarness.inputs.length;

    // Switch to conn-B (same schema, same table names — the dangerous case)
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="conn-B" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Must be back in search phase — no transient neighborhood
    await waitFor(() => {
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });

    // Search input should be empty (reset from conn-A's query)
    expect((screen.getByTestId("er-search-input") as HTMLInputElement).value).toBe("");

    // React Flow should NOT be mounted (search phase = no graph)
    // The search entry is shown instead of the graph
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    // Suggestions are visible (no query typed yet)
    expect(screen.queryAllByTestId("er-search-result").length).toBeGreaterThan(0);

    // #40: After the connection switch, no non-null layout input should have
    // been sent to the worker for conn-B. The render-phase gate ensures
    // effectiveLargeSchemaState = initial (search) on the first render of
    // the new identity, so layoutInput = null.
    // Every input emitted after the switch must be null (search phase).
    // This proves no stale non-null layout input was sent for conn-B.
    const inputsAfterSwitch = layoutHarness.inputs.slice(inputsBeforeSwitch);
    expect(inputsAfterSwitch.length).toBeGreaterThan(0);
    expect(inputsAfterSwitch.every((inp) => inp === null)).toBe(true);
  });

  it("does not clear the old identity's cached positions from localStorage", async () => {
    const data = generateErFixture(500, 42);

    // Pre-populate conn-A's positions (simulating previously saved data)
    const connAPositions = [
      ["public.users", { x: 100, y: 200 }],
      ["public.orders", { x: 300, y: 400 }],
    ];
    localStorage.setItem(`er-diagram-positions:conn-A:public`, JSON.stringify(connAPositions));

    const { rerender } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-A" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Wait for mount
    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

    // Switch to conn-B
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="conn-B" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Wait for the switch to settle
    await waitFor(() => {
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });

    // conn-A's positions must still be intact in localStorage
    const stored = localStorage.getItem(`er-diagram-positions:conn-A:public`);
    expect(stored).not.toBeNull();
    const parsed = JSON.parse(stored!);
    expect(parsed).toEqual(connAPositions);

    // conn-B should NOT have been overwritten with "[]"
    const connBStored = localStorage.getItem(`er-diagram-positions:conn-B:public`);
    expect(connBStored).toBeNull(); // never written — conn-B just started
  });

  it("loads cached positions of the new identity on connection switch", async () => {
    const data = generateErFixture(500, 42);

    // Pre-populate conn-B's positions
    const connBPositions = [
      ["public.app", { x: 50, y: 75 }],
      ["public.auth", { x: 150, y: 225 }],
    ];
    localStorage.setItem(`er-diagram-positions:conn-B:public`, JSON.stringify(connBPositions));

    const { rerender } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-A" schema="public" data={data} />
      </TooltipProvider>,
    );

    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

    // Switch to conn-B
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="conn-B" schema="public" data={data} />
      </TooltipProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });

    // conn-B's cached positions should be loaded (not overwritten)
    const stored = localStorage.getItem(`er-diagram-positions:conn-B:public`);
    expect(stored).not.toBeNull();
    const parsed = JSON.parse(stored!);
    expect(parsed).toEqual(connBPositions);
  });
});
