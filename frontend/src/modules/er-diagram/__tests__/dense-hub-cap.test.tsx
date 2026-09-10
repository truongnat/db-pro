import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  getBoundedNeighborhood,
  deriveNeighborhoodVisibleSet,
  NEIGHBORHOOD_NODE_CAP,
  type LargeSchemaState,
} from "../utils/large-schema";
import { buildAdjacencyMap } from "../utils/neighborhood";
import type {
  IntrospectResult,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

/**
 * #48 F2 — Dense-hub cap + determinism verification.
 *
 * Proves that when a seed has >100 reachable neighbors:
 * 1. Hard cap = 100 (NEIGHBORHOOD_NODE_CAP)
 * 2. Seed is always included
 * 3. truncated = true
 * 4. Ordered IDs are deterministic regardless of FK input order
 *
 * Component contract for dense hub:
 * - Rendered RF IDs == worker/layout IDs (exact ordered comparison)
 * - Edges non-empty, all edge endpoints in visible set
 * - Cytoscape = false, overview builder = 0, overview layout = null
 */

/* ── Dense hub fixture ──────────────────────────────────────────────────── */

/**
 * Build a dense-hub IntrospectResult: one hub table connected to `spokeCount`
 * spoke tables, plus `fillerCount` isolated filler tables.
 *
 * - Hub: "hub" (10 cols, no FKs)
 * - Spokes: "hub_spoke_XXX" (5 cols, FK → hub)
 * - Fillers: "filler_XXX" (5 cols, no FKs, isolated)
 *
 * Total tables = 1 + spokeCount + fillerCount.
 * BFS from hub at hops=2 reaches hub + all spokes (fillers are isolated).
 */
function generateDenseHubFixture(spokeCount: number, fillerCount: number): IntrospectResult {
  const schema = "public";
  const tables: IntrospectResult["tables"] = [];
  const columns: IntrospectResult["columns"] = [];
  const primaryKeys: IntrospectResult["primaryKeys"] = [];
  const foreignKeys: IntrospectResult["foreignKeys"] = [];
  const indexes: IntrospectResult["indexes"] = [];

  // Hub table — 10 columns, no FKs
  tables.push({ name: "hub", schema, rowCount: 10000 });
  for (let c = 0; c < 10; c++) {
    columns.push({
      name: c === 0 ? "id" : `col_${c}`,
      dataType: c === 0 ? "serial" : "text",
      nullable: c === 0 ? false : true,
      defaultValue: c === 0 ? "nextval(...)" : null,
      isPrimaryKey: c === 0,
      tableName: "hub",
      schema,
    });
  }
  primaryKeys.push({
    constraintName: "hub_pkey",
    columns: ["id"],
    tableName: "hub",
    schema,
  });
  indexes.push({
    name: "hub_pkey",
    columns: ["id"],
    unique: true,
    tableName: "hub",
    schema,
  });

  // Spoke tables — each with FK → hub
  for (let i = 0; i < spokeCount; i++) {
    const name = `hub_spoke_${String(i).padStart(3, "0")}`;
    tables.push({ name, schema, rowCount: 100 });

    for (let c = 0; c < 5; c++) {
      columns.push({
        name: c === 0 ? "id" : `col_${c}`,
        dataType: c === 0 ? "serial" : "text",
        nullable: c === 0 ? false : true,
        defaultValue: c === 0 ? "nextval(...)" : null,
        isPrimaryKey: c === 0,
        tableName: name,
        schema,
      });
    }
    primaryKeys.push({
      constraintName: `${name}_pkey`,
      columns: ["id"],
      tableName: name,
      schema,
    });
    indexes.push({
      name: `${name}_pkey`,
      columns: ["id"],
      unique: true,
      tableName: name,
      schema,
    });

    foreignKeys.push({
      name: `fk_${name}_hub`,
      fromTable: name,
      fromColumns: ["col_1"],
      toTable: "hub",
      toColumns: ["id"],
      schema,
      toSchema: schema,
    });
    indexes.push({
      name: `idx_${name}_col_1`,
      columns: ["col_1"],
      unique: false,
      tableName: name,
      schema,
    });
  }

  // Filler tables — isolated, no FKs (ensures tableCount > 200 for large-schema)
  for (let i = 0; i < fillerCount; i++) {
    const name = `filler_${String(i).padStart(3, "0")}`;
    tables.push({ name, schema, rowCount: 50 });

    for (let c = 0; c < 5; c++) {
      columns.push({
        name: c === 0 ? "id" : `col_${c}`,
        dataType: c === 0 ? "serial" : "text",
        nullable: c === 0 ? false : true,
        defaultValue: c === 0 ? "nextval(...)" : null,
        isPrimaryKey: c === 0,
        tableName: name,
        schema,
      });
    }
    primaryKeys.push({
      constraintName: `${name}_pkey`,
      columns: ["id"],
      tableName: name,
      schema,
    });
    indexes.push({
      name: `${name}_pkey`,
      columns: ["id"],
      unique: true,
      tableName: name,
      schema,
    });
  }

  return {
    schemas: [{ name: "public" }],
    tables,
    columns,
    primaryKeys,
    indexes,
    foreignKeys,
    views: [],
    triggers: [],
  };
}

/** Reverse an FK array (same relations, different input order). */
function reverseFks(fks: SchemaForeignKeyDto[]): SchemaForeignKeyDto[] {
  return [...fks].reverse();
}

/** Deterministic shuffle (Fisher–Yates with fixed seed). */
function shuffleFks(fks: SchemaForeignKeyDto[]): SchemaForeignKeyDto[] {
  const result = [...fks];
  let s = 12345;
  for (let i = result.length - 1; i > 0; i--) {
    s = (s * 16807) % 2147483647;
    const j = s % (i + 1);
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

/** Deterministic shuffle for tables (Fisher–Yates with fixed seed). */
function shuffleTables(tables: TableDto[]): TableDto[] {
  const result = [...tables];
  let s = 12345;
  for (let i = result.length - 1; i > 0; i--) {
    s = (s * 16807) % 2147483647;
    const j = s % (i + 1);
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

/** Rebuild with both tables and FKs reordered. */
function withReorderedTablesAndFks(
  data: IntrospectResult,
  newTables: TableDto[],
  newFks: SchemaForeignKeyDto[],
): IntrospectResult {
  return { ...data, tables: newTables, foreignKeys: newFks };
}

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

// Layout harness — captures RF inputs per profile.
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

// Builder spy — tracks buildLayoutInputFromModel calls.
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

/** Enter neighborhood by searching for "hub" and clicking the hub result.
 *  Must find the exact "hub" label, not a spoke containing "hub". */
async function enterNeighborhoodViaSearch(container: HTMLElement) {
  const searchInput = screen.getByTestId("er-search-input");
  fireEvent.change(searchInput, { target: { value: "hub" } });
  await waitFor(() => {
    expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
  });
  // Find the result whose label is exactly "hub" (not "hub_spoke_XXX").
  // textContent includes metadata ("hub 10 cols · 0 FK"), so match by startsWith + no underscore.
  const results = screen.getAllByTestId("er-search-result");
  const hubResult = results.find(
    (el) => el.textContent?.startsWith("hub") && !el.textContent?.includes("_"),
  );
  expect(hubResult, "hub table must appear in search results").toBeDefined();
  fireEvent.click(hubResult!);
  await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());
}

/** Get rendered RF node IDs from the DOM (raw DOM order, no sorting). */
function getRenderedNodeIds(container: HTMLElement): string[] {
  const nodes = container.querySelectorAll(".react-flow__node");
  return [...nodes].map((n) => n.getAttribute("data-id") || "").filter(Boolean);
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

/** Get the last non-null RF layout input's node IDs (raw pipeline order). */
function getLastRfInputNodeIds(): string[] | null {
  const input = getLastRfInput();
  return input ? input.nodes.map((n) => n.id) : null;
}

/* ── Pure logic tests ───────────────────────────────────────────────────── */

describe("#48 F2 — dense-hub pure logic", () => {
  const SPOKE_COUNT = 200;
  const FILLER_COUNT = 50;

  it("hard cap at NEIGHBORHOOD_NODE_CAP with truncated=true", () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const adj = buildAdjacencyMap(data.foreignKeys);

    const { nodes, truncated } = getBoundedNeighborhood(
      adj,
      "public.hub",
      2,
      NEIGHBORHOOD_NODE_CAP,
    );

    expect(nodes.size).toBe(NEIGHBORHOOD_NODE_CAP);
    expect(truncated).toBe(true);
  });

  it("seed always included in capped result", () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const adj = buildAdjacencyMap(data.foreignKeys);

    const { nodes } = getBoundedNeighborhood(adj, "public.hub", 2, NEIGHBORHOOD_NODE_CAP);

    expect(nodes.has("public.hub")).toBe(true);
  });

  it("canonical ordered IDs deterministic regardless of input order", () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const seed = "public.hub";
    const hops = 2;
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: seed,
      focusedNodeId: null,
    };

    // Original order
    const adjOriginal = buildAdjacencyMap(data.foreignKeys);
    const keysOriginal = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    const resultOriginal = deriveNeighborhoodVisibleSet(state, adjOriginal, hops, keysOriginal);

    // Reversed FK order
    const adjReversed = buildAdjacencyMap(reverseFks(data.foreignKeys));
    const resultReversed = deriveNeighborhoodVisibleSet(state, adjReversed, hops, keysOriginal);

    // Shuffled FK order
    const adjShuffled = buildAdjacencyMap(shuffleFks(data.foreignKeys));
    const resultShuffled = deriveNeighborhoodVisibleSet(state, adjShuffled, hops, keysOriginal);

    // deriveNeighborhoodVisibleSet returns canonical sorted tableIds.
    // Compare raw — no additional .sort() on top.
    expect(resultOriginal.tableIds).toEqual(resultReversed.tableIds);
    expect(resultOriginal.tableIds).toEqual(resultShuffled.tableIds);

    // Sanity: actually truncated, not vacuously equal
    expect(resultOriginal.truncated).toBe(true);
    expect(resultOriginal.tableIds.length).toBe(NEIGHBORHOOD_NODE_CAP);
  });

  it("canonical ordered IDs deterministic with reversed/shuffled tables + FKs", () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const seed = "public.hub";
    const hops = 2;
    const state: LargeSchemaState = {
      phase: "neighborhood",
      seedTable: seed,
      focusedNodeId: null,
    };

    // Original order
    const adjOriginal = buildAdjacencyMap(data.foreignKeys);
    const keysOriginal = new Set(data.tables.map((t) => `${t.schema}.${t.name}`));
    const resultOriginal = deriveNeighborhoodVisibleSet(state, adjOriginal, hops, keysOriginal);

    // Reversed tables + reversed FKs
    const reversedData = withReorderedTablesAndFks(
      data,
      [...data.tables].reverse(),
      reverseFks(data.foreignKeys),
    );
    const adjReversed = buildAdjacencyMap(reversedData.foreignKeys);
    const keysReversed = new Set(reversedData.tables.map((t) => `${t.schema}.${t.name}`));
    const resultReversed = deriveNeighborhoodVisibleSet(state, adjReversed, hops, keysReversed);

    // Shuffled tables + shuffled FKs
    const shuffledData = withReorderedTablesAndFks(
      data,
      shuffleTables(data.tables),
      shuffleFks(data.foreignKeys),
    );
    const adjShuffled = buildAdjacencyMap(shuffledData.foreignKeys);
    const keysShuffled = new Set(shuffledData.tables.map((t) => `${t.schema}.${t.name}`));
    const resultShuffled = deriveNeighborhoodVisibleSet(state, adjShuffled, hops, keysShuffled);

    // All canonical ordered outputs must match exactly
    expect(resultOriginal.tableIds).toEqual(resultReversed.tableIds);
    expect(resultOriginal.tableIds).toEqual(resultShuffled.tableIds);

    // Sanity
    expect(resultOriginal.truncated).toBe(true);
    expect(resultOriginal.tableIds.length).toBe(NEIGHBORHOOD_NODE_CAP);
  });
});

