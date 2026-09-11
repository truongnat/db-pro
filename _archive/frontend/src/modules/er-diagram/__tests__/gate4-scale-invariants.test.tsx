import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { generateErFixture } from "./er-fixture";

/**
 * #47 F1 — Gate 4 scale invariants for 201 / 500 / 1000 table schemas.
 *
 * Proves the same three-phase contract holds at every scale the large-schema
 * flow supports. No production changes — verification only.
 *
 * Contract per scale:
 * | Phase        | Evidence                                                                    |
 * |--------------|-----------------------------------------------------------------------------|
 * | Search       | renderer=0, nodes=0, edges=0, builder=0, worker inputs=null, Cytoscape=false|
 * | Neighborhood | 1..100 nodes, rendered IDs = RF input IDs, no edge→hidden node, builder=0   |
 * | First paint  | detail nodes = 0                                                            |
 * | Overview     | only via Show All, Cytoscape=true, overview worker nodes = full fixture count|
 */

/* ── Mocks ───────────────────────────────────────────────────────────────── */

const cytoscapeHarness = vi.hoisted(() => ({
  mounted: false,
  reset() {
    this.mounted = false;
  },
}));

vi.mock("../components/cytoscape-view", () => ({
  CytoscapeErView: (props: { onBackToNeighborhood?: () => void; onBackToSearch?: () => void }) => {
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
      </div>
    );
  },
}));

// Layout harness — captures inputs per profile (two hook instances since #46).
const layoutHarness = vi.hoisted(() => ({
  rfInputs: [] as (unknown | null)[],
  overviewInputs: [] as (unknown | null)[],
  reset() {
    this.rfInputs = [];
    this.overviewInputs = [];
  },
}));

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: (input: unknown, options?: { profile?: string }) => {
    if (options?.profile === "overview") {
      layoutHarness.overviewInputs.push(input);
    } else {
      layoutHarness.rfInputs.push(input);
    }
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

// Builder spy — tracks calls to buildLayoutInputFromModel (overview builder).
const builderSpy = vi.hoisted(() => {
  const calls: unknown[][] = [];
  return {
    calls,
    wrapper: ((...args: unknown[]) => {
      calls.push(args);
      return (builderSpy as any)._real(...args);
    }) as typeof import("../utils/layout-profile").buildLayoutInputFromModel,
    _real: null as ((...args: unknown[]) => unknown) | null,
  };
});

vi.mock("../utils/layout-profile", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../utils/layout-profile")>();
  builderSpy._real = actual.buildLayoutInputFromModel as unknown as (...args: unknown[]) => unknown;
  return {
    ...actual,
    buildLayoutInputFromModel: builderSpy.wrapper,
  };
});

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
function getRenderedNodeIds(container: HTMLElement): string[] {
  const nodes = container.querySelectorAll(".react-flow__node");
  return [...nodes]
    .map((n) => n.getAttribute("data-id") || "")
    .filter(Boolean)
    .sort();
}

/** Get the last non-null RF layout input. */
function getLastRfInput(): {
  nodes: Array<{ id: string }>;
  edges: Array<{ source: string; target: string }>;
} | null {
  for (let i = layoutHarness.rfInputs.length - 1; i >= 0; i--) {
    const inp = layoutHarness.rfInputs[i];
    if (inp !== null)
      return inp as {
        nodes: Array<{ id: string }>;
        edges: Array<{ source: string; target: string }>;
      };
  }
  return null;
}

/** Get the last non-null RF layout input's node IDs. */
function getLastRfInputNodeIds(): string[] | null {
  const input = getLastRfInput();
  return input ? input.nodes.map((n) => n.id).sort() : null;
}

/* ── Parameterized test suite ────────────────────────────────────────────── */

const SCALE_MATRIX = [
  { tables: 201, label: "201 tables (L tier threshold)" },
  { tables: 500, label: "500 tables (L tier)" },
  { tables: 1000, label: "1000 tables (XL tier)" },
];

