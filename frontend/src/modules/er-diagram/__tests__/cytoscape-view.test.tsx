import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CytoscapeErView, type CytoscapeErViewProps } from "../components/cytoscape-view";
import type { ErGraphModel, ErPosition, TableId } from "../renderer/types";

/**
 * PR#12 re-review — app-level interaction test for the large-schema overview.
 *
 * The regression: a single node click on the canvas navigated straight to the
 * table's detail tab (openDbObject), unmounting the ER diagram before the
 * opass-style focus (fade rest + highlight neighborhood) could render. The fix
 * splits the interactions: single tap = FOCUS ONLY (never navigate), double
 * tap / the side inspector's "Open table" button = the explicit navigation.
 *
 * The real `CytoscapeErRenderer` needs a canvas, so the renderer module is
 * mocked here — the component under test is the REAL view (all of the
 * interaction wiring lives in it), and the mocked renderer records the exact
 * renderer contract the view issues (updateSelection args, clear calls).
 */
const rendererState = vi.hoisted(() => ({
  callbacks: {} as Record<string, (...args: unknown[]) => void>,
  updateSelection: [] as unknown[],
  highlightSearch: [] as unknown[],
  clearSelectionCount: 0,
  clearSearchCount: 0,
  reset() {
    this.callbacks = {};
    this.updateSelection = [];
    this.highlightSearch = [];
    this.clearSelectionCount = 0;
    this.clearSearchCount = 0;
  },
}));

vi.mock("../renderer/cytoscape-renderer", () => ({
  CytoscapeErRenderer: class {
    constructor(options: Record<string, (...args: unknown[]) => void>) {
      rendererState.callbacks = options;
    }
    mount() {}
    updatePositions() {}
    updateViewport() {}
    updateSelection(sel: unknown) {
      rendererState.updateSelection.push(sel);
    }
    highlightSearch(ids: unknown, opts?: unknown) {
      rendererState.highlightSearch.push({ ids, opts });
    }
    clearSearchHighlight() {
      rendererState.clearSearchCount++;
    }
    clearSelection() {
      rendererState.clearSelectionCount++;
    }
    updateTheme() {}
    focusNode() {}
    fit() {}
    dispose() {}
    getCy() {
      return null;
    }
  },
}));

/** Chain model t0–t1–t2–t3 — hop-1 neighborhood of t1 = {t0, t2}; t3 unrelated. */
function buildModel(): ErGraphModel {
  const ids = ["t0", "t1", "t2", "t3"] as TableId[];
  const tables = ids.map((id) => ({
    id,
    label: id,
    schema: "public",
    columnCount: id === "t1" ? 42 : 8,
    fkCount: id === "t1" ? 5 : 1,
  }));
  const relations = [
    { id: "r0", source: "t0" as TableId, target: "t1" as TableId, name: "r0" },
    { id: "r1", source: "t1" as TableId, target: "t2" as TableId, name: "r1" },
    { id: "r2", source: "t2" as TableId, target: "t3" as TableId, name: "r2" },
  ];
  const adjacency = new Map<TableId, Set<TableId>>();
  for (const rel of relations) {
    let from = adjacency.get(rel.source);
    if (!from) adjacency.set(rel.source, (from = new Set()));
    from.add(rel.target);
    let to = adjacency.get(rel.target);
    if (!to) adjacency.set(rel.target, (to = new Set()));
    to.add(rel.source);
  }
  return {
    tables,
    relations,
    adjacency,
    stats: { tables: 4, relations: 3, columns: 66 },
  };
}

function positionsOf(...ids: string[]): Map<TableId, ErPosition> {
  const map = new Map<TableId, ErPosition>();
  ids.forEach((id, i) => map.set(id, { x: i * 100, y: 0 }));
  return map;
}

function renderView(overrides: Partial<CytoscapeErViewProps> = {}): {
  onOpenTable: ReturnType<typeof vi.fn>;
  onSearchChange: ReturnType<typeof vi.fn>;
  rerender: (props: Partial<CytoscapeErViewProps>) => void;
} {
  const onOpenTable = vi.fn();
  const onSearchChange = vi.fn();
  const base: CytoscapeErViewProps = {
    model: buildModel(),
    positions: positionsOf("t0", "t1", "t2", "t3"),
    degraded: false,
    layoutReady: true,
    onViewportChange: vi.fn(),
    onOpenTable,
    explorer: {
      totalTables: 4,
      relationCount: 3,
      columnCount: 66,
      hops: 1,
      onSelectHops: vi.fn(),
    },
    searchQuery: "",
    onSearchChange,
    ...overrides,
  };
  const view = render(<CytoscapeErView {...base} />);
  return {
    onOpenTable,
    onSearchChange,
    rerender: (props) => view.rerender(<CytoscapeErView {...base} {...props} />),
  };
}

