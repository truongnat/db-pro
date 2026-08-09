import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        dataGrid: {
          noData: "No data",
          confirmDelete: "Delete this row?",
        },
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

import { UnifiedGrid } from "../components/unified-grid";
import type { ColumnMeta, Row } from "../types";

const columns: ColumnMeta[] = [
  { name: "id", dataType: "INTEGER", nullable: false },
  { name: "name", dataType: "TEXT", nullable: false },
];

const rows: Row[] = [
  [{ type: "int64", value: 1 }, { type: "text", value: "Alice" }],
  [{ type: "int64", value: 2 }, { type: "text", value: "Bob" }],
];

function renderGrid(props: Partial<React.ComponentProps<typeof UnifiedGrid>> = {}) {
  const defaults = {
    columns,
    rows,
    sorts: [] as const,
    onSort: vi.fn(),
    editingCell: null as { row: number; col: number } | null,
    onEditCell: vi.fn(),
    onCellSave: vi.fn(),
    selectedRows: new Set<number>(),
    isLoading: false,
  };
  return render(
    <I18nextProvider i18n={i18n}>
      <UnifiedGrid {...defaults} {...props} />
    </I18nextProvider>,
  );
}

describe("UnifiedGrid — copy keyboard scope", () => {
  let writeTextSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    writeTextSpy = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue();
  });

  it("does NOT call clipboard when Cmd+C is pressed inside an INPUT", () => {
    // Row 0 selected, cell (0, 1) being edited → renders an <input>
    const { container } = renderGrid({
      selectedRows: new Set([0]),
      editingCell: { row: 0, col: 1 },
    });

    const input = container.querySelector("input");
    expect(input).toBeTruthy();

    // Fire Cmd+C from the input
    fireEvent.keyDown(input!, { key: "c", ctrlKey: true, metaKey: true });

    // Grid clipboard writer must NOT be called
    expect(writeTextSpy).not.toHaveBeenCalled();
  });

  it("calls clipboard when Cmd+C is pressed on the grid container (no editable target)", () => {
    const { container } = renderGrid({
      selectedRows: new Set([0]),
    });

    // The grid container has tabIndex=0 and onKeyDown
    const gridContainer = container.querySelector('[tabindex="0"]');
    expect(gridContainer).toBeTruthy();

    // Fire Cmd+C from the grid container itself
    fireEvent.keyDown(gridContainer!, { key: "c", ctrlKey: true, metaKey: true });

    // Grid clipboard writer SHOULD be called
    expect(writeTextSpy).toHaveBeenCalled();
  });
});
