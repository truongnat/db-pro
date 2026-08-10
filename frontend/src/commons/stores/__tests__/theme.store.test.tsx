import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  resolveThemeMode,
  useThemeStore,
  useResolvedTheme,
} from "@/commons/stores/theme.store";

function mockMatchMedia(matches: boolean) {
  const listeners: Array<(e: { matches: boolean }) => void> = [];
  const mql = {
    matches,
    media: "(prefers-color-scheme: dark)",
    addEventListener: vi.fn((_evt: string, cb: (e: { matches: boolean }) => void) => {
      listeners.push(cb);
    }),
    removeEventListener: vi.fn(),
    onchange: null,
  };
  const original = window.matchMedia;
  window.matchMedia = vi.fn().mockReturnValue(mql);
  return {
    mql,
    listeners,
    restore: () => {
      window.matchMedia = original;
    },
  };
}

describe("resolveThemeMode", () => {
  it("returns the mode directly for explicit light/dark", () => {
    expect(resolveThemeMode("light")).toBe("light");
    expect(resolveThemeMode("dark")).toBe("dark");
  });

  it("resolves system → dark when prefers-color-scheme is dark", () => {
    const { restore } = mockMatchMedia(true);
    expect(resolveThemeMode("system")).toBe("dark");
    restore();
  });

  it("resolves system → light when prefers-color-scheme is light", () => {
    const { restore } = mockMatchMedia(false);
    expect(resolveThemeMode("system")).toBe("light");
    restore();
  });
});

describe("useThemeStore", () => {
  beforeEach(() => {
    useThemeStore.setState({ mode: "system" });
  });

  afterEach(() => {
    useThemeStore.setState({ mode: "system" });
  });

  it("cycles system → light → dark → system via toggle()", () => {
    expect(useThemeStore.getState().mode).toBe("system");

    act(() => useThemeStore.getState().toggle());
    expect(useThemeStore.getState().mode).toBe("light");

    act(() => useThemeStore.getState().toggle());
    expect(useThemeStore.getState().mode).toBe("dark");

    act(() => useThemeStore.getState().toggle());
    expect(useThemeStore.getState().mode).toBe("system");
  });

  it("setMode sets the mode directly", () => {
    act(() => useThemeStore.getState().setMode("dark"));
    expect(useThemeStore.getState().mode).toBe("dark");
  });
});

describe("useResolvedTheme", () => {
  beforeEach(() => {
    useThemeStore.setState({ mode: "system" });
  });

  afterEach(() => {
    useThemeStore.setState({ mode: "system" });
  });

  function ThemeDisplay() {
    const theme = useResolvedTheme();
    return <div data-testid="theme">{theme}</div>;
  }

  it("reflects explicit mode changes", () => {
    const { restore } = mockMatchMedia(false);
    useThemeStore.setState({ mode: "light" });
    render(<ThemeDisplay />);
    expect(screen.getByTestId("theme").textContent).toBe("light");

    act(() => useThemeStore.setState({ mode: "dark" }));
    expect(screen.getByTestId("theme").textContent).toBe("dark");
    restore();
  });

  it("reacts to OS prefers-color-scheme changes in system mode", () => {
    const { mql, listeners, restore } = mockMatchMedia(false);

    useThemeStore.setState({ mode: "system" });
    render(<ThemeDisplay />);
    expect(screen.getByTestId("theme").textContent).toBe("light");

    act(() => {
      mql.matches = true;
      for (const cb of listeners) cb({ matches: true });
    });
    expect(screen.getByTestId("theme").textContent).toBe("dark");

    restore();
  });
});