describe.each(SCALE_MATRIX)("#47 F1 — Gate 4 scale invariants: $label", ({ tables }) => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    builderSpy.calls.length = 0;
    localStorage.clear();
  });

  /* ── Search phase ─────────────────────────────────────────────────────── */

  describe("search phase", () => {
    it("renderer=0, nodes=0, edges=0, builder=0, layout=null, Cytoscape=false", async () => {
      const data = generateErFixture(tables, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
        </TooltipProvider>,
      );

      await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

      // No React Flow renderer
      expect(container.querySelector(".react-flow")).toBeNull();

      // No nodes or edges
      expect(container.querySelectorAll(".react-flow__node").length).toBe(0);
      expect(container.querySelectorAll(".react-flow__edge").length).toBe(0);

      // Cytoscape not mounted
      expect(cytoscapeHarness.mounted).toBe(false);

      // Builder not called
      expect(builderSpy.calls.length).toBe(0);

      // All RF layout inputs are null (search phase = no layout)
      expect(layoutHarness.rfInputs.length).toBeGreaterThan(0);
      expect(layoutHarness.rfInputs.every((inp) => inp === null)).toBe(true);

      // No overview layout inputs either
      expect(layoutHarness.overviewInputs.every((inp) => inp === null)).toBe(true);
    });
  });

  /* ── Neighborhood phase ───────────────────────────────────────────────── */

  describe("neighborhood phase", () => {
    it("1..100 nodes, rendered IDs = RF input IDs, no edge→hidden, builder=0", async () => {
      const data = generateErFixture(tables, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
        </TooltipProvider>,
      );

      await enterNeighborhood(container);

      // Node count: 1..100 (bounded neighborhood)
      const renderedIds = getRenderedNodeIds(container);
      expect(renderedIds.length).toBeGreaterThan(0);
      expect(renderedIds.length).toBeLessThanOrEqual(100);

      // Rendered IDs must match RF worker input IDs (exact match)
      const rfInputIds = getLastRfInputNodeIds();
      expect(rfInputIds).not.toBeNull();
      expect(renderedIds).toEqual(rfInputIds!);

      // No edge points to a hidden node — use captured RF layout input edges
      // (not DOM attributes, which may not exist). Assert non-vacuous: edges > 0.
      const rfInput = getLastRfInput();
      expect(rfInput).not.toBeNull();
      expect(rfInput!.edges.length).toBeGreaterThan(0);
      const visibleSet = new Set(renderedIds);
      for (const edge of rfInput!.edges) {
        expect(visibleSet.has(edge.source), `edge source ${edge.source} not in visible set`).toBe(
          true,
        );
        expect(visibleSet.has(edge.target), `edge target ${edge.target} not in visible set`).toBe(
          true,
        );
      }

      // Cytoscape not mounted
      expect(cytoscapeHarness.mounted).toBe(false);

      // Builder not called (overview builder only runs in overview phase)
      expect(builderSpy.calls.length).toBe(0);

      // No overview layout inputs
      expect(layoutHarness.overviewInputs.every((inp) => inp === null)).toBe(true);
    });

    it("first paint: detail nodes = 0, compact/base nodes exist", async () => {
      const data = generateErFixture(tables, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
        </TooltipProvider>,
      );

      await enterNeighborhood(container);

      // No nodes should have detail tier on first paint
      // Production uses data-tier="3" for detail (lodTier("detail") = 3)
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);

      // Compact/base nodes must exist (proves nodes actually rendered)
      const compactNodes = container.querySelectorAll('[data-tier="1"]');
      expect(compactNodes.length).toBeGreaterThan(0);
    });

    it("explicit focus: exactly 1 detail node, Cytoscape still false", async () => {
      const data = generateErFixture(tables, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
        </TooltipProvider>,
      );

      await enterNeighborhood(container);

      // Before focus: zero detail nodes
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);

      // Click a node → FOCUS_NODE dispatch → that node hydrates to detail
      const rfNodes = container.querySelectorAll(".react-flow__node");
      expect(rfNodes.length).toBeGreaterThan(0);
      const targetNode = rfNodes[0];
      const targetId = targetNode.getAttribute("data-id");
      fireEvent.click(targetNode);

      // After focus: exactly 1 detail node
      await waitFor(() => {
        const detailNodes = container.querySelectorAll('[data-tier="3"]');
        expect(detailNodes.length).toBe(1);
      });

      // The detail node is the one we clicked
      const detailNode = container.querySelector('[data-tier="3"]');
      expect(detailNode?.closest(".react-flow__node")?.getAttribute("data-id")).toBe(targetId);

      // Cytoscape still NOT mounted (focus doesn't trigger overview)
      expect(cytoscapeHarness.mounted).toBe(false);

      // Other nodes remain at compact tier
      const compactNodes = container.querySelectorAll('[data-tier="1"]');
      expect(compactNodes.length).toBeGreaterThan(0);
    });
  });

  /* ── Overview phase ───────────────────────────────────────────────────── */

  describe("overview phase", () => {
    it("only via Show All, Cytoscape=true, overview worker nodes = full count", async () => {
      const data = generateErFixture(tables, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
        </TooltipProvider>,
      );

      // Enter neighborhood first
      await enterNeighborhood(container);

      // Before Show All: no Cytoscape, builder not called
      expect(cytoscapeHarness.mounted).toBe(false);
      expect(builderSpy.calls.length).toBe(0);

      // Enter overview via Show All
      await enterOverview();

      // Cytoscape mounted
      expect(cytoscapeHarness.mounted).toBe(true);

      // React Flow gone
      expect(container.querySelector(".react-flow")).toBeNull();

      // Builder called ≥1 time
      expect(builderSpy.calls.length).toBeGreaterThanOrEqual(1);

      // Overview layout input has exactly `tables` nodes (full schema)
      const overviewInputs = layoutHarness.overviewInputs.filter((inp) => inp !== null) as Array<{
        nodes: unknown[];
      }>;
      expect(overviewInputs.length).toBeGreaterThan(0);

      // At least one input must have the full fixture count
      const fullInput = overviewInputs.find((inp) => inp.nodes.length === tables);
      expect(
        fullInput,
        `overview layout input must have ${tables} nodes (got ${overviewInputs.map((i) => i.nodes.length).join(", ")})`,
      ).toBeDefined();
    });
  });
});
