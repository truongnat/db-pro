import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { SchemaTree } from "../components/schema-tree";
import type { TreeNode } from "../types/schema.types";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const SAMPLE_NODES: TreeNode[] = [
  {
    id: "schema:public",
    type: "schema",
    label: "public",
    schemaName: "public",
    children: [
      { id: "table:public:users", type: "table", label: "users", schemaName: "public", tableName: "users" },
      { id: "table:public:orders", type: "table", label: "orders", schemaName: "public", tableName: "orders" },
    ],
  },
];

describe("SchemaTree", () => {
  it("renders schema nodes", () => {
    render(
      <SchemaTree
        treeNodes={SAMPLE_NODES}
        expandedNodes={new Set(["schema:public"])}
        selectedNodeId={null}
        onToggleNode={() => {}}
        onSelectNode={() => {}}
      />,
    );
    expect(screen.getByText("public")).toBeTruthy();
  });

  it("renders table nodes when schema is expanded", () => {
    render(
      <SchemaTree
        treeNodes={SAMPLE_NODES}
        expandedNodes={new Set(["schema:public"])}
        selectedNodeId={null}
        onToggleNode={() => {}}
        onSelectNode={() => {}}
      />,
    );
    expect(screen.getByText("users")).toBeTruthy();
    expect(screen.getByText("orders")).toBeTruthy();
  });

  it("hides table nodes when schema is collapsed", () => {
    render(
      <SchemaTree
        treeNodes={SAMPLE_NODES}
        expandedNodes={new Set()}
        selectedNodeId={null}
        onToggleNode={() => {}}
        onSelectNode={() => {}}
      />,
    );
    expect(screen.getByText("public")).toBeTruthy();
    expect(screen.queryByText("users")).toBeNull();
  });

  it("calls onToggleNode when clicking a schema node", () => {
    const onToggle = vi.fn();
    render(
      <SchemaTree
        treeNodes={SAMPLE_NODES}
        expandedNodes={new Set()}
        selectedNodeId={null}
        onToggleNode={onToggle}
        onSelectNode={() => {}}
      />,
    );
    fireEvent.click(screen.getByText("public"));
    expect(onToggle).toHaveBeenCalledWith("schema:public");
  });

  it("calls onSelectNode when clicking a table node", () => {
    const onSelect = vi.fn();
    render(
      <SchemaTree
        treeNodes={SAMPLE_NODES}
        expandedNodes={new Set(["schema:public"])}
        selectedNodeId={null}
        onToggleNode={() => {}}
        onSelectNode={onSelect}
      />,
    );
    fireEvent.click(screen.getByText("users"));
    expect(onSelect).toHaveBeenCalledWith("public", "users", "table");
  });

  it("shows empty state when no nodes", () => {
    render(
      <SchemaTree
        treeNodes={[]}
        expandedNodes={new Set()}
        selectedNodeId={null}
        onToggleNode={() => {}}
        onSelectNode={() => {}}
      />,
    );
    expect(screen.getByText("common.states.empty")).toBeTruthy();
  });
});
