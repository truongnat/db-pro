import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { SearchView } from "./search-view";

const openSchemaPreview = vi.fn();
const openTableData = vi.fn();

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string, params?: { count?: number }) =>
      key === "shell.sidebar.searchResultCount" ? `${params?.count ?? 0} matching objects` : key,
  }),
}));

vi.mock("@/commons/stores/connection.store", () => ({
  useConnectionStore: (selector: (state: { explorerConnectionId: string }) => unknown) =>
    selector({ explorerConnectionId: "connection-1" }),
}));

vi.mock("@/commons/hooks/use-sidebar-tab-ops", () => ({
  useSidebarTabOps: () => ({ openSchemaPreview, openTableData }),
}));

vi.mock("@/modules/schema/queries/schema.queries", () => ({
  useIntrospect: () => ({
    data: {
      tables: [{ schema: "public", name: "orders" }],
      views: [{ schema: "reporting", name: "daily_sales" }],
    },
  }),
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 28,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({ index, start: index * 28 })),
  }),
}));

describe("SearchView", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  beforeEach(() => {
    vi.useFakeTimers();
    openSchemaPreview.mockReset();
    openTableData.mockReset();
  });

  it("debounces catalog filtering and reports matching objects", () => {
    render(<SearchView />);
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "orders" } });

    expect(screen.queryByText("1 matching objects")).not.toBeInTheDocument();
    act(() => vi.advanceTimersByTime(120));

    expect(screen.getByText("1 matching objects")).toBeInTheDocument();
    expect(screen.getByTitle("public.orders")).toBeInTheDocument();
  });
});
