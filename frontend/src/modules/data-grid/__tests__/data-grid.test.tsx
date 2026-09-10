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
          readOnlyNoPk: "Read-only — editing requires a primary key",
          readOnlyConnection: "Read-only connection — editing is disabled",
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
import { DataToolbar } from "../components/data-toolbar";
import type { ColumnMeta, Row } from "../types/data-grid.types";

const columns: ColumnMeta[] = [
  { name: "id", dataType: "INTEGER", nullable: false },
  { name: "name", dataType: "TEXT", nullable: false },
];

const rows: Row[] = [
  [
    { type: "int64", value: "1" },
    { type: "text", value: "Alice" },
  ],
  [
    { type: "int64", value: "2" },
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

  it("renders read-only for a read-only connection despite pkColumns", async () => {
    const onEditCell = vi.fn();
    const user = userEvent.setup();
    renderGrid({ onEditCell, readOnly: true });
    expect(screen.getByText("Read-only connection — editing is disabled")).toBeInTheDocument();
    await user.dblClick(screen.getByText("Alice"));
    expect(onEditCell).not.toHaveBeenCalled();
  });

  it("shows sort indicator", () => {
    const { container } = renderGrid({ sorts: [{ column: "name", direction: "desc" }] });

    // The direction is exposed to assistive tech via aria-sort, and rendered
    // visually as a Lucide arrow (previously a bare "▼" text glyph).
    const sortedHeader = container.querySelector('[aria-sort="descending"]');
    expect(sortedHeader).not.toBeNull();
    expect(sortedHeader?.textContent).toContain("name");
    expect(container.querySelector(".lucide-arrow-down")).toBeInTheDocument();
    expect(screen.queryByText("\u25BC")).not.toBeInTheDocument();
  });

  it("marks unsorted columns as aria-sort none and hints the affordance on hover", () => {
    const { container } = renderGrid({ sorts: [] });
    const unsortedHeader = container.querySelector('[aria-sort="none"]');
    expect(unsortedHeader).not.toBeNull();
    expect(container.querySelector(".lucide-arrow-up-down")).toBeInTheDocument();
    // id / name / gutter / (no row actions here) → header cells are real
    // columnheaders so screen readers can navigate the result set.
    expect(screen.getAllByRole("columnheader").length).toBeGreaterThanOrEqual(2);
  });

  it("shows empty state when no columns", () => {
    renderGrid({ columns: [], rows: [] });
    expect(screen.getByText("No data")).toBeInTheDocument();
  });
});

describe("DataToolbar QA-P2-07", () => {
  it("calls onToggleHiddenColumn exactly once when clicking column checkbox directly", async () => {
    const onToggleHiddenColumn = vi.fn();
    const user = userEvent.setup();

    render(
      <I18nextProvider i18n={i18n}>
        <TooltipProvider>
          <DataToolbar
            columns={columns}
            rowCount={2}
            filters={[]}
            sorts={[]}
            draftFilters={[]}
            draftSorts={[]}
            hiddenColumns={[]}
            onAddDraftFilter={vi.fn()}
            onRemoveDraftFilter={vi.fn()}
            onApplyFilters={vi.fn()}
            onClearFilters={vi.fn()}
            onAddDraftSort={vi.fn()}
            onRemoveDraftSort={vi.fn()}
            onApplySorts={vi.fn()}
            onClearSorts={vi.fn()}
            onToggleHiddenColumn={onToggleHiddenColumn}
            onShowAllColumns={vi.fn()}
          />
        </TooltipProvider>
      </I18nextProvider>,
    );

    // Open Columns popover
    const columnsBtn = screen.getByRole("button", { name: /Columns/i });
    await user.click(columnsBtn);

    // Find the checkboxes
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes.length).toBeGreaterThan(0);

    // Click directly on the first column checkbox
    await user.click(checkboxes[0]);

    // Check that onToggleHiddenColumn was called exactly ONCE (not twice)
    expect(onToggleHiddenColumn).toHaveBeenCalledTimes(1);
    expect(onToggleHiddenColumn).toHaveBeenCalledWith("id");
  });
});
