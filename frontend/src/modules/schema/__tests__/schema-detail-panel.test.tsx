import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { SchemaDetailPanel } from "../components/schema-detail-panel";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("../queries/schema.queries", () => ({
  useTableInfo: () => ({
    data: null,
    isLoading: false,
    isError: false,
    error: null,
  }),
  useTableDdl: () => ({
    data: null,
    isLoading: false,
    isError: false,
    error: null,
  }),
}));

describe("SchemaDetailPanel", () => {
  it("shows select table message when no table selected", () => {
    render(
      <SchemaDetailPanel
        connectionId="conn-1"
        schema={null}
        table={null}
        nodeType={null}
        activeTab="columns"
        onTabChange={() => {}}
      />,
    );
    expect(screen.getByText("schema.selectTable")).toBeTruthy();
  });

  it("renders tab bar for table node", () => {
    render(
      <SchemaDetailPanel
        connectionId="conn-1"
        schema="public"
        table="users"
        nodeType="table"
        activeTab="columns"
        onTabChange={() => {}}
      />,
    );
    expect(screen.getByText("schema.columns")).toBeTruthy();
    expect(screen.getByText("schema.indexes")).toBeTruthy();
    expect(screen.getByText("schema.foreignKeys")).toBeTruthy();
    expect(screen.getByText("schema.ddl")).toBeTruthy();
  });

  it("renders only columns and ddl tabs for view node", () => {
    render(
      <SchemaDetailPanel
        connectionId="conn-1"
        schema="public"
        table="active_users"
        nodeType="view"
        activeTab="columns"
        onTabChange={() => {}}
      />,
    );
    expect(screen.getByText("schema.columns")).toBeTruthy();
    expect(screen.getByText("schema.ddl")).toBeTruthy();
    expect(screen.queryByText("schema.indexes")).toBeNull();
    expect(screen.queryByText("schema.foreignKeys")).toBeNull();
  });

  it("calls onTabChange when clicking a tab", () => {
    const onTabChange = vi.fn();
    render(
      <SchemaDetailPanel
        connectionId="conn-1"
        schema="public"
        table="users"
        nodeType="table"
        activeTab="columns"
        onTabChange={onTabChange}
      />,
    );
    fireEvent.click(screen.getByText("schema.ddl"));
    expect(onTabChange).toHaveBeenCalledWith("ddl");
  });
});
