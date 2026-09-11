/**
 * P1.7 — Layout Web Worker.
 *
 * Receives a plain `LayoutInput` (node ids + estimated heights, edge topology,
 * options) over the boundary and runs layout here, off the main thread.
 *
 * Option C (progressive layout quality): when `progressive` is set (large-schema
 * overview), the worker first posts a sequence of `stage: "refine"` position
 * sets — deterministic Fruchterman-Reingold passes that pull connected tables
 * together — then the single `stage: "final"` dagre result. Every message is a
 * FULL, stable position set (never a partial/unstable stream), so each stage is
 * a valid atomic commit; the user sees the overview progressively improve
 * instead of staring at a bare circle for 8–122 s.
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
import { computeApproximateLayoutFromInput } from "./utils/approximate-layout";
import {
  computeOptimalDistance,
  FORCE_REFINE_ALPHA,
  PROGRESSIVE_MIN_NODES,
  refinePositions,
} from "./utils/force-refine";

interface LayoutWorkerRequest {
  requestId: number;
  input: LayoutInput;
  options: LayoutOptions;
  /** Option C — post progressive force-refined stages before the dagre final. */
  progressive?: boolean;
}

interface LayoutWorkerResponse {
  requestId: number;
  stage: "refine" | "final";
  positions: Record<string, LayoutPosition>;
  layoutMs: number;
}

interface LayoutWorkerScope {
  onmessage: ((event: MessageEvent<LayoutWorkerRequest>) => void) | null;
  postMessage(message: LayoutWorkerResponse): void;
}

const scope = self as unknown as LayoutWorkerScope;

// Option C refinement schedule: 30 FR passes total, posting after every 6, then
// dagre. 30 passes ≈ tens of ms at 1000 tables — the first stage lands well
// before the user notices the circle, and dagre still wins the final quality.
const REFINE_TOTAL_PASSES = 30;
const REFINE_POST_EVERY = 6;

function toRecord(positions: Map<string, LayoutPosition>): Record<string, LayoutPosition> {
  const out: Record<string, LayoutPosition> = {};
  positions.forEach((pos, id) => {
    out[id] = pos;
  });
  return out;
}

scope.onmessage = (event: MessageEvent<LayoutWorkerRequest>) => {
  const { requestId, input, options, progressive } = event.data;

  if (progressive && input.nodes.length >= PROGRESSIVE_MIN_NODES) {
    const k = computeOptimalDistance(input);
    let positions = computeApproximateLayoutFromInput(input);
    let temperature = k;

    for (let pass = 1; pass <= REFINE_TOTAL_PASSES; pass++) {
      const t0 = performance.now();
      positions = refinePositions(input, positions, {
        iterations: 1,
        k,
        temperature,
      });
      temperature *= FORCE_REFINE_ALPHA;
      if (pass % REFINE_POST_EVERY === 0) {
        scope.postMessage({
          requestId,
          stage: "refine",
          positions: toRecord(positions),
          layoutMs: performance.now() - t0,
        });
      }
    }
  }

  const t0 = performance.now();
  const positions = computeLayoutPositions(input, options);
  scope.postMessage({
    requestId,
    stage: "final",
    positions: toRecord(positions),
    layoutMs: performance.now() - t0,
  });
};
