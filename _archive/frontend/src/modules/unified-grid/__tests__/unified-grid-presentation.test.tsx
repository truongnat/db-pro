import { describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/react";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        common: { states: { loading: "Loading" } },
        dataGrid: {
          noData: "No data",
          sortBy: "Sort by {{column}}",
          resizeColumn: "Resize {{column}}",
          rowNumber: "Select row {{row}}",
          rowNumberColumn: "Row number",
          selectAllRows: "Select all rows",
          rowActionsColumn: "Row actions",
          gridAriaLabel: "Result grid",
          contextMenuLabel: "Grid context menu",
          editRow: "Edit row {{row}}",
          deleteRow: "Delete row {{row}}",
        },
      },
    },
  },
  lng: "en",
  fallbackLng: "en",
});

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 30,
    getVirtualItems: () =>
      Array.from({ length: Math.min(count, 10) }, (_, i) => ({
        index: i,
        start: i * 30,
        size: 30,
        end: (i + 1) * 30,
        key: i,
        measureElement: vi.fn(),
      })),
  }),
}));

import { UnifiedGrid } from "../components/unified-grid";
import type { ColumnMeta, Row } from "../types";

const columns: ColumnMeta[] = [
  { name: "id", dataType: "int8", nullable: false },
  { name: "name", dataType: "varchar", nullable: false },
  { name: "is_active", dataType: "bool", nullable: false },
  { name: "price", dataType: "numeric", nullable: true },
];

const rows: Row[] = [
  [
    { type: "int64", value: "1" },
    { type: "text", value: "Alice" },
    { type: "bool", value: true },
    { type: "float64", value: 2499 },
  ],
  [
    { type: "int64", value: "2" },
    { type: "text", value: "Bob" },
    { type: "bool", value: false },
    { type: "null" },
  ],
];

function renderGrid(props: Partial<React.ComponentProps<typeof UnifiedGrid>> = {}) {
  return render(
    <I18nextProvider i18n={i18n}>
      <UnifiedGrid columns={columns} rows={rows} sorts={[]} onSort={vi.fn()} {...props} />
    </I18nextProvider>,
  );
}

describe("UnifiedGrid — cell presentation", () => {
  it("right-aligns numeric cells and leaves text left-aligned", () => {
    const { container } = renderGrid();
    const firstRow = container.querySelector('[data-index="0"]');
    const cells = firstRow?.querySelectorAll('[role="gridcell"]');

    const idCell = cells?.[0];
    const nameCell = cells?.[1];
    expect(idCell?.className).toContain("justify-end");
    expect(idCell?.className).toContain("tabular-nums");
    expect(nameCell?.className).not.toContain("justify-end");
  });

  it("renders NULL as an explicit sentinel, never as an empty string", () => {
    const { container } = renderGrid();
    const nullCell = container.querySelector('[data-index="1"] [data-cell-kind="null"]');
    expect(nullCell).not.toBeNull();
    expect(nullCell?.textContent).toBe("NULL");
  });

  it("renders booleans with a check / dash marker plus the literal", () => {
    const { container } = renderGrid();
    expect(container.querySelector('[data-index="0"] [data-cell-kind="true"]')).not.toBeNull();
    expect(container.querySelector('[data-index="0"] .lucide-check')).toBeInTheDocument();
    expect(container.querySelector('[data-index="1"] [data-cell-kind="false"]')).not.toBeNull();
    expect(container.querySelector('[data-index="1"] .lucide-minus')).toBeInTheDocument();
  });

  it("truncates long text through a span so the ellipsis can render", () => {
    const longText: Row[] = [[{ type: "text", value: "x".repeat(400) }]];
    const { container } = renderGrid({
      columns: [{ name: "note", dataType: "text", nullable: false }],
      rows: longText,
    });
    const cell = container.querySelector('[data-index="0"] [role="gridcell"]');
    expect(cell?.firstElementChild?.className).toContain("truncate");
  });
});

describe("UnifiedGrid — accessibility contract", () => {
  it("exposes a grid with row/cell roles and a stable row count", () => {
    const { container, getByRole } = renderGrid();
    const grid = getByRole("grid");
    expect(grid).toHaveAttribute("aria-rowcount", "2");
    expect(grid).toHaveAttribute("aria-colcount", "4");
    expect(container.querySelectorAll('[role="row"]').length).toBe(3); // header + 2 body rows
    expect(container.querySelectorAll('[role="gridcell"]').length).toBe(8);
    expect(container.querySelectorAll('[role="rowheader"]').length).toBe(2);
  });

  it("localises the resize handle instead of hard-coding English", () => {
    const { container } = renderGrid();
    const separator = container.querySelector('[role="separator"]');
    expect(separator).toHaveAttribute("aria-label", "Resize id");
    expect(separator?.getAttribute("aria-label")).not.toContain("dataGrid.");
  });

  it("flags the grid busy while a page is loading", () => {
    const { getByRole } = renderGrid({ isLoading: true });
    expect(getByRole("grid")).toHaveAttribute("aria-busy", "true");
  });
});

describe("UnifiedGrid — loading and pinned columns", () => {
  it("shows skeleton rows plus a live status while loading", () => {
    const { container, getByRole } = renderGrid({ isLoading: true });
    expect(getByRole("status")).toHaveTextContent("Loading");
    expect(container.querySelector(".lucide-loader-circle, .lucide-loader-2")).toBeInTheDocument();
    expect(container.querySelectorAll(".animate-pulse").length).toBeGreaterThan(0);
  });

  it("pins frozen columns with sticky offsets behind the row-number gutter", () => {
    const { container } = renderGrid({ frozenColumns: ["name"] });

    const firstRow = container.querySelector('[data-index="0"]');
    const gutter = firstRow?.querySelector('[role="rowheader"]') as HTMLElement;
    const frozenCell = firstRow?.querySelectorAll('[role="gridcell"]')[0] as HTMLElement;
    const normalCell = firstRow?.querySelectorAll('[role="gridcell"]')[1] as HTMLElement;

    expect(gutter.style.position).toBe("sticky");
    expect(gutter.style.left).toBe("0px");

    expect(frozenCell.style.position).toBe("sticky");
    // Gutter is 40px wide, so the first pinned column starts at 40px.
    expect(frozenCell.style.left).toBe("40px");

    // Non-pinned columns scroll normally.
    expect(normalCell.style.position).toBe("");
  });

  it("offsets a second frozen column by the width of the first", () => {
    const { container } = renderGrid({
      frozenColumns: ["id", "name"],
      columnWidths: { id: 100 },
    });
    const firstRow = container.querySelector('[data-index="0"]');
    const frozenCells = firstRow?.querySelectorAll('[role="gridcell"]');
    expect((frozenCells?.[0] as HTMLElement).style.left).toBe("40px");
    expect((frozenCells?.[1] as HTMLElement).style.left).toBe("140px");
  });
});
