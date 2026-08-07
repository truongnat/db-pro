import { render, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { DockSeparator } from "@/commons/components/dock-separator";

function createSeparatorProps(overrides: Record<string, unknown> = {}) {
  return {
    onMouseDown: vi.fn(),
    onDoubleClick: vi.fn(),
    onKeyDown: vi.fn(),
    role: "separator" as const,
    "aria-orientation": "horizontal" as const,
    "aria-valuenow": 35,
    "aria-valuemin": 10,
    "aria-valuemax": 85,
    tabIndex: 0 as const,
    ...overrides,
  };
}

describe("DockSeparator", () => {
  it("renders with correct ARIA attributes", () => {
    const props = createSeparatorProps();
    const { container } = render(
      <DockSeparator separatorProps={props} isCollapsed={false} />,
    );

    const el = container.firstElementChild as HTMLElement;
    expect(el).toBeTruthy();
    expect(el.getAttribute("role")).toBe("separator");
    expect(el.getAttribute("aria-orientation")).toBe("horizontal");
    expect(el.getAttribute("aria-valuenow")).toBe("35");
    expect(el.getAttribute("aria-valuemin")).toBe("10");
    expect(el.getAttribute("aria-valuemax")).toBe("85");
    expect(el.getAttribute("tabindex")).toBe("0");
  });

  it("calls onDoubleClick from separatorProps when double-clicked", () => {
    const onDoubleClick = vi.fn();
    const props = createSeparatorProps({ onDoubleClick });
    const { container } = render(
      <DockSeparator separatorProps={props} isCollapsed={false} />,
    );

    fireEvent.doubleClick(container.firstElementChild!);
    expect(onDoubleClick).toHaveBeenCalledOnce();
  });

  it("forwards mouseDown to separatorProps handler", () => {
    const onMouseDown = vi.fn();
    const props = createSeparatorProps({ onMouseDown });
    const { container } = render(
      <DockSeparator separatorProps={props} isCollapsed={false} />,
    );

    fireEvent.mouseDown(container.firstElementChild!);
    expect(onMouseDown).toHaveBeenCalledOnce();
  });

  it("forwards keyDown to separatorProps handler", () => {
    const onKeyDown = vi.fn();
    const props = createSeparatorProps({ onKeyDown });
    const { container } = render(
      <DockSeparator separatorProps={props} isCollapsed={false} />,
    );

    fireEvent.keyDown(container.firstElementChild!, { key: "ArrowUp" });
    expect(onKeyDown).toHaveBeenCalledOnce();
  });

  it("shows expand label when collapsed", () => {
    const props = createSeparatorProps();
    const { container } = render(
      <DockSeparator separatorProps={props} isCollapsed={true} />,
    );

    const el = container.firstElementChild as HTMLElement;
    expect(el.getAttribute("aria-label")).toBe("Expand panel");
  });

  it("shows resize label when expanded", () => {
    const props = createSeparatorProps();
    const { container } = render(
      <DockSeparator separatorProps={props} isCollapsed={false} />,
    );

    const el = container.firstElementChild as HTMLElement;
    expect(el.getAttribute("aria-label")).toBe("Resize panel");
  });
});
