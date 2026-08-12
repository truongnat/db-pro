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
}

export interface LayoutRunResult {
  requestId: number;
  positions: Record<string, LayoutPosition>;
  layoutMs: number;
}

export interface LayoutRunner {
  run(request: LayoutRunRequest): Promise<LayoutRunResult>;
  dispose(): void;
}

/**
 * Main-thread client for `er-layout.worker.ts`. Each request is matched back
 * by `requestId`; worker errors reject every in-flight request. Callers are
 * responsible for discarding stale results (compare `requestId`).
 */
export function createWorkerLayoutRunner(): LayoutRunner {
  if (typeof Worker === "undefined") {
    return createFallbackLayoutRunner();
  }

  const worker = new Worker(new URL("../er-layout.worker.ts", import.meta.url), {
    type: "module",
  });
  const pending = new Map<
    number,
    { resolve: (result: LayoutRunResult) => void; reject: (error: Error) => void }
  >();

  worker.onmessage = (event: MessageEvent<LayoutRunResult>) => {
    const msg = event.data;
    const entry = pending.get(msg.requestId);
    if (!entry) return; // stale response already discarded by caller
    pending.delete(msg.requestId);
    entry.resolve(msg);
  };

  worker.onerror = (event) => {
    const error = new Error(event.message || "layout worker error");
    for (const entry of pending.values()) entry.reject(error);
    pending.clear();
  };

  return {
    run(request) {
      return new Promise((resolve, reject) => {
        pending.set(request.requestId, { resolve, reject });
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
 * unusual webviews). Blocks the main thread — acceptable only as a safety net;
 * production Tauri webviews all support module workers.
 */
export function createFallbackLayoutRunner(): LayoutRunner {
  return {
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
