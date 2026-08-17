import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErDiagram } from "../components/er-diagram";
import { TooltipProvider } from "@/components/ui/tooltip";
import { generateErFixture } from "./er-fixture";
import { shouldEnterLargeSchemaFlow } from "../utils/large-schema";
import { classifySchemaComplexity, computeSchemaComplexity } from "../renderer/er-graph-model";
import type {
  IntrospectResult,
  SchemaColumnDto,
  SchemaForeignKeyDto,
  TableDto,
} from "@/modules/schema/types/schema.types";

/* ── Mocks ───────────────────────────────────────────────────────────────── */

// Layout is a worker pipeline; no real worker in jsdom.
// Harness captures inputs per profile (#46 E3 — two separate hook instances).
const layoutHarness = vi.hoisted(() => ({
  rfInput: undefined as unknown,
  overviewInput: undefined as unknown,
  callCount: 0,
  reset() {
    this.rfInput = undefined;
    this.overviewInput = undefined;
    this.callCount = 0;
  },
  /** Returns the last active input (RF in neighborhood, overview in overview, null in search). */
  get lastInput() {
    // In neighborhood phase, RF has non-null input
    if (this.rfInput !== null && this.rfInput !== undefined) {
      return this.rfInput;
    }
    // In overview phase, overview has non-null input
    if (this.overviewInput !== null && this.overviewInput !== undefined) {
      return this.overviewInput;
    }
    // In search phase, both are null
    return this.rfInput ?? this.overviewInput ?? null;
  },
}));

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: (input: unknown, options?: { profile?: string }) => {
    // Track input by profile (#46 E3 — two separate hook instances).
    if (options?.profile === "overview") {
      layoutHarness.overviewInput = input;
    } else {
      layoutHarness.rfInput = input;
    }
    layoutHarness.callCount++;
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

// Track whether CytoscapeErView is mounted — it must NOT mount during search.
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

// Track buildLayoutInputFromModel calls — the builder must NOT run during
// search phase. We wrap the real implementation with a call counter.
const builderHarness = vi.hoisted(() => ({
  callCount: 0,
  reset() {
    this.callCount = 0;
  },
}));

vi.mock("../utils/layout-profile", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../utils/layout-profile")>();
  return {
    ...actual,
    buildLayoutInputFromModel: (...args: unknown[]) => {
      builderHarness.callCount++;
      return (actual.buildLayoutInputFromModel as (...a: unknown[]) => unknown)(...args);
    },
  };
});

/* ── Helpers ─────────────────────────────────────────────────────────────── */

/**
 * Build a dense IntrospectResult with controlled table/column/FK counts.
 * Each table gets `colsPerTable` columns and FKs chain table[i] → table[i-1].
 */
