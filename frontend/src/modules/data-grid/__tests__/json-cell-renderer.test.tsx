import { describe, expect, it } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { JsonCellRenderer } from "../components/json-cell-renderer";

describe("JsonCellRenderer", () => {
  it("renders collapsed preview by default", () => {
    render(<JsonCellRenderer value={{ key: "val" }} />);
    expect(screen.getByText("{ }")).toBeTruthy();
  });

  it("shows truncated preview for long JSON", () => {
    const longValue = { a: "x".repeat(50) };
    render(<JsonCellRenderer value={longValue} />);
    const btn = screen.getByRole("button");
    expect(btn.title.length).toBeGreaterThan(40);
  });

  it("shows full string value without JSON.stringify quotes", () => {
    render(<JsonCellRenderer value="hello world" />);
    const btn = screen.getByRole("button");
    expect(btn.title).toBe("hello world");
  });

  it("expands on click to show formatted JSON", () => {
    render(<JsonCellRenderer value={{ a: 1 }} />);
    const btn = screen.getByRole("button");
    fireEvent.click(btn);

    expect(screen.getByText("collapse")).toBeTruthy();
    const pre = document.querySelector("pre");
    expect(pre).toBeTruthy();
    expect(pre!.textContent).toContain('"a": 1');
  });

  it("collapses again when clicking collapse button", () => {
    render(<JsonCellRenderer value={{ a: 1 }} />);
    fireEvent.click(screen.getByRole("button"));
    fireEvent.click(screen.getByText("collapse"));

    expect(screen.getByText("{ }")).toBeTruthy();
  });

  it("handles null value", () => {
    render(<JsonCellRenderer value={null} />);
    const btn = screen.getByRole("button");
    expect(btn.title).toBe("null");
  });
});
