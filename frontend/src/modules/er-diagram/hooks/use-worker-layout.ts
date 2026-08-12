import { useEffect, useMemo, useState } from "react";
import type { LayoutInput, LayoutOptions, LayoutPosition } from "../utils/layout";
import { computeLayoutHash } from "../utils/layout-hash";
import { LayoutCache } from "../utils/layout-cache";
import {
  createApproximateLayoutRunner,
  createFallbackLayoutRunner,
  createWorkerLayoutRunner,
  type LayoutRunner,
  type LayoutRunResult,
} from "../utils/layout-runner";

export type LayoutStatus = "idle" | "computing" | "ready" | "error";

export interface WorkerLayoutState {
  status: LayoutStatus;
  /** null while computing; the full, stable position set once ready. */
  positions: Map<string, LayoutPosition> | null;
  /** layout duration in ms (worker, fallback, approximate, or cache). */
  layoutMs: number | null;
  nodeCount: number;
  fromCache: boolean;
  /**
   * P1-3 — true when positions came from the approximate runner (worker
   * unavailable on a large graph). Hosts show a degraded-layout notice; the
   * worker is retried on the next run (F-MR-2 non-sticky semantics).
   */
  degraded: boolean;
  error: string | null;
}

/**
 * P1-3 — graphs above this node count NEVER run synchronous dagre on the main
 * thread. P1.8 measured dagre at 151 ms @100 but 8,110 ms @500; 250 is a
 * conservative bound that keeps the sync fallback a sub-second safety net
 * while guaranteeing large graphs stay off the main thread.
 */
export const SYNC_FALLBACK_MAX_NODES = 250;

const IDLE_STATE: WorkerLayoutState = {
  status: "idle",
  positions: null,
  layoutMs: null,
  nodeCount: 0,
  fromCache: false,
  degraded: false,
  error: null,
};

// The ER diagram mounts once per connection/schema view — module-level
// singletons avoid re-spawning the worker on every dependency change.
let runnerSingleton: LayoutRunner | null = null;
let fallbackSingleton: LayoutRunner | null = null;

/**
 * Pick the layout runner, retrying worker creation on failure (F-MR-2).
 *
 * A synchronous worker-creation failure (module workers unsupported in an old
 * webview, transient init error, …) must NOT permanently downgrade the session
 * to the main-thread fallback. `sticky: false` tells the caller to use the
 * fallback for this attempt only and retry the worker on the next run.
 */
export function resolveLayoutRunner(
  current: LayoutRunner | null,
  create: () => LayoutRunner,
  fallback: () => LayoutRunner,
): { runner: LayoutRunner; sticky: boolean } {
  if (current) return { runner: current, sticky: true };
  try {
    return { runner: create(), sticky: true };
  } catch {
    return { runner: fallback(), sticky: false };
  }
}

function getRunner(nodeCount: number): { runner: LayoutRunner; degraded: boolean } {
  const selection = resolveLayoutRunner(runnerSingleton, createWorkerLayoutRunner, () =>
    getFallbackRunner(nodeCount),
  );
  if (selection.sticky) runnerSingleton = selection.runner;
  // P1-3 — worker creation can fail synchronously (no Worker global, module
  // workers unsupported, …). When it does, the size-gated fallback runs:
  // approximate for large graphs (degraded), sync dagre for small. The degraded
  // flag must reflect the runner that actually produced the result — a large
  // graph must never be marked healthy (approximate positions would then be
  // written to the schemaHash cache and shown without the degraded notice).
  return { runner: selection.runner, degraded: selection.runner.kind === "approximate" };
}

/**
 * P1-3 — the fallback depends on graph size: small graphs may use synchronous
 * dagre (sub-second); large graphs get the approximate runner instead and
 * never block the main thread.
 */
function getFallbackRunner(nodeCount: number): LayoutRunner {
  if (nodeCount > SYNC_FALLBACK_MAX_NODES) return createApproximateLayoutRunner();
  if (!fallbackSingleton) fallbackSingleton = createFallbackLayoutRunner();
  return fallbackSingleton;
}

export const layoutCache = new LayoutCache();

