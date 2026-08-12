import { describe, expect, it } from "vitest";

import { CytoscapeErRenderer } from "../renderer/cytoscape-renderer";
import type { ErGraphModel, ErPosition, TableId } from "../renderer/types";

/**
 * P2-2 (review F-REV-5) — runtime tests for the REAL canvas renderer path.
 *
 * These instantiate the actual `CytoscapeErRenderer` with the actual cytoscape
 * library in headless mode (jsdom has no canvas; positions/classes/theme/bounds
 * all work headless — painting does not). This mechanically proves the renderer
 * contract: mount, async position upgrade (P1-1), selection, theme swap
 * (P2-1), fit, and dispose — not a mock or a reimplementation.
 */

function buildModel(n: number): ErGraphModel {
  const tables = Array.from({ length: n }, (_, i) => ({
    id: `t${i}` as TableId,
    label: `t${i}`,
    schema: "public",
    columnCount: 3,
    fkCount: i > 0 ? 1 : 0,
  }));
  const relations = Array.from({ length: Math.max(0, n - 1) }, (_, i) => ({
    id: `r${i}`,
    source: `t${i}` as TableId,
    target: `t${i + 1}` as TableId,
    name: `fk${i}`,
  }));
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
    stats: { tables: n, relations: relations.length, columns: n * 3 },
  };
}

function positionsOf(...ids: string[]): Map<TableId, ErPosition> {
  const map = new Map<TableId, ErPosition>();
  ids.forEach((id, i) => map.set(id, { x: i * 100, y: 0 }));
  return map;
}

function makeRenderer() {
  const renderer = new CytoscapeErRenderer({
    container: null as unknown as HTMLElement,
    headless: true,
  });
  return renderer;
}