describe("CytoscapeErView — click-to-focus vs click-to-navigate (PR#12 re-review P1)", () => {
  beforeEach(() => {
    rendererState.reset();
  });

  it("single tap focuses (fade rest + highlight neighborhood) and does NOT navigate", () => {
    const { onOpenTable } = renderView();

    // The renderer forwards a real canvas tap to this callback.
    act(() => {
      rendererState.callbacks.onNodeClick?.("t1");
    });

    // P1: a single click must never open the table — the schema workspace
    // stays active (navigation is double-click / explicit action only).
    expect(onOpenTable).not.toHaveBeenCalled();

    // Focus contract: seed is selected, hop-1 neighbors highlighted, and the
    // rest (t3) is faded — the exact updateSelection the renderer applies.
    const sel = rendererState.updateSelection.at(-1) as {
      nodeIds: TableId[];
      highlightNodeIds?: TableId[];
      fadeRest?: boolean;
    };
    expect(sel.nodeIds).toEqual(["t1"]);
    expect([...(sel.highlightNodeIds ?? [])].sort()).toEqual(["t0", "t2"]);
    expect(sel.fadeRest).toBe(true);

    // The side inspector surfaces the focused table with an explicit action.
    expect(screen.getByText("t1")).toBeInTheDocument();
    expect(screen.getByText("42 columns · 5 FK")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open table" })).toBeInTheDocument();
  });

  it("double tap opens the table (the explicit navigation action)", () => {
    const { onOpenTable } = renderView();

    act(() => {
      rendererState.callbacks.onNodeDoubleClick?.("t1");
    });

    expect(onOpenTable).toHaveBeenCalledTimes(1);
    expect(onOpenTable).toHaveBeenCalledWith("t1");
    // Double tap alone must not have re-focused the neighborhood — navigation
    // is the only effect of the explicit action.
    expect(rendererState.updateSelection.length).toBe(0);
  });

  it("the side inspector 'Open table' button opens the focused table", () => {
    const { onOpenTable } = renderView();

    act(() => {
      rendererState.callbacks.onNodeClick?.("t1");
    });
    fireEvent.click(screen.getByRole("button", { name: "Open table" }));

    expect(onOpenTable).toHaveBeenCalledTimes(1);
    expect(onOpenTable).toHaveBeenCalledWith("t1");
  });

  it("single tap on one node re-focuses (clears the previous focus, no navigation)", () => {
    const { onOpenTable } = renderView();

    act(() => {
      rendererState.callbacks.onNodeClick?.("t1");
    });
    act(() => {
      rendererState.callbacks.onNodeClick?.("t3");
    });

    expect(onOpenTable).not.toHaveBeenCalled();
    const sel = rendererState.updateSelection.at(-1) as {
      nodeIds: TableId[];
      highlightNodeIds?: TableId[];
    };
    expect(sel.nodeIds).toEqual(["t3"]);
    // t3's hop-1 neighbor is t2; t0/t1 fall outside the focus → faded.
    expect([...(sel.highlightNodeIds ?? [])].sort()).toEqual(["t2"]);
  });
});

describe("CytoscapeErView — background tap vs search state (PR#12 re-review P2)", () => {
  beforeEach(() => {
    rendererState.reset();
  });

  it("background tap clears the focus fade but keeps the search ring + query", () => {
    // Single-match search "t1" rings the match AND focuses it.
    const { onSearchChange } = renderView({ searchQuery: "t1" });
    expect(rendererState.highlightSearch).toHaveLength(1);
    expect(rendererState.highlightSearch[0]).toEqual({
      ids: ["t1"],
      opts: { focus: true },
    });

    act(() => {
      rendererState.callbacks.onBackgroundTap?.();
    });

    // P2: search is search state, selection is selection state. The background
    // tap clears focus/fade only — the ring survives and the input keeps the
    // query (the search effect must NOT have re-run or cleared the ring).
    expect(rendererState.clearSelectionCount).toBeGreaterThan(0);
    expect(rendererState.clearSearchCount).toBe(0);
    expect(onSearchChange).not.toHaveBeenCalled();
    expect((screen.getByPlaceholderText("Search tables...") as HTMLInputElement).value).toBe("t1");
  });

  it("Escape clears the search query (and thereby the ring + fade)", () => {
    const { onSearchChange } = renderView({ searchQuery: "t1" });

    fireEvent.keyDown(screen.getByPlaceholderText("Search tables..."), {
      key: "Escape",
    });

    expect(onSearchChange).toHaveBeenCalledWith("");
  });

  it("a fresh single tap after a background tap still focuses (state not stuck)", () => {
    const { onOpenTable } = renderView({ searchQuery: "t1" });

    act(() => {
      rendererState.callbacks.onBackgroundTap?.();
    });
    act(() => {
      rendererState.callbacks.onNodeClick?.("t2");
    });

    expect(onOpenTable).not.toHaveBeenCalled();
    const sel = rendererState.updateSelection.at(-1) as {
      nodeIds: TableId[];
      highlightNodeIds?: TableId[];
    };
    expect(sel.nodeIds).toEqual(["t2"]);
    expect([...(sel.highlightNodeIds ?? [])].sort()).toEqual(["t1", "t3"]);
  });

  it("search ring survives clicking a DIFFERENT table (search state ≠ selection state)", () => {
    const { onOpenTable } = renderView({ searchQuery: "t1" });
    // Single-match "t1" rings the match AND focuses it.
    expect(rendererState.highlightSearch).toHaveLength(1);

    // Click another table while the query is still live.
    act(() => {
      rendererState.callbacks.onNodeClick?.("t2");
    });

    // No navigation; focus moved to t2...
    expect(onOpenTable).not.toHaveBeenCalled();
    const sel = rendererState.updateSelection.at(-1) as {
      nodeIds: TableId[];
      highlightNodeIds?: TableId[];
    };
    expect(sel.nodeIds).toEqual(["t2"]);
    expect([...(sel.highlightNodeIds ?? [])].sort()).toEqual(["t1", "t3"]);
    // ...and the t1 search ring is still live (search state persists).
    expect(rendererState.clearSearchCount).toBe(0);
    expect((screen.getByPlaceholderText("Search tables...") as HTMLInputElement).value).toBe("t1");
  });
});