// Module-level so multiple hook instances (if ever mounted) cannot collide on
// the shared runner's requestId keys.
let requestSeq = 0;

/**
 * P1.7 — layout off the main thread, atomically committed.
 *
 * Flow per the locked architecture:
 *
 *   schema metadata → hash → cache hit? → commit immediately
 *                              ↘ miss → Worker dagre → atomic commit once
 *
 * - `input` identity must be stable across renders (wrap in `useMemo`); the
 *   hash only changes when the graph topology / sizes / options actually
 *   change, so layout runs once per distinct graph (direction toggles and
 *   neighborhood scope changes included).
 * - Stale results are discarded (effect cleanup + requestId); results are
 *   still cached, since a superseded result is valid for its own hash.
 * - Worker runtime failures fall back to the synchronous dagre runner once
 *   before surfacing an error state.
 * - The commit is atomic by construction — a single `setState` with the full
 *   position Map; the component applies it in one pass.
 */
export function useWorkerLayout(
  input: LayoutInput | null,
  options: LayoutOptions,
): WorkerLayoutState {
  const hash = useMemo(
    () => (input && input.nodes.length > 0 ? computeLayoutHash(input, options) : null),
    [input, options],
  );

  const [state, setState] = useState<WorkerLayoutState>(IDLE_STATE);

  useEffect(() => {
    if (!hash || !input) {
      setState(IDLE_STATE);
      return;
    }

    const nodeIds = input.nodes.map((n) => n.id);

    // Cache-first: identical graph + options never touch the worker.
    const cached = layoutCache.get(hash, nodeIds);
    if (cached) {
      setState({
        status: "ready",
        positions: toPositionsMap(cached.positions),
        layoutMs: cached.layoutMs,
        nodeCount: nodeIds.length,
        fromCache: true,
        degraded: false,
        error: null,
      });
      return;
    }

    const requestId = ++requestSeq;
    setState((prev) => ({
      ...prev,
      status: "computing",
      fromCache: false,
      degraded: false,
      error: null,
      nodeCount: nodeIds.length,
    }));

    let cancelled = false;

    // A result is cached regardless of staleness (it is valid for its hash);
    // the UI state only updates for the current request. Degraded (approximate)
    // results are NOT cached — they are not dagre output for this hash, and
    // caching them would poison the layout cache for the next (healthy) run.
    const commit = (result: LayoutRunResult, degraded: boolean) => {
      if (!degraded) {
        layoutCache.set({
          hash,
          positions: result.positions,
          layoutMs: result.layoutMs,
          nodeIds,
          createdAt: Date.now(),
        });
      }
      if (cancelled || result.requestId !== requestSeq) return;
      setState({
        status: "ready",
        positions: toPositionsMap(result.positions),
        layoutMs: result.layoutMs,
        nodeCount: nodeIds.length,
        fromCache: false,
        degraded,
        error: null,
      });
    };

    const { runner, degraded: creationDegraded } = getRunner(nodeIds.length);
    runner
      .run({ requestId, input, options })
      .then((result) => commit(result, creationDegraded))
      .catch((_err: unknown) => {
        if (cancelled || requestId !== requestSeq) return;
        // Worker runtime failure (module-worker unsupported in an old webview,
        // dagre crash, …) → one fallback attempt, then error. P1-3: for large
        // graphs the fallback is the approximate runner (no main-thread dagre),
        // flagged degraded — never cached.
        const fallback = getFallbackRunner(nodeIds.length);
        fallback
          .run({ requestId, input, options })
          .then((result) => commit(result, fallback.kind === "approximate"))
          .catch((fallbackErr: unknown) => {
            if (cancelled || requestId !== requestSeq) return;
            setState((prev) => ({
              ...prev,
              status: "error",
              degraded: false,
              error: String((fallbackErr as Error)?.message ?? fallbackErr),
            }));
          });
      });

    return () => {
      cancelled = true;
    };
  }, [hash, input, options]);

  return state;
}

function toPositionsMap(positions: Record<string, LayoutPosition>): Map<string, LayoutPosition> {
  const map = new Map<string, LayoutPosition>();
  for (const [id, pos] of Object.entries(positions)) map.set(id, pos);
  return map;
}
