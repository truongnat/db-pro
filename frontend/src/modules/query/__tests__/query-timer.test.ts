import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useQueryTimer } from "../hooks/use-query-timer";

describe("useQueryTimer", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(1000000);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns 0 when status is idle and no startedAt", () => {
    const { result } = renderHook(() => useQueryTimer("idle", null));
    expect(result.current).toBe(0);
  });

  it("returns 0 when status is idle even with startedAt", () => {
    const { result } = renderHook(() => useQueryTimer("idle", 999000));
    expect(result.current).toBe(0);
  });

  it("returns elapsed time when status is running", () => {
    const startedAt = 999500; // 500ms ago
    const { result } = renderHook(() => useQueryTimer("running", startedAt));
    // Initial render sets elapsed
    expect(result.current).toBe(500);
  });

  it("updates elapsed over time while running", () => {
    const startedAt = 999000; // 1000ms ago at start
    const { result } = renderHook(() => useQueryTimer("running", startedAt));

    expect(result.current).toBe(1000);

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current).toBe(1500);
  });

  it("freezes elapsed when status changes from running to success", () => {
    const startedAt = 999000;
    const { result, rerender } = renderHook(
      ({ status }: { status: "running" | "success" }) => useQueryTimer(status, startedAt),
      { initialProps: { status: "running" as const } },
    );

    expect(result.current).toBe(1000);

    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(result.current).toBe(1300);

    // Change status to success — timer should stop updating
    rerender({ status: "success" });

    act(() => {
      vi.advanceTimersByTime(1000);
    });

    // Should remain at the last running value
    expect(result.current).toBe(1300);
  });

  it("resets to 0 when returning to idle with null startedAt", () => {
    const { result, rerender } = renderHook(
      ({ status, startedAt }: { status: "idle" | "running" | "success"; startedAt: number | null }) =>
        useQueryTimer(status, startedAt),
      { initialProps: { status: "running" as const, startedAt: 999000 } },
    );

    expect(result.current).toBe(1000);

    rerender({ status: "idle", startedAt: null });
    expect(result.current).toBe(0);
  });
});
