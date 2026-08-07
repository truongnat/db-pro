import { useEffect, useState } from "react";

/**
 * Returns live elapsed milliseconds since `startedAt` (a `Date.now()` timestamp).
 * While `status === "running"` the value updates every 100 ms so the UI can
 * show a running timer.  Once the status changes away from "running" the
 * elapsed value is frozen at the final reading.
 */
export function useQueryTimer(
  status: "idle" | "running" | "success" | "error",
  startedAt: number | null,
): number {
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (status !== "running" || startedAt == null) {
      return;
    }

    // Set initial value immediately
    setElapsed(Date.now() - startedAt);

    const id = setInterval(() => {
      setElapsed(Date.now() - startedAt);
    }, 100);

    return () => clearInterval(id);
  }, [status, startedAt]);

  // Reset when idle and no start timestamp
  useEffect(() => {
    if (status === "idle" && startedAt == null) {
      setElapsed(0);
    }
  }, [status, startedAt]);

  return elapsed;
}