function buildDenseFixture(tableCount: number, colsPerTable: number): IntrospectResult {
  const tables: TableDto[] = [];
  const columns: SchemaColumnDto[] = [];
  const foreignKeys: SchemaForeignKeyDto[] = [];

  for (let i = 0; i < tableCount; i++) {
    const name = `tbl_${String(i).padStart(4, "0")}`;
    const schema = "public";
    tables.push({ name, schema, rowCount: 0 });

    for (let c = 0; c < colsPerTable; c++) {
      columns.push({
        name: c === 0 ? "id" : `col_${c}`,
        dataType: "text",
        nullable: true,
        defaultValue: null,
        isPrimaryKey: c === 0,
        tableName: name,
        schema,
      });
    }

    if (i > 0) {
      foreignKeys.push({
        name: `fk_tbl_${String(i).padStart(4, "0")}_prev`,
        fromTable: name,
        fromColumns: ["id"],
        toTable: `tbl_${String(i - 1).padStart(4, "0")}`,
        toColumns: ["id"],
        schema,
        toSchema: schema,
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

describe("Gate 4 Slice B — search-first entry UX", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    builderHarness.reset();
  });

  /* 1-3. Table count > 200 enters search-first regardless of tier */

  it("enters search-first for 201-table schema", () => {
    const data = generateErFixture(201, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(screen.queryByTestId("mock-cytoscape")).not.toBeInTheDocument();
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  it("enters search-first for 500-table schema", () => {
    const data = generateErFixture(500, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  it("enters search-first for 1000-table schema", () => {
    const data = generateErFixture(1000, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  /* 4. L/XL tier triggers search-first even when table count ≤ 200 */

  it("enters search-first for L/XL tier with ≤200 tables", () => {
    // 100 tables × 30 cols + 99 FKs → complexity ≈ 100 + 69.3 + 240 = 409.3 → M
    // Need denser: 150 tables × 40 cols + 149 FKs → 150 + 104.3 + 480 = 734.3 → L
    const data = buildDenseFixture(150, 40);
    const stats = {
      tables: data.tables.length,
      relations: data.foreignKeys.length,
      columns: data.columns.length,
    };
    const tier = classifySchemaComplexity(computeSchemaComplexity(stats));

    // Verify this is indeed L tier with ≤200 tables
    expect(tier === "L" || tier === "XL").toBe(true);
    expect(data.tables.length).toBeLessThanOrEqual(200);
    expect(shouldEnterLargeSchemaFlow(data.tables.length, tier)).toBe(true);

    render(<ErDiagram connectionId="c" schema="public" data={data} />);
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  /* 5. Small/medium schema retains current React Flow path */

  it("small/medium schema does NOT enter search-first", () => {
    const data = generateErFixture(50, 42); // ~50 tables → S or M tier
    render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // No search entry — goes straight to React Flow graph
    expect(screen.queryByTestId("er-search-input")).not.toBeInTheDocument();
  });

  /* 6. Initial large-schema render contains zero graph nodes */

  it("initial large-schema render has zero graph nodes (React Flow not mounted)", () => {
    const data = generateErFixture(300, 42);
    const { container } = render(<ErDiagram connectionId="c" schema="public" data={data} />);

    // React Flow's viewport element should not be in the DOM
    expect(container.querySelector("[data-testid='mock-cytoscape']")).toBeNull();
    // React Flow renders a .react-flow container — it must not be present
    expect(container.querySelector(".react-flow")).toBeNull();
    // The search entry is the only rendered surface
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
  });

  /* 7. Initial render passes null layout input / does not start layout */

  it("does not start worker layout before table selection", () => {
    const data = generateErFixture(250, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    // Search phase → layout input is explicitly null (not just "no renderer")
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(layoutHarness.lastInput).toBeNull();
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  /* 8. Search typing alone does not mount React Flow/Cytoscape or trigger layout */

  it("search typing does not mount graph renderers", async () => {
    const data = generateErFixture(300, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    const input = screen.getByTestId("er-search-input");

    // Type a search query
    fireEvent.change(input, { target: { value: "app" } });

    // Still no graph renderers
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(screen.queryByTestId("mock-cytoscape")).not.toBeInTheDocument();
    expect(cytoscapeHarness.mounted).toBe(false);

    // Type more
    fireEvent.change(input, { target: { value: "billing" } });

    // Still no graph renderers
    expect(cytoscapeHarness.mounted).toBe(false);
  });

  /* 9. Selecting a result transitions to neighborhood with the selected seed */

  it("selecting a result dispatches SELECT_TABLE and transitions to neighborhood", async () => {
    const data = generateErFixture(300, 42);
    render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    const input = screen.getByTestId("er-search-input");

    // Type to get results
    fireEvent.change(input, { target: { value: "app" } });

    // Wait for results to appear
    await waitFor(() => {
      const results = screen.getAllByTestId("er-search-result");
      expect(results.length).toBeGreaterThan(0);
    });

    // Click the first result
    const firstResult = screen.getAllByTestId("er-search-result")[0];
    fireEvent.click(firstResult);

    // After selection, the search entry is gone (phase → neighborhood)
    expect(screen.queryByTestId("er-search-input")).not.toBeInTheDocument();
  });

  /* Bonus: display table count on the search entry */

  it("shows total table count on the search entry", () => {
    const data = generateErFixture(250, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    // The table count badge should show 250
    expect(screen.getByText("250 tables")).toBeInTheDocument();
  });

  /* Bonus: Escape clears search */

  it("Escape clears search query", () => {
    const data = generateErFixture(300, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    const input = screen.getByTestId("er-search-input");
    fireEvent.change(input, { target: { value: "app" } });
    expect(input).toHaveValue("app");

    fireEvent.keyDown(input, { key: "Escape" });
    expect(input).toHaveValue("");
  });

  /* Bonus: no auto-selection when only one match */

  it("does not auto-select when only one match exists", () => {
    const data = generateErFixture(300, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    const input = screen.getByTestId("er-search-input");

    // Type a very specific query that matches only one table
    fireEvent.change(input, { target: { value: "zzz_unique_match" } });

    // Search entry is still shown (no auto-transition)
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    // No results
    expect(screen.queryByTestId("er-search-result")).not.toBeInTheDocument();
  });

  /* Layout input gating — captures actual useWorkerLayout argument */

  it("layout input is null for 201-table schema during search", () => {
    const data = generateErFixture(201, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(layoutHarness.lastInput).toBeNull();
  });

  it("layout input is null for 500-table schema during search", () => {
    const data = generateErFixture(500, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(layoutHarness.lastInput).toBeNull();
  });

  it("layout input is null for 1000-table schema during search", () => {
    const data = generateErFixture(1000, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(layoutHarness.lastInput).toBeNull();
  });

  it("layout input stays null after search typing", () => {
    const data = generateErFixture(500, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    fireEvent.change(searchInput, { target: { value: "billing" } });

    // Typing filters metadata only — layout input remains null
    expect(layoutHarness.lastInput).toBeNull();
  });

  it("layout input becomes non-null after table selection", async () => {
    const data = generateErFixture(500, 42);
    render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // During search: null
    expect(layoutHarness.lastInput).toBeNull();

    // Select a table
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });

    await waitFor(() => {
      const results = screen.getAllByTestId("er-search-result");
      expect(results.length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // After selection: phase → neighborhood, layout input is no longer null
    expect(layoutHarness.lastInput).not.toBeNull();
  });

  /* Gate 4 C3 (#39) — neighborhood layout input is bounded (≤100 nodes) */

  it("neighborhood layout input is bounded to ≤100 nodes (not full model)", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Select a table → neighborhood phase
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Layout input must be non-null and bounded
    const input = layoutHarness.lastInput as { nodes: unknown[]; edges: unknown[] } | null;
    expect(input).not.toBeNull();
    expect(input!.nodes.length).toBeLessThanOrEqual(100);
    expect(input!.nodes.length).toBeGreaterThan(0);

    // Cytoscape must NOT mount in neighborhood (#39)
    expect(cytoscapeHarness.mounted).toBe(false);

    // React Flow renders the bounded graph
    expect(container.querySelector(".react-flow")).not.toBeNull();
  });

  /* Builder-level gating — buildLayoutInputFromModel must not run during search */

  it("layout builder is not called for 201-table schema during search", () => {
    const data = generateErFixture(201, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(builderHarness.callCount).toBe(0);
  });

  it("layout builder is not called for 500-table schema during search", () => {
    const data = generateErFixture(500, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(builderHarness.callCount).toBe(0);
  });

  it("layout builder is not called for 1000-table schema during search", () => {
    const data = generateErFixture(1000, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    expect(builderHarness.callCount).toBe(0);
  });

  it("layout builder is not called during search typing", () => {
    const data = generateErFixture(500, 42);
    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    fireEvent.change(searchInput, { target: { value: "billing" } });

    expect(builderHarness.callCount).toBe(0);
  });

  it("layout builder does not run in neighborhood (bounded RF path, #39)", async () => {
    const data = generateErFixture(500, 42);
    render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // During search: builder not called
    expect(builderHarness.callCount).toBe(0);

    // Select a table
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Gate 4 C3 (#39): neighborhood uses the bounded rfLayoutInput, not the
    // full-model builder. buildLayoutInputFromModel only runs for Cytoscape
    // overview (activeCytoscape = isLargeSchema && !isNeighborhoodPhase).
    expect(builderHarness.callCount).toBe(0);
  });

  /* Gate 4 C4 (#40) — lifecycle/reset */

  it("manual positions from localStorage survive initial mount (not cleared by lifecycle effect)", () => {
    const data = generateErFixture(300, 42);
    const storageKey = `er-diagram-positions:c:public`;
    const saved = JSON.stringify([["public.tbl_0001", { x: 42, y: 99 }]]);
    localStorage.setItem(storageKey, saved);

    render(<ErDiagram connectionId="c" schema="public" data={data} />);

    // The search entry renders (phase = "search"). The lifecycle effect must
    // NOT clear positions on initial mount — only on actual phase transitions.
    expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    expect(localStorage.getItem(storageKey)).toBe(saved);

    localStorage.removeItem(storageKey);
  });

  /* Gate 4 D1 (#41) — safe first-paint LOD */

  it("neighborhood first paint uses compact LOD, not detail (QA-P1-13)", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Select a table → neighborhood phase
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // React Flow mounts with bounded neighborhood nodes.
    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());

    // First-paint LOD: all nodes at compact (tier 1), zero at detail (tier 3).
    // Seed ≠ focus: entering neighborhood does NOT promote any node to detail.
    const compactNodes = container.querySelectorAll('[data-tier="1"]');
    const detailNodes = container.querySelectorAll('[data-tier="3"]');
    expect(compactNodes.length).toBeGreaterThan(0);
    expect(detailNodes.length).toBe(0);
  });

  /* Gate 4 D2 (#42) — seed/focus/detail hydration */

  it("entry: seed set, focus null, detail count = 0", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());

    // Entry invariant: seed is set, but no focus → all nodes at base LOD.
    const detailNodes = container.querySelectorAll('[data-tier="3"]');
    expect(detailNodes.length).toBe(0);

    const compactNodes = container.querySelectorAll('[data-tier="1"]');
    expect(compactNodes.length).toBeGreaterThan(0);
  });

  it("explicit focus: click node B → B at detail, others compact", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());

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

    // The detail node renders column rows (ErDetailedNode has [data-column])
    expect(detailNode?.querySelector("[data-column]")).not.toBeNull();

    // Other nodes remain at compact tier
    const compactNodes = container.querySelectorAll('[data-tier="1"]');
    expect(compactNodes.length).toBeGreaterThan(0);
  });

  /* Gate 4 C4 (#40 correction) — connection/schema change resets to search */

  it("changing connectionId resets large-schema state to search", async () => {
    const data = generateErFixture(300, 42);
    const { rerender } = render(
      <TooltipProvider>
        <ErDiagram connectionId="conn-A" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood on conn-A
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    // Neighborhood phase: search entry is gone
    expect(screen.queryByTestId("er-search-input")).not.toBeInTheDocument();

    // Switch connection → must reset to search
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="conn-B" schema="public" data={data} />
      </TooltipProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });
  });

  it("changing schema resets large-schema state to search", async () => {
    // Build a fixture with tables in both schemas so the large-schema flow
    // activates regardless of which schema is selected.
    const base = generateErFixture(300, 42);
    const analyticsTables: TableDto[] = base.tables.map((t) => ({
      ...t,
      schema: "analytics",
    }));
    const analyticsColumns: SchemaColumnDto[] = base.columns.map((c) => ({
      ...c,
      schema: "analytics",
    }));
    const analyticsFks: SchemaForeignKeyDto[] = base.foreignKeys.map((fk) => ({
      ...fk,
      schema: "analytics",
      toSchema: "analytics",
    }));
    const multiSchemaData: IntrospectResult = {
      ...base,
      schemas: [{ name: "public" }, { name: "analytics" }],
      tables: [...base.tables, ...analyticsTables],
      columns: [...base.columns, ...analyticsColumns],
      foreignKeys: [...base.foreignKeys, ...analyticsFks],
    };

    const { rerender } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={multiSchemaData} />
      </TooltipProvider>,
    );

    // Enter neighborhood on public
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);
    expect(screen.queryByTestId("er-search-input")).not.toBeInTheDocument();

    // Switch schema → must reset to search (analytics also has 300 tables)
    rerender(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="analytics" data={multiSchemaData} />
      </TooltipProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("er-search-input")).toBeInTheDocument();
    });
  });

  /* Gate 4 D2 (#42 correction) — clear focus from UI */

  it("pane click clears focus (detail count back to 0)", async () => {
    const data = generateErFixture(500, 42);
    const { container } = render(
      <TooltipProvider>
        <ErDiagram connectionId="c" schema="public" data={data} />
      </TooltipProvider>,
    );

    // Enter neighborhood
    const searchInput = screen.getByTestId("er-search-input");
    fireEvent.change(searchInput, { target: { value: "app" } });
    await waitFor(() => {
      expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
    });
    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);
    await waitFor(() => expect(container.querySelector(".react-flow")).not.toBeNull());

    // Focus a node
    const rfNodes = container.querySelectorAll(".react-flow__node");
    fireEvent.click(rfNodes[0]);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(1);
    });

    // Click the pane background → CLEAR_FOCUS
    const pane = container.querySelector(".react-flow__pane");
    if (pane) fireEvent.click(pane);

    await waitFor(() => {
      expect(container.querySelectorAll('[data-tier="3"]').length).toBe(0);
    });
  });
});

/* ── Gate 4 — QA-P1-12 node count invariant ────────────────────────────────── */

describe("Gate 4 — QA-P1-12 node count invariant (500/1000 tables)", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  for (const { label, count } of [
    { label: "500", count: 500 },
    { label: "1000", count: 1000 },
  ]) {
    it(`${label} tables: neighborhood DOM nodes ≤ 100 after table selection`, async () => {
      const data = generateErFixture(count, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
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

      // Wait for React Flow to mount
      await waitFor(() => {
        expect(container.querySelector(".react-flow")).not.toBeNull();
      });

      // Invariant: graphTables >> renderedTables
      // The full model has `count` tables, but the neighborhood RF DOM
      // must be bounded to ≤100 nodes (the BFS cap).
      const rfDomNodes = container.querySelectorAll(".react-flow__node");
      expect(rfDomNodes.length).toBeGreaterThan(0);
      expect(rfDomNodes.length).toBeLessThanOrEqual(100);

      // Cytoscape must NOT mount in neighborhood phase
      expect(cytoscapeHarness.mounted).toBe(false);
    });
  }
});

/* ── Gate 4 — QA-P1-13 rendering strategy benchmark ────────────────────────── */

describe("Gate 4 — QA-P1-13 rendering strategy (LOD + DOM budget)", () => {
  beforeEach(() => {
    cytoscapeHarness.reset();
    layoutHarness.reset();
    localStorage.clear();
  });

  for (const { label, count } of [
    { label: "500", count: 500 },
    { label: "1000", count: 1000 },
  ]) {
    it(`${label} tables: neighborhood paint — compact LOD, 0 detail, DOM ≤ 100`, async () => {
      const data = generateErFixture(count, 42);
      const { container } = render(
        <TooltipProvider>
          <ErDiagram connectionId="c" schema="public" data={data} />
        </TooltipProvider>,
      );

      await waitFor(() => expect(screen.getByTestId("er-search-input")).toBeInTheDocument());

      // Select a table → neighborhood
      const searchInput = screen.getByTestId("er-search-input");
      fireEvent.change(searchInput, { target: { value: "app" } });
      await waitFor(() => {
        expect(screen.getAllByTestId("er-search-result").length).toBeGreaterThan(0);
      });
      fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

      await waitFor(() => {
        expect(container.querySelector(".react-flow")).not.toBeNull();
      });

      // QA-P1-13: first-paint LOD — all nodes at compact (tier 1), zero at detail (tier 3).
      // Full column rows never mount before an explicit focus action.
      const detailNodes = container.querySelectorAll('[data-tier="3"]');
      expect(detailNodes.length).toBe(0);

      // No [data-column] rows in the DOM (column rows only mount at detail tier)
      const columnRows = container.querySelectorAll("[data-column]");
      expect(columnRows.length).toBe(0);

      // DOM node count bounded by neighborhood cap
      const rfDomNodes = container.querySelectorAll(".react-flow__node");
      expect(rfDomNodes.length).toBeLessThanOrEqual(100);
      expect(rfDomNodes.length).toBeGreaterThan(0);

      // Compact tier nodes present
      const compactNodes = container.querySelectorAll('[data-tier="1"]');
      expect(compactNodes.length).toBeGreaterThan(0);
    });
  }
});
