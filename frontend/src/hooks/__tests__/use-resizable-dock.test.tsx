import { render, act } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useResizableDock } from "@/hooks/use-resizable-dock";

const STORAGE_KEY = "db-pro-dock-size-test";
const MOCK_HEIGHT = 1000;

let resizeCallback: ResizeObserverCallback | null = null;

beforeEach(() => {
  localStorage.clear();
  resizeCallback = null;

  Object.defineProperty(HTMLDivElement.prototype, "clientHeight", {
    configurable: true,
    get() {
      return MOCK_HEIGHT;
    },
  });
  vi.spyOn(HTMLDivElement.prototype, "getBoundingClientRect").mockReturnValue({
    top: 0,
    left: 0,
    width: 800,
    height: MOCK_HEIGHT,
    bottom: MOCK_HEIGHT,
    right: 800,
    x: 0,
    y: 0,
    toJSON: () => ({}),
  });

  globalThis.ResizeObserver = class ResizeObserver {
    constructor(callback: ResizeObserverCallback) {
      resizeCallback = callback;
    }
    observe(target: Element) {
      if (resizeCallback) {
        resizeCallback(
          [
            {
              contentRect: { height: MOCK_HEIGHT, width: 800 } as DOMRectReadOnly,
              target,
              borderBoxSize: [],
              contentBoxSize: [],
              devicePixelContentBoxSize: [],
            },
          ],
          this as unknown as ResizeObserver,
        );
      }
    }
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
});

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

function renderWithRealDom(options?: Parameters<typeof useResizableDock>[0]) {
  let hookResult: ReturnType<typeof useResizableDock> | null = null;

  function TestComponent() {
    const result = useResizableDock(options);
    hookResult = result;
    return <div ref={result.containerRef} data-testid="container" />;
  }

  render(<TestComponent />);

  if (!hookResult) {
    throw new Error("Hook did not render");
  }

  return {
    get current() {
      return hookResult!;
    },
  };
}

describe("useResizableDock", () => {
  it("uses initial ratio of 0.65 by default", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-1" });

    expect(result.current.topHeight).toBe(650);
    expect(result.current.isCollapsed).toBe(false);
  });

  it("applies custom initial ratio", () => {
    const result = renderWithRealDom({
      initialRatio: 0.5,
      storageKey: STORAGE_KEY + "-2",
    });

    expect(result.current.topHeight).toBe(500);
  });

  it("restores ratio from localStorage", () => {
    localStorage.setItem(STORAGE_KEY, "0.6");

    const result = renderWithRealDom({ storageKey: STORAGE_KEY });

    expect(result.current.topHeight).toBe(600);
  });

  it("provides correct ARIA separator props", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-3" });

    const { separatorProps } = result.current;
    expect(separatorProps.role).toBe("separator");
    expect(separatorProps["aria-orientation"]).toBe("horizontal");
    expect(separatorProps["aria-valuenow"]).toBe(65);
    expect(separatorProps.tabIndex).toBe(0);
  });

  it("computes aria-valuemin and aria-valuemax from container size", () => {
    const result = renderWithRealDom({
      minTop: 100,
      minBottom: 150,
      storageKey: STORAGE_KEY + "-4",
    });

    expect(result.current.separatorProps["aria-valuemin"]).toBe(10);
    expect(result.current.separatorProps["aria-valuemax"]).toBe(85);
  });

  it("adjusts ratio with ArrowUp key", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-5" });

    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "ArrowUp",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    expect(result.current.separatorProps["aria-valuenow"]).toBe(64);
  });

  it("adjusts ratio with ArrowDown key", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-6" });

    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "ArrowDown",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    expect(result.current.separatorProps["aria-valuenow"]).toBe(66);
  });

  it("jumps to min with Home key", () => {
    const result = renderWithRealDom({
      minTop: 100,
      minBottom: 150,
      storageKey: STORAGE_KEY + "-7",
    });

    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "Home",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    expect(result.current.separatorProps["aria-valuenow"]).toBe(10);
  });

  it("jumps to max with End key", () => {
    const result = renderWithRealDom({
      minTop: 100,
      minBottom: 150,
      storageKey: STORAGE_KEY + "-8",
    });

    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "End",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    expect(result.current.separatorProps["aria-valuenow"]).toBe(85);
  });

  it("resets to default ratio on double-click", () => {
    const result = renderWithRealDom({
      initialRatio: 0.5,
      storageKey: STORAGE_KEY + "-9",
    });

    // Change ratio first via keyboard.
    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "ArrowDown",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    // Double-click should reset to initialRatio (0.5).
    act(() => {
      result.current.separatorProps.onDoubleClick();
    });

    expect(result.current.isCollapsed).toBe(false);
    expect(result.current.topHeight).toBe(500);
  });

  it("toggleCollapse then double-click resets to initial ratio", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-10" });

    // Collapse via toggleCollapse.
    act(() => {
      result.current.toggleCollapse();
    });
    expect(result.current.isCollapsed).toBe(true);

    // Double-click restores (resets to initialRatio and uncollapses).
    act(() => {
      result.current.separatorProps.onDoubleClick();
    });
    expect(result.current.isCollapsed).toBe(false);
    expect(result.current.topHeight).toBe(650);
  });

  it("toggleCollapse toggles state", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-11" });

    act(() => {
      result.current.toggleCollapse();
    });
    expect(result.current.isCollapsed).toBe(true);

    act(() => {
      result.current.toggleCollapse();
    });
    expect(result.current.isCollapsed).toBe(false);
  });

  it("persists ratio to localStorage on keyboard change", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY });

    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "ArrowDown",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    const stored = localStorage.getItem(STORAGE_KEY);
    expect(stored).toBe("0.66");
  });

  it("ignores non-handled keys", () => {
    const result = renderWithRealDom({ storageKey: STORAGE_KEY + "-12" });
    const initial = result.current.separatorProps["aria-valuenow"];

    act(() => {
      result.current.separatorProps.onKeyDown({
        key: "Enter",
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent);
    });

    expect(result.current.separatorProps["aria-valuenow"]).toBe(initial);
  });
});
