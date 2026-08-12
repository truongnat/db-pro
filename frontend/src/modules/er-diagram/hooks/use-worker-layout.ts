import { useEffect, useMemo, useState } from "react";
import type { LayoutInput, LayoutOptions, LayoutPosition } from "../utils/layout";
import { computeLayoutHash } from "../utils/layout-hash";
import { LayoutCache } from "../utils/layout-cache";
import {
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
  /** dagre duration in ms (worker, fallback, or cache). */
  layoutMs: number | null;
  nodeCount: number;
  fromCache: boolean;
  error: string | null;
}

const IDLE_STATE: WorkerLayoutState = {
  status: "idle",
  positions: null,
  layoutMs: null,
  nodeCount: 0,
  fromCache: false,
  error: null,
};

// The ER diagram mounts once per connection/schema view — module-level
// singletons avoid re-spawning the worker on every dependency change.
let runnerSingleton: LayoutRunner | null = null;
let fallbackSingleton: LayoutRunner | null = null;

function getRunner(): LayoutRunner {
  if (!runnerSingleton) {
    try {
      runnerSingleton = createWorkerLayoutRunner();
    } catch {
      runnerSingleton = createFallbackLayoutRunner();
    }
  }
  return runnerSingleton;
}

function getFallbackRunner(): LayoutRunner {
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
        error: null,
      });
      return;
    }

    const requestId = ++requestSeq;
    setState((prev) => ({
      ...prev,
      status: "computing",
      fromCache: false,
      error: null,
      nodeCount: nodeIds.length,
    }));

    let cancelled = false;

    // A result is cached regardless of staleness (it is valid for its hash);
    // the UI state only updates for the current request.
    const commit = (result: LayoutRunResult) => {
      layoutCache.set({
        hash,
        positions: result.positions,
        layoutMs: result.layoutMs,
        nodeIds,
        createdAt: Date.now(),
      });
      if (cancelled || result.requestId !== requestSeq) return;
      setState({
        status: "ready",
        positions: toPositionsMap(result.positions),
        layoutMs: result.layoutMs,
        nodeCount: nodeIds.length,
        fromCache: false,
        error: null,
      });
    };

    getRunner()
      .run({ requestId, input, options })
      .then(commit)
      .catch((_err: unknown) => {
        if (cancelled || requestId !== requestSeq) return;
        // Worker runtime failure (module-worker unsupported in an old webview,
        // dagre crash, …) → one synchronous fallback attempt, then error.
        getFallbackRunner()
          .run({ requestId, input, options })
          .then(commit)
          .catch((fallbackErr: unknown) => {
            if (cancelled || requestId !== requestSeq) return;
            setState((prev) => ({
              ...prev,
              status: "error",
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
