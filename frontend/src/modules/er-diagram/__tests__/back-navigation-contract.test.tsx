import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { generateErFixture } from "./er-fixture";

/**
 * #46 E3 — back navigation from overview to neighborhood/search.
 *
 * Contract matrix:
 * | Flow                                | Result                              |
 * |-------------------------------------|-------------------------------------|
 * | neighborhood → overview → back      | exact same seed + hops + node IDs   |
 * | overview đổi highlight hops → back  | neighborhood cũ không đổi           |
 * | overview → search                   | 0 nodes, layout null, seed null     |
 * | cycle ×2                            | exact same bounded set              |
 * | focus → overview → back             | focus cleared                       |
 * | overview → schema/connection change | search wins                         |
 */

/* ── Mocks ───────────────────────────────────────────────────────────────── */

const cytoscapeHarness = vi.hoisted(() => ({
  mounted: false,
  reset() {
    this.mounted = false;
  },
}));

vi.mock("../components/cytoscape-view", () => ({
  CytoscapeErView: (props: {
    onBackToNeighborhood?: () => void;
    onBackToSearch?: () => void;
    explorer?: { onSelectHops?: (hops: unknown) => void };
  }) => {
    useEffect(() => {
      cytoscapeHarness.mounted = true;
      return () => {
        cytoscapeHarness.mounted = false;
      };
    }, []);
    return (
      <div data-testid="mock-overview">
        {props.onBackToNeighborhood && (
          <button
            type="button"
            data-testid="er-back-to-neighborhood"
            onClick={props.onBackToNeighborhood}
          >
            Neighborhood
          </button>
        )}
        {props.onBackToSearch && (
          <button type="button" data-testid="er-back-to-search" onClick={props.onBackToSearch}>
            Search
          </button>
        )}
        {/* Expose hops control for testing hops isolation */}
        {props.explorer?.onSelectHops && (
          <button
            type="button"
            data-testid="er-mock-set-hops-domain"
            onClick={() => props.explorer!.onSelectHops!("domain")}
          >
            Set Domain
          </button>
        )}
      </div>
    );
  },
}));