/* ── Component contract test ────────────────────────────────────────────── */

describe("#48 F2 — dense-hub component contract", () => {
  const SPOKE_COUNT = 200;
  const FILLER_COUNT = 50;

  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    builderSpy.calls.length = 0;
    localStorage.clear();
  });

  it("dense hub: cap=100, rendered IDs == worker IDs, edges valid, no overview", async () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood via the hub table
    await enterNeighborhoodViaSearch(container);

    // ── Node cap: exactly 100 ──
    const renderedIds = getRenderedNodeIds(container);
    expect(renderedIds.length).toBe(NEIGHBORHOOD_NODE_CAP);

    // ── Seed always included ──
    expect(renderedIds).toContain("public.hub");

    // ── Rendered IDs exactly match RF worker input IDs (ordered comparison) ──
    const rfInputIds = getLastRfInputNodeIds();
    expect(rfInputIds).not.toBeNull();
    expect(renderedIds).toEqual(rfInputIds!);

    // ── Edges non-empty, all endpoints in visible set ──
    const rfInput = getLastRfInput();
    expect(rfInput).not.toBeNull();
    expect(rfInput!.edges.length).toBeGreaterThan(0);
    const visibleSet = new Set(renderedIds);
    for (const edge of rfInput!.edges) {
      expect(visibleSet.has(edge.source), `edge source ${edge.source} not visible`).toBe(true);
      expect(visibleSet.has(edge.target), `edge target ${edge.target} not visible`).toBe(true);
    }

    // ── Cytoscape not mounted ──
    expect(cytoscapeHarness.mounted).toBe(false);

    // ── Overview builder not called ──
    expect(builderSpy.calls.length).toBe(0);

    // ── No overview layout inputs ──
    expect(layoutHarness.overviewInputs.every((inp) => inp === null)).toBe(true);
  });

  it("component determinism: same fixture → same rendered IDs on re-render", async () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);

    // First render
    const { container: c1, unmount } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );
    await enterNeighborhoodViaSearch(c1);
    const ids1 = getRenderedNodeIds(c1);

    unmount();
    layoutHarness.reset();
    cytoscapeHarness.reset();

    // Second render — same fixture, same seed
    const { container: c2 } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );
    await enterNeighborhoodViaSearch(c2);
    const ids2 = getRenderedNodeIds(c2);

    // Exact ordered match — deterministic rendering
    expect(ids1).toEqual(ids2);
    expect(ids1.length).toBe(NEIGHBORHOOD_NODE_CAP);
  });

  it("reversed tables + FKs → same raw ordered rendered IDs", async () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const dataReversed = withReorderedTablesAndFks(
      data,
      [...data.tables].reverse(),
      reverseFks(data.foreignKeys),
    );

    // Original order
    const { container: c1, unmount } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );
    await enterNeighborhoodViaSearch(c1);
    const idsOriginal = getRenderedNodeIds(c1);

    unmount();
    layoutHarness.reset();
    cytoscapeHarness.reset();

    // Reversed tables + reversed FKs
    const { container: c2 } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={dataReversed} />
      </TooltipProvider>,
    );
    await enterNeighborhoodViaSearch(c2);
    const idsReversed = getRenderedNodeIds(c2);

    // Same raw ordered IDs (no .sort()) despite different table + FK input order
    expect(idsOriginal).toEqual(idsReversed);
    expect(idsOriginal.length).toBe(NEIGHBORHOOD_NODE_CAP);
  });

  it("shuffled tables + FKs → same raw ordered rendered IDs", async () => {
    const data = generateDenseHubFixture(SPOKE_COUNT, FILLER_COUNT);
    const dataShuffled = withReorderedTablesAndFks(
      data,
      shuffleTables(data.tables),
      shuffleFks(data.foreignKeys),
    );

    // Original order
    const { container: c1, unmount } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );
    await enterNeighborhoodViaSearch(c1);
    const idsOriginal = getRenderedNodeIds(c1);

    unmount();
    layoutHarness.reset();
    cytoscapeHarness.reset();

    // Shuffled tables + shuffled FKs
    const { container: c2 } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={dataShuffled} />
      </TooltipProvider>,
    );
    await enterNeighborhoodViaSearch(c2);
    const idsShuffled = getRenderedNodeIds(c2);

    // Same raw ordered IDs despite shuffled table + FK input order
    expect(idsOriginal).toEqual(idsShuffled);
    expect(idsOriginal.length).toBe(NEIGHBORHOOD_NODE_CAP);
  });
});
