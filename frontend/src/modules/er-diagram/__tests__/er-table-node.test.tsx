import { render } from "@testing-library/react";
import { ReactFlowProvider, type NodeProps } from "@xyflow/react";
import { describe, expect, it } from "vitest";

import { ErTableNode } from "../components/lod/er-table-node";
import type { TableNodeData } from "../components/lod/types";

/**
 * P3.5 / P1.3 — mechanical verification that the LOD dispatcher switches the
 * RENDER TREE (hard rule #2: LOD changes the render tree, never CSS-hidden):
 * below `detail` the leaf components must not contain any per-column DOM.
 */

const TWO_COLUMN_DATA: TableNodeData = {
  label: "orders",
  schema: "public",
  columns: [
    { name: "id", dataType: "serial", nullable: false, isPrimaryKey: true, isForeignKey: false },
    {
      name: "user_id",
      dataType: "integer",
      nullable: true,
      isPrimaryKey: false,
      isForeignKey: true,
    },
  ],
  lod: "detail",
  compact: false,
};

function makeProps(lod: TableNodeData["lod"], selected = false): NodeProps {
  return {
    id: "public.orders",
    data: { ...TWO_COLUMN_DATA, lod },
    type: "table",
    selected,
    zIndex: 0,
    dragging: false,
    draggable: true,
    selectable: true,
    deletable: false,
    isConnectable: true,
    positionAbsoluteX: 0,
    positionAbsoluteY: 0,
  };
}

function renderNode(lod: TableNodeData["lod"], selected = false) {
  return render(
    <ReactFlowProvider>
      <ErTableNode {...makeProps(lod, selected)} />
    </ReactFlowProvider>,
  );
}

describe("ErTableNode LOD dispatcher", () => {
  it("renders the dot leaf (tier 0) with no column rows", () => {
    const { container } = renderNode("dot");
    expect(container.querySelector('[data-tier="0"]')).toBeTruthy();
    expect(container.querySelector('[data-tier="3"]')).toBeNull();
    expect(container.querySelectorAll("[data-column]")).toHaveLength(0);
  });

  it("renders the compact leaf (tier 1) with no column rows", () => {
    const { container } = renderNode("compact");
    expect(container.querySelector('[data-tier="1"]')).toBeTruthy();
    expect(container.querySelectorAll("[data-column]")).toHaveLength(0);
  });

  it("renders the summary leaf (tier 2) with counts but no per-column rows", () => {
    const { container } = renderNode("summary");
    expect(container.querySelector('[data-tier="2"]')).toBeTruthy();
    expect(container.textContent).toContain("2 cols");
    expect(container.textContent).toContain("1 FK");
    expect(container.querySelectorAll("[data-column]")).toHaveLength(0);
  });

  it("renders the detailed leaf (tier 3) with every column row", () => {
    const { container } = renderNode("detail");
    expect(container.querySelector('[data-tier="3"]')).toBeTruthy();
    const rows = container.querySelectorAll("[data-column]");
    expect(rows).toHaveLength(2);
    expect(rows[0].getAttribute("data-column")).toBe("id");
    expect(rows[1].getAttribute("data-column")).toBe("user_id");
  });

  it("exactly one leaf is mounted at any time (true switch, not CSS hiding)", () => {
    for (const lod of ["dot", "compact", "summary", "detail"] as const) {
      const { container } = renderNode(lod);
      const tiers = container.querySelectorAll("[data-tier]");
      expect(tiers.length).toBe(1);
    }
  });

  it("selection changes styling only — never the render tree (zoom-driven LOD)", () => {
    const plain = renderNode("compact");
    expect(plain.container.querySelector("[data-tier]")?.className).toContain(
      "border-[var(--border-default)]",
    );

    const selected = renderNode("compact", true);
    expect(selected.container.querySelector("[data-tier]")?.className).toContain("border-primary");
    // Same leaf (tier 1), same no-column render — only the highlight class differs.
    expect(selected.container.querySelectorAll("[data-column]")).toHaveLength(0);
  });
});
