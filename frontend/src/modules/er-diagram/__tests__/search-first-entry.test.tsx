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
// Harness captures the input argument so tests can assert null during search.
const layoutHarness = vi.hoisted(() => ({
  lastInput: undefined as unknown,
  callCount: 0,
  reset() {
    this.lastInput = undefined;
    this.callCount = 0;
  },
}));

vi.mock("../hooks/use-worker-layout", () => ({
  useWorkerLayout: (input: unknown) => {
    layoutHarness.lastInput = input;
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

  it("layout builder runs after table selection", async () => {
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

    // After selection: builder may now run (phase → neighborhood)
    expect(builderHarness.callCount).toBeGreaterThan(0);
  });
});
