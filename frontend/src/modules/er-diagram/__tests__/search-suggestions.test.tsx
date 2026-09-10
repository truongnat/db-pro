import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ErSearchEntry } from "../components/er-search-entry";
import type { ErGraphModel } from "../renderer/types";

const model: ErGraphModel = {
  tables: [
    { id: "public.hub", label: "hub", schema: "public", columnCount: 4, fkCount: 2 },
    { id: "public.orders", label: "orders", schema: "public", columnCount: 6, fkCount: 1 },
    { id: "public.users", label: "users", schema: "public", columnCount: 5, fkCount: 1 },
  ],
  relations: [
    { id: "orders-hub", source: "public.orders", target: "public.hub", name: "orders_hub" },
    { id: "users-hub", source: "public.users", target: "public.hub", name: "users_hub" },
  ],
  adjacency: new Map([
    ["public.hub", new Set(["public.orders", "public.users"])],
    ["public.orders", new Set(["public.hub"])],
    ["public.users", new Set(["public.hub"])],
  ]),
  stats: { tables: 3, relations: 2, columns: 15 },
};

describe("ErSearchEntry suggested starting points", () => {
  it("selects the highlighted suggestion with Enter", () => {
    const onSelectTable = vi.fn();
    render(<ErSearchEntry model={model} onSelectTable={onSelectTable} />);

    fireEvent.keyDown(screen.getByTestId("er-search-input"), { key: "Enter" });

    expect(onSelectTable).toHaveBeenCalledWith("public.hub");
  });

  it("selects a suggestion on click", () => {
    const onSelectTable = vi.fn();
    render(<ErSearchEntry model={model} onSelectTable={onSelectTable} />);

    fireEvent.click(screen.getAllByTestId("er-search-result")[0]);

    expect(onSelectTable).toHaveBeenCalledWith("public.hub");
  });
});
