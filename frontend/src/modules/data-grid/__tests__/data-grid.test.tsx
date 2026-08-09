import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";
import { TooltipProvider } from "@/components/ui/tooltip";

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        dataGrid: {
          noData: "No data",
          confirmDelete: "Delete this row?",
        },
        query: {
          rowsAffected: "{{count}} row affected",
          rowsAffected_other: "{{count}} rows affected",
        },
        common: { actions: { delete: "Delete" } },
      },
    },
  },
  lng: "en",
  fallbackLng: "en",
});

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 32,
    getVirtualItems: () =>
      Array.from({ length: Math.min(count, 10) }, (_, i) => ({
        index: i,
        start: i * 32,
        size: 32,
        end: (i + 1) * 32,
        key: i,
        measureElement: vi.fn(),
      })),
  }),
}));

import { DataGrid } from "../components/data-grid";
import type { ColumnMeta, Row } from "../types/data-grid.types";

const columns: ColumnMeta[] = [
  { name: "id", dataType: "INTEGER", nullable: false },
  { name: "name", dataType: "TEXT", nullable: false },
];

const rows: Row[] = [
  [
    { type: "int64", value: 1 },
    { type: "text", value: "Alice" },
  ],
  [
    { type: "int64", value: 2 },
    { type: "text", value: "Bob" },
  ],
];

function renderGrid(props: Partial<React.ComponentProps<typeof DataGrid>> = {}) {
  const defaults = {
    columns,
    rows,
    sorts: [],
    onSort: vi.fn(),
    editingCell: null,
    onEditCell: vi.fn(),
    onCellSave: vi.fn(),
    onDeleteRow: vi.fn(),
    isDeleting: false,
    isLoading: false,
    pkColumns: ["id"],
  };
  return render(
    <I18nextProvider i18n={i18n}>
      <TooltipProvider>
        <DataGrid {...defaults} {...props} />
      </TooltipProvider>
    </I18nextProvider>,
  );
}

describe("DataGrid", () => {
  it("renders column headers", () => {
    renderGrid();
    expect(screen.getByText("id")).toBeInTheDocument();
    expect(screen.getByText("name")).toBeInTheDocument();
  });

  it("renders cell values", () => {
    renderGrid();
    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
  });

  it("calls onSort when clicking a column header", async () => {
    const onSort = vi.fn();
    const user = userEvent.setup();
    renderGrid({ onSort });
    await user.click(screen.getByText("id"));
    expect(onSort).toHaveBeenCalledWith("id");
  });

  it("calls onEditCell on double-click when pkColumns exist", async () => {
    const onEditCell = vi.fn();
    const user = userEvent.setup();
    renderGrid({ onEditCell });
    await user.dblClick(screen.getByText("Alice"));
    expect(onEditCell).toHaveBeenCalledWith({ row: 0, col: 1 });
  });

  it("does not call onEditCell on double-click when no pkColumns", async () => {
    const onEditCell = vi.fn();
    const user = userEvent.setup();
    renderGrid({ onEditCell, pkColumns: [] });
    await user.dblClick(screen.getByText("Alice"));
    expect(onEditCell).not.toHaveBeenCalled();
  });

  it("shows sort indicator", () => {
    renderGrid({ sorts: [{ column: "name", direction: "desc" }] });
    expect(screen.getByText("\u25BC")).toBeInTheDocument();
  });

  it("shows empty state when no columns", () => {
    renderGrid({ columns: [], rows: [] });
    expect(screen.getByText("No data")).toBeInTheDocument();
  });
});
