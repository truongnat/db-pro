import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { generateErFixture } from "./er-fixture";

/**
 * #45 E2 — Activate full Cytoscape renderer and full-schema layout
 *           only in overview phase.
 *
 * Contract:
 *   search       → Cytoscape=0, layout input=null
 *   neighborhood → Cytoscape=0, layout input=rf (bounded ≤100)
 *   overview     → Cytoscape=mounted, layout input=overview (full schema)
 *
 * The predicate change:
 *   OLD: activeCytoscape = isLargeSchema && !isNeighborhoodPhase
 *   NEW: activeCytoscape = isOverviewPhase (explicit phase check)
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

// Controllable layout harness — captures every input + options sent by the
// component so tests can assert on the layout pipeline routing per phase.
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

// Spy on buildLayoutInputFromModel — the expensive full-schema overview
// builder. Must be called 0 times in search/neighborhood, ≥1 in overview.
const builderSpy = vi.hoisted(() => {
  const calls: unknown[][] = [];
  return {
    calls,
    wrapper: ((...args: unknown[]) => {
      calls.push(args);
      // Delegate to the real implementation (set by the mock factory)
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

/** Return only the non-null layout inputs (actual layout requests). */
function nonNullInputs() {
  return layoutHarness.inputs.filter((inp) => inp !== null);
}

/* ── Tests ───────────────────────────────────────────────────────────────── */

describe("#45 E2 — overview activation contract", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    builderSpy.calls.length = 0;
    localStorage.clear();
  });

  /* search: Cytoscape=0, layout input=null */

  it("search phase: no Cytoscape, all layout inputs are null", async () => {
    const data = generateErFixture(500, 42);
    render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

    // Cytoscape must NOT mount in search
    expect(cytoscapeHarness.mounted).toBe(false);

    // All layout inputs must be null (no layout work in search phase)
    expect(layoutHarness.inputs.length).toBeGreaterThan(0);
    expect(layoutHarness.inputs.every((inp) => inp === null)).toBe(true);

    // No overview profile should appear
    expect(layoutHarness.profiles).not.toContain("overview");

    // Full-schema overview builder must NOT be called in search
    expect(builderSpy.calls.length).toBe(0);
  });

  /* neighborhood: Cytoscape=0, layout uses react-flow profile */

  it("neighborhood phase: no Cytoscape, layout uses react-flow profile", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // Cytoscape must NOT mount in neighborhood
    expect(cytoscapeHarness.mounted).toBe(false);

    // Layout inputs after entering neighborhood: at least one non-null input
    // (the rfLayoutInput for the bounded ≤100 graph)
    const rfInputs = nonNullInputs();
    expect(rfInputs.length).toBeGreaterThan(0);

    // Profile must be react-flow, NOT overview
    expect(layoutHarness.profiles).toContain("react-flow");
    expect(layoutHarness.profiles).not.toContain("overview");

    // Full-schema overview builder must NOT be called in neighborhood
    expect(builderSpy.calls.length).toBe(0);

    // Neighborhood layout input has ≤100 nodes (bounded neighborhood)
    const boundedInputs = nonNullInputs() as { nodes: unknown[] }[];
    for (const inp of boundedInputs) {
      expect(inp.nodes.length).toBeLessThanOrEqual(100);
    }
  });

  /* overview: Cytoscape=mounted, layout uses overview profile */

  it("overview phase: Cytoscape mounts, layout uses overview profile", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // search → neighborhood → overview
    await enterNeighborhood(container);
    await waitFor(() => {
      expect(screen.getByTestId("er-show-all-tables")).toBeInTheDocument();
    });

    // Capture state before overview transition
    const profilesBeforeOverview = [...layoutHarness.profiles];

    // Click Show all → overview
    act(() => {
      fireEvent.click(screen.getByTestId("er-show-all-tables"));
    });

    // Overview: Cytoscape mounts
    await waitFor(() => {
      expect(cytoscapeHarness.mounted).toBe(true);
    });

    // React Flow neighborhood is gone
    expect(container.querySelector(".react-flow")).toBeNull();

    // Overview layout profile must now appear (was NOT present before)
    expect(layoutHarness.profiles).toContain("overview");
    // The overview profile was NOT used before the transition
    expect(profilesBeforeOverview).not.toContain("overview");

    // Non-null layout input exists for the overview (full-schema geometry)
    const inputsAfterClick = layoutHarness.inputs.filter((inp) => inp !== null) as {
      nodes: unknown[];
    }[];
    expect(inputsAfterClick.length).toBeGreaterThan(0);

    // Full-schema overview builder was called ≥1 time
    expect(builderSpy.calls.length).toBeGreaterThanOrEqual(1);

    // Overview layout input has exactly 500 nodes (full schema, not bounded)
    const overviewInput = inputsAfterClick.find((inp) => inp.nodes.length === 500);
    expect(overviewInput, "overview layout input must have 500 nodes").toBeDefined();
  });

  /* Negative: search selection alone never activates overview */

  it("search selection → neighborhood only, never overview", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    await enterNeighborhood(container);

    // After selection: neighborhood, NOT overview
    expect(cytoscapeHarness.mounted).toBe(false);
    expect(container.querySelector(".react-flow")).not.toBeNull();
    expect(layoutHarness.profiles).not.toContain("overview");
  });
});
