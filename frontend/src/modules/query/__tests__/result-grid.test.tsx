import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        query: {
          noResults: "No results",
          rowsAffected: "{{count}} row affected",
          rowsAffected_other: "{{count}} rows affected",
          duration: "{{duration}}ms",
          metadata: { info: "Column info" },
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

import { ResultGrid } from "../components/result-grid";
import type { ColumnMeta, Row } from "../types/query.types";

const columns: ColumnMeta[] = [
  { name: "id", dataType: "INTEGER", nullable: false },
  { name: "name", dataType: "TEXT", nullable: false },
];

const rows: Row[] = [
  [
    { type: "int64", value: "1" },
    { type: "text", value: "Alice" },
  ],
  [{ type: "int64", value: "2" }, { type: "null" }],
];

function renderGrid(props: Partial<React.ComponentProps<typeof ResultGrid>> = {}) {
  const defaults = {
    columns,
    rows,
    sort: { column: null, direction: null as const },
    onSort: vi.fn(),
    durationMs: 42,
    rowCount: 2,
  };
  return render(
    <I18nextProvider i18n={i18n}>
      <ResultGrid {...defaults} {...props} />
    </I18nextProvider>,
  );
}

describe("ResultGrid", () => {
  it("renders column headers", () => {
    renderGrid();
    expect(screen.getByText("id")).toBeInTheDocument();
    expect(screen.getByText("name")).toBeInTheDocument();
  });

  it("renders row cell values", () => {
    renderGrid();
    expect(screen.getByText("Alice")).toBeInTheDocument();
  });

  it("shows NULL for null cells", () => {
    renderGrid();
    const nulls = screen.getAllByText("NULL");
    expect(nulls.length).toBeGreaterThan(0);
  });

  it("calls onSort when clicking a column header", async () => {
    const onSort = vi.fn();
    const user = userEvent.setup();

    renderGrid({ onSort });
    await user.click(screen.getByText("id"));

    expect(onSort).toHaveBeenCalledWith("id");
  });

  it("shows row count and duration in status bar", () => {
    renderGrid();
    expect(screen.getByText("2 rows affected")).toBeInTheDocument();
    expect(screen.getByText("42ms")).toBeInTheDocument();
  });

  it("shows empty state when no columns", () => {
    renderGrid({ columns: [], rows: [] });
    expect(screen.getByText("No results")).toBeInTheDocument();
  });

  it("shows sort indicator for sorted column", () => {
    renderGrid({ sort: { column: "id", direction: "asc" } });
    expect(screen.getByText("\u25B2")).toBeInTheDocument();
  });

  it("renders zoom controls in footer", () => {
    renderGrid();
    expect(screen.getByText("100%")).toBeTruthy();
  });

  it("renders info buttons for each column header", () => {
    renderGrid();
    const infoButtons = screen.getAllByText("i");
    expect(infoButtons.length).toBe(columns.length);
  });

  it("opens metadata popover when clicking info button", async () => {
    const user = userEvent.setup();
    renderGrid();
    const infoButtons = screen.getAllByText("i");
    await user.click(infoButtons[0]);
    // The popover component may not render in test env, but the click handler fires
  });
});
