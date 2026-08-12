/**
 * P1.7 — Layout Web Worker.
 *
 * Receives a plain `LayoutInput` (node ids + estimated heights, edge topology,
 * options) over the boundary, runs the full dagre computation here, and posts
 * back a single atomic `LayoutRunResult`. The main thread never receives
 * partial/unstable positions — the locked architecture's "no fake progressive
 * layout" rule.
 *
 * The `scope` cast avoids pulling the DOM lib's Window typing (with its
 * different `postMessage` signature) into the worker file; Vite bundles this
 * file as a dedicated module worker.
 */
import {
  computeLayoutPositions,
  type LayoutInput,
  type LayoutOptions,
  type LayoutPosition,
} from "./utils/layout";

interface LayoutWorkerRequest {
  requestId: number;
  input: LayoutInput;
  options: LayoutOptions;
}

interface LayoutWorkerResponse {
  requestId: number;
  positions: Record<string, LayoutPosition>;
  layoutMs: number;
}

interface LayoutWorkerScope {
  onmessage: ((event: MessageEvent<LayoutWorkerRequest>) => void) | null;
  postMessage(message: LayoutWorkerResponse): void;
}

const scope = self as unknown as LayoutWorkerScope;

scope.onmessage = (event: MessageEvent<LayoutWorkerRequest>) => {
  const { requestId, input, options } = event.data;
  const t0 = performance.now();
  const positions = computeLayoutPositions(input, options);
  const layoutMs = performance.now() - t0;

  const out: Record<string, LayoutPosition> = {};
  positions.forEach((pos, id) => {
    out[id] = pos;
  });

  scope.postMessage({ requestId, positions: out, layoutMs });
};