describe("CytoscapeErRenderer — real cytoscape (headless)", () => {
  it("mount adds every table + relation of the model", () => {
    const renderer = makeRenderer();
    const model = buildModel(5);
    renderer.mount(model, positionsOf("t0", "t1", "t2", "t3", "t4"));

    const cy = renderer.getCy();
    expect(cy.nodes().length).toBe(5);
    expect(cy.edges().length).toBe(4);

    // Positions from the mount map were applied.
    expect(cy.getElementById("t2").position()).toEqual({ x: 200, y: 0 });
    renderer.dispose();
  });

  it("updatePositions upgrades positions in place (P1-1 async layout upgrade)", () => {
    const renderer = makeRenderer();
    const model = buildModel(3);
    renderer.mount(model, positionsOf("t0", "t1", "t2"));

    const cy = renderer.getCy();
    expect(cy.getElementById("t1").position()).toEqual({ x: 100, y: 0 });

    // Simulate dagre committing: nodes move, structure stays (no re-mount).
    renderer.updatePositions(
      new Map<TableId, ErPosition>([
        ["t0", { x: 500, y: 500 }],
        ["t1", { x: 600, y: 500 }],
        ["t2", { x: 700, y: 500 }],
      ]),
    );
    expect(cy.getElementById("t0").position()).toEqual({ x: 500, y: 500 });
    expect(cy.getElementById("t2").position()).toEqual({ x: 700, y: 500 });
    expect(cy.nodes().length).toBe(3); // no element churn
    expect(cy.edges().length).toBe(2);
    renderer.dispose();
  });

  it("updateSelection toggles selected + neighbor classes", () => {
    const renderer = makeRenderer();
    const model = buildModel(4);
    renderer.mount(model, positionsOf("t0", "t1", "t2", "t3"));

    const cy = renderer.getCy();
    renderer.updateSelection({ nodeIds: ["t1"] });

    expect(cy.getElementById("t1").hasClass("selected")).toBe(true);
    // t1's 1-hop neighbors: t0 and t2 (closedNeighborhood includes self).
    expect(cy.getElementById("t0").hasClass("neighbor")).toBe(true);
    expect(cy.getElementById("t2").hasClass("neighbor")).toBe(true);
    expect(cy.getElementById("t3").hasClass("neighbor")).toBe(false);

    // Re-select elsewhere clears the old highlight.
    renderer.updateSelection({ nodeIds: ["t3"] });
    expect(cy.getElementById("t1").hasClass("selected")).toBe(false);
    expect(cy.getElementById("t3").hasClass("selected")).toBe(true);
    renderer.dispose();
  });

  it("updateTheme stores resolved tokens and keeps the graph alive (P2-1)", () => {
    const renderer = makeRenderer();
    const model = buildModel(3);
    renderer.mount(model, positionsOf("t0", "t1", "t2"));

    renderer.updateTheme({
      nodeBg: "#123456",
      nodeLabel: "#abcdef",
      edgeColor: "#fedcba",
      selectedNodeBorder: "#010203",
    });

    const theme = renderer.getTheme();
    expect(theme.nodeBg).toBe("#123456");
    expect(theme.nodeLabel).toBe("#abcdef");
    expect(theme.edgeColor).toBe("#fedcba");
    expect(theme.selectedNodeBorder).toBe("#010203");
    // Unspecified tokens keep the defaults.
    expect(theme.nodeBorder).toBe("#475569");

    // Graph survives the theme swap.
    expect(renderer.getCy().nodes().length).toBe(3);
    renderer.dispose();
  });

  it("dispose destroys the instance", () => {
    const renderer = makeRenderer();
    renderer.mount(buildModel(2), positionsOf("t0", "t1"));
    const cy = renderer.getCy();
    renderer.dispose();
    expect(cy.destroyed()).toBe(true);
  });

  it("getBoundingBox reflects mounted node extents", () => {
    const renderer = makeRenderer();
    const model = buildModel(3);
    renderer.mount(model, positionsOf("t0", "t1", "t2"));
    const bb = renderer.getBoundingBox();
    // Nodes span x=0..200 (positions), so bounds must cover [0, 200].
    expect(bb.x1).toBeLessThanOrEqual(0);
    expect(bb.x2).toBeGreaterThanOrEqual(200);
    renderer.dispose();
  });

  it("updateSelection with highlightNodeIds + fadeRest fades the rest (opass focus)", () => {
    const renderer = makeRenderer();
    const model = buildModel(5); // chain t0-t1-t2-t3-t4
    renderer.mount(model, positionsOf("t0", "t1", "t2", "t3", "t4"));

    const cy = renderer.getCy();
    // Focus t2, highlight neighbor t1, fade everything outside {t2, t1}.
    renderer.updateSelection({
      nodeIds: ["t2"],
      highlightNodeIds: ["t1"],
      fadeRest: true,
    });

    expect(cy.getElementById("t2").hasClass("selected")).toBe(true);
    expect(cy.getElementById("t1").hasClass("highlighted")).toBe(true);
    // t0, t3, t4 are outside the focus set → faded.
    expect(cy.getElementById("t0").hasClass("faded")).toBe(true);
    expect(cy.getElementById("t3").hasClass("faded")).toBe(true);
    expect(cy.getElementById("t4").hasClass("faded")).toBe(true);
    // Edges within the focus set are highlighted; edges touching it stay
    // unfaded; edges between two faded nodes are faded.
    expect(cy.getElementById("r1").hasClass("highlighted")).toBe(true); // t1-t2
    expect(cy.getElementById("r0").hasClass("faded")).toBe(false); // t0-t1 touches t1
    expect(cy.getElementById("r2").hasClass("faded")).toBe(false); // t2-t3 touches t2
    expect(cy.getElementById("r3").hasClass("faded")).toBe(true); // t3-t4: both faded

    // Re-select elsewhere clears the old highlight + fade.
    renderer.updateSelection({ nodeIds: ["t0"], highlightNodeIds: ["t1"], fadeRest: true });
    expect(cy.getElementById("t2").hasClass("selected")).toBe(false);
    expect(cy.getElementById("t2").hasClass("faded")).toBe(true);
    expect(cy.getElementById("t0").hasClass("faded")).toBe(false);
    renderer.dispose();
  });

  it("fadeRest without highlightNodeIds keeps legacy neighbors unfaded (reviewer P2)", () => {
    const renderer = makeRenderer();
    const model = buildModel(4);
    renderer.mount(model, positionsOf("t0", "t1", "t2", "t3"));

    const cy = renderer.getCy();
    // Legacy path (no highlightNodeIds) + fadeRest: the closedNeighborhood
    // neighbors must be part of the keep set, never faded.
    renderer.updateSelection({ nodeIds: ["t1"], fadeRest: true });
    expect(cy.getElementById("t1").hasClass("neighbor")).toBe(true);
    expect(cy.getElementById("t0").hasClass("faded")).toBe(false);
    expect(cy.getElementById("t2").hasClass("faded")).toBe(false);
    expect(cy.getElementById("t3").hasClass("faded")).toBe(true);
    renderer.dispose();
  });

  it("clearSelection removes focus classes but not search rings", () => {
    const renderer = makeRenderer();
    const model = buildModel(4);
    renderer.mount(model, positionsOf("t0", "t1", "t2", "t3"));

    const cy = renderer.getCy();
    renderer.updateSelection({ nodeIds: ["t1"], highlightNodeIds: ["t0", "t2"], fadeRest: true });
    renderer.highlightSearch(["t3"]);
    expect(cy.getElementById("t1").hasClass("selected")).toBe(true);
    expect(cy.getElementById("t3").hasClass("searched")).toBe(true);

    renderer.clearSelection();
    expect(cy.getElementById("t1").hasClass("selected")).toBe(false);
    expect(cy.getElementById("t0").hasClass("faded")).toBe(false);
    expect(cy.getElementById("t1").hasClass("highlighted")).toBe(false);
    // Search rings survive clearSelection (cleared by query change / Escape).
    expect(cy.getElementById("t3").hasClass("searched")).toBe(true);
    renderer.dispose();
  });

  it("highlightSearch rings matches and clearSearchHighlight removes the rings", () => {
    const renderer = makeRenderer();
    const model = buildModel(4);
    renderer.mount(model, positionsOf("t0", "t1", "t2", "t3"));

    const cy = renderer.getCy();
    renderer.highlightSearch(["t1", "t3"]);
    expect(cy.getElementById("t1").hasClass("searched")).toBe(true);
    expect(cy.getElementById("t3").hasClass("searched")).toBe(true);
    expect(cy.getElementById("t2").hasClass("searched")).toBe(false);

    // Re-ringing replaces the previous set.
    renderer.highlightSearch(["t0"]);
    expect(cy.getElementById("t1").hasClass("searched")).toBe(false);
    expect(cy.getElementById("t0").hasClass("searched")).toBe(true);

    renderer.clearSearchHighlight();
    expect(cy.getElementById("t0").hasClass("searched")).toBe(false);
    renderer.dispose();
  });

  it("onBackgroundTap fires on an empty-canvas tap", () => {
    let tapped = 0;
    const renderer = new CytoscapeErRenderer({
      container: null as unknown as HTMLElement,
      headless: true,
      onBackgroundTap: () => {
        tapped++;
      },
    });
    const model = buildModel(2);
    renderer.mount(model, positionsOf("t0", "t1"));

    renderer.getCy().emit("tap"); // target === cy → background
    expect(tapped).toBe(1);
    renderer.dispose();
  });
});
