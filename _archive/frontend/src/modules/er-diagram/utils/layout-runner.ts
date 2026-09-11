import { computeApproximateLayoutFromInput } from "./approximate-layout";
import {
  computeLayoutPositions,
  type LayoutInput,
  type LayoutOptions,
  type LayoutPosition,
} from "./layout";

export interface LayoutRunRequest {
  requestId: number;
  input: LayoutInput;
  options: LayoutOptions;
  /** Option C — the worker posts progressive force-refined stages before the final. */
  progressive?: boolean;
}

export interface LayoutRunResult {
  requestId: number;
  positions: Record<string, LayoutPosition>;
  layoutMs: number;
  /** "refine" = progressive intermediate stage (Option C); "final" = the real layout. */
  stage?: "refine" | "final";
}

/**
 * What a runner actually computes — lets callers distinguish "real dagre in a
 * worker" from "degraded" results without re-deriving the graph size (P1-3).
 */
export type LayoutRunnerKind = "worker" | "dagre-sync" | "approximate";

export interface LayoutRunner {
  kind: LayoutRunnerKind;
  run(
    request: LayoutRunRequest,
    onProgress?: (result: LayoutRunResult) => void,
  ): Promise<LayoutRunResult>;
  dispose(): void;
}

/**
 * Main-thread client for `er-layout.worker.ts`. Each request is matched back
 * by `requestId`; worker errors reject every in-flight request. Callers are
 * responsible for discarding stale results (compare `requestId`).
 */
export function createWorkerLayoutRunner(): LayoutRunner {
  if (typeof Worker === "undefined") {
    // P1-3 — NEVER silently fall back to synchronous dagre here: a large graph
    // would then run dagre on the main thread (8 s @500 / 122 s @1000 per
    // P1.8) while the size gate in use-worker-layout.ts is bypassed. Throw so
    // resolveLayoutRunner routes to the size-gated getFallbackRunner.
    throw new Error("Worker is unavailable in this environment");
  }

  const worker = new Worker(new URL("../er-layout.worker.ts", import.meta.url), {
    type: "module",
  });
  const pending = new Map<
    number,
    {
      resolve: (result: LayoutRunResult) => void;
      reject: (error: Error) => void;
      onProgress?: (result: LayoutRunResult) => void;
    }
  >();

  worker.onmessage = (event: MessageEvent<LayoutRunResult>) => {
    const msg = event.data;
    const entry = pending.get(msg.requestId);
    if (!entry) return; // stale response already discarded by caller
    if (msg.stage === "refine") {
      // Option C — progressive intermediate: forward without resolving.
      entry.onProgress?.(msg);
      return;
    }
    pending.delete(msg.requestId);
    entry.resolve(msg);
  };

  worker.onerror = (event) => {
    const error = new Error(event.message || "layout worker error");
    for (const entry of pending.values()) entry.reject(error);
    pending.clear();
  };

  return {
    kind: "worker",
    run(request, onProgress) {
      return new Promise((resolve, reject) => {
        pending.set(request.requestId, { resolve, reject, onProgress });
        worker.postMessage(request);
      });
    },
    dispose() {
      worker.terminate();
    },
  };
}

/**
 * Synchronous dagre fallback for environments without Workers (jsdom tests,
 * unusual webviews). Blocks the main thread — acceptable only as a safety net
 * and ONLY for small graphs (P1-3): production Tauri webviews all support
 * module workers, and large graphs must never run synchronous dagre.
 */
export function createFallbackLayoutRunner(): LayoutRunner {
  return {
    kind: "dagre-sync",
    run(request) {
      const t0 = performance.now();
      const positions = computeLayoutPositions(request.input, request.options);
      const out: Record<string, LayoutPosition> = {};
      positions.forEach((pos, id) => {
        out[id] = pos;
      });
      return Promise.resolve({
        requestId: request.requestId,
        positions: out,
        layoutMs: performance.now() - t0,
      });
    },
    dispose() {
      /* no-op */
    },
  };
}

/**
 * P1-3 — approximate fallback for LARGE graphs when the worker is unavailable.
 * Never runs dagre on the main thread for big graphs (8 s @500 / 122 s @1000
 * per P1.8): returns a deterministic degree-ordered circle in O(N log N)
 * instead. The host shows a degraded-layout notice and can still retry the
 * worker on the next run (F-MR-2 non-sticky semantics).
 */
export function createApproximateLayoutRunner(): LayoutRunner {
  return {
    kind: "approximate",
    run(request) {
      const t0 = performance.now();
      const positions = computeApproximateLayoutFromInput(request.input);
      const out: Record<string, LayoutPosition> = {};
      positions.forEach((pos, id) => {
        out[id] = pos;
      });
      return Promise.resolve({
        requestId: request.requestId,
        positions: out,
        layoutMs: performance.now() - t0,
      });
    },
    dispose() {
      /* no-op */
    },
  };
}