// Layout harness — captures inputs to verify the layout pipeline switches
// from overview (full-schema) back to react-flow (bounded neighborhood).
const layoutHarness = vi.hoisted(() => ({
  inputs: [] as (unknown | null)[],
  profiles: [] as string[],
  reset() {
    this.inputs = [];
    this.profiles = [];
  },
}));

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: (input: unknown, options?: { profile?: string }) => {
    layoutHarness.inputs.push(input);
    // Only track profile for non-null inputs (actual layout work).
    // With two separate hook instances (#46 E3), both are called on every
    // render, but only the active one receives non-null input.
    if (input !== null && options?.profile) layoutHarness.profiles.push(options.profile);
    return {
      status: "ready",
      positions: new Map(),
      layoutMs: 5,
      nodeCount: 0,
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

/* ── Helpers ─────────────────────────────────────────────────────────────── */

async function enterNeighborhood(container: HTMLElement) {
  const searchInput = screen.getByTestId("er-search-input");
  fireEvent.change(searchInput, { target: { value: "app" } });
  await waitFor(() => {
    expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
  });
  fireEvent.click(screen.getAllByTestId("er-search-result")[0]);
  await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
}

async function enterOverview() {
  await waitFor(() => {
    expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
  });
  act(() => {
    fireEvent.click(screen.getByTestId("er-show-all-tables"));
  });
  await waitFor(() => {
    expect(cytoscapeHarness.mounted).toBe(true);
  });
}

/** Extract sorted node IDs from React Flow nodes. */
function getNodeIds(container: HTMLElement): string[] {
  const nodes = container.querySelectorAll(".react-flow__node");
  return [...nodes]
    .map((n) => n.getAttribute("data-id") || "")
    .filter(Boolean)
    .sort();
}

/* ── Tests ───────────────────────────────────────────────────────────────── */

describe("#46 E3 — back navigation contract", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  /* Back buttons appear ONLY in overview phase */

  it("back buttons absent in neighborhood, present in overview", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood — back buttons should NOT exist
    await enterNeighborhood(container);
    expect(screen.queryByTestId("er-back-to-neighborhood")).toBeNull();
    expect(screen.queryByTestId("er-back-to-search")).toBeNull();

    // Enter overview — both back buttons should appear
    await enterOverview();
    expect(screen.getByTestId("er-back-to-neighborhood")).toBeInTheDocument();
    expect(screen.getByTestId("er-back-to-search")).toBeInTheDocument();
  });

  /* Clicking Back to Neighborhood returns to neighborhood with exact same IDs */

  it("back to neighborhood preserves exact node IDs", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // search → neighborhood
    await enterNeighborhood(container);
    const idsBeforeOverview = getNodeIds(container);
    expect(idsBeforeOverview.length).toBeGreaterThan(0);
    expect(idsBeforeOverview.length).toBeLessThanOrEqual(100);

    // neighborhood → overview
    await enterOverview();
    expect(container.querySelector(".react-flow")).toBeNull();

    // overview → neighborhood
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-neighborhood"));
    });

    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });

    // Exact same node IDs (same seed + same hops = same BFS)
    const idsAfterBack = getNodeIds(container);
    expect(idsAfterBack).toEqual(idsBeforeOverview);
  });

  /* Clicking Back to Search returns to search phase */

  it("back to search resets to zero nodes and null layout", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // search → neighborhood → overview
    await enterNeighborhood(container);
    await enterOverview();

    // Clear harness to track only post-back inputs
    layoutHarness.reset();

    // overview → search
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-search"));
    });

    // Cytoscape unmounts, search entry remounts
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(false);
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });

    // No React Flow nodes
    expect(container.querySelector(".react-flow__node")).toBeNull();

    // All layout inputs should be null (search phase = no layout)
    expect(layoutHarness.inputs.length).toBeGreaterThan(0);
    expect(layoutHarness.inputs.every((inp) => inp === null)).toBe(true);
  });

  /* Repeated cycles are deterministic */

  it("repeated neighborhood → overview → neighborhood cycles preserve exact IDs", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // First cycle
    await enterNeighborhood(container);
    const idsCycle1 = getNodeIds(container);

    await enterOverview();
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-neighborhood"));
    });
    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });
    const idsAfterCycle1 = getNodeIds(container);
    expect(idsAfterCycle1).toEqual(idsCycle1);

    // Second cycle
    await enterOverview();
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-neighborhood"));
    });
    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });
    const idsAfterCycle2 = getNodeIds(container);
    expect(idsAfterCycle2).toEqual(idsCycle1);
  });

  /* No stale overview state flows to React Flow */

  it("no stale overview layout input after back navigation", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    await enterOverview();

    // Clear the harness to track only post-back inputs
    layoutHarness.reset();

    // Click Back
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-neighborhood"));
    });

    // Wait for React Flow
    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });

    // All non-null layout inputs after back should have ≤100 nodes (neighborhood)
    const inputsAfterBack = nonNullInputs() as { nodes: unknown[] }[];
    for (const inp of inputsAfterBack) {
      expect(inp.nodes.length).toBeLessThanOrEqual(100);
    }

    // No "overview" profile should appear after back
    expect(layoutHarness.profiles).not.toContain("overview");
  });

  /* Hops isolation: changing overview highlight hops doesn't affect neighborhood */

  it("changing overview highlight hops then back preserves exact neighborhood IDs", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood and capture exact IDs
    await enterNeighborhood(container);
    const idsBeforeOverview = getNodeIds(container);

    // Enter overview
    await enterOverview();

    // Change highlight hops to "domain" (simulates user clicking Domain in overview explorer)
    await waitFor(() => {
      expect(screen.getByTestId("er-mock-set-hops-domain")).toBeInTheDocument();
    });
    act(() => {
      fireEvent.click(screen.getByTestId("er-mock-set-hops-domain"));
    });

    // Back to neighborhood
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-neighborhood"));
    });

    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });

    // Exact same IDs — neighborhood hops were NOT mutated by overview
    const idsAfterBack = getNodeIds(container);
    expect(idsAfterBack).toEqual(idsBeforeOverview);
  });

  /* Identity change: overview → connection/schema change → search */

  it("overview → identity change resets to search with null layout", async () => {
    const data = generateErFixture(500, 42);
    const { container, rerender } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood → overview
    await enterNeighborhood(container);
    await enterOverview();

    // Clear harness to track post-identity-change inputs
    layoutHarness.reset();

    // Simulate identity change (different connectionId)
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="different" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Should reset to search phase
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(false);
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });

    // No React Flow nodes
    expect(container.querySelector(".react-flow__node")).toBeNull();

    // All layout inputs should be null (search phase)
    expect(layoutHarness.inputs.length).toBeGreaterThan(0);
    expect(layoutHarness.inputs.every((inp) => inp === null)).toBe(true);
  });

  /* Stale layout regression: RF never consumes overview positions */

  it("RF never consumes overview positions during Back transition", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);
    await enterOverview();

    // Simulate overview positions existing (500 nodes worth)
    // The layout harness captures inputs — verify overview had full-schema input
    const overviewInputs = layoutHarness.inputs.filter(
      (inp) => inp !== null && (inp as { nodes: unknown[] }).nodes.length === 500,
    );
    expect(overviewInputs.length).toBeGreaterThan(0);

    // Clear harness to track only post-back inputs
    layoutHarness.reset();

    // Click Back — RF starts computing
    act(() => {
      fireEvent.click(screen.getByTestId("er-back-to-neighborhood"));
    });

    await waitFor(() => {
      expect(container.querySelector(".react-flow")).not.toBeNull();
    });

    // All RF layout inputs after back must have ≤100 nodes (bounded neighborhood)
    // No overview-sized (500-node) inputs should appear
    const rfInputsAfterBack = nonNullInputs() as { nodes: unknown[] }[];
    for (const inp of rfInputsAfterBack) {
      expect(inp.nodes.length).toBeLessThanOrEqual(100);
      expect(inp.nodes.length).not.toBe(500);
    }
  });
});

/** Return only the non-null layout inputs (actual layout requests). */
function nonNullInputs() {
  return layoutHarness.inputs.filter((inp) => inp !== null);
}
