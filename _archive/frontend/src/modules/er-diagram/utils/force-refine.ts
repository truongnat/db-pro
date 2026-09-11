import type { LayoutInput, LayoutPosition } from "./layout";

/**
 * Option C (review F-REV) — deterministic force-directed refinement.
 *
 * The overview paints an approximate circle in <1 s, but the user then stares
 * at that bare circle for 8–122 s until dagre finishes (P1.8 evidence). This
 * module closes the gap: a Fruchterman-Reingold refinement (uniform-grid
 * repulsion, spring attraction along edges) pulls connected tables together
 * and spreads clusters apart, so the worker can post progressively better
 * COMPLETE position sets while dagre computes.
 *
 * - Pure + deterministic: same input + same initial positions → same output
 *   (fixed iteration order, no randomness). Runs in the layout worker.
 * - Time-bounded by construction: `iterations` is explicit, and each iteration
 *   is O(N·density + E) via the uniform grid — a few ms at 1000 tables.
 * - Each stage is a full, stable set (not a partial/unstable stream), so the
 *   "atomic commit" contract is preserved — the pipeline just delivers more
 *   than one commit: circle → refined stages → dagre final.
 */
export const FORCE_REFINE_ALPHA = 0.92;

/** Below this node count dagre is fast (151 ms @100 per P1.8) — refinement adds nothing. */
export const PROGRESSIVE_MIN_NODES = 120;

export interface ForceRefineOptions {
  /** Number of FR iterations to run. */
  iterations: number;
  /** Optimal edge length k. Defaults to computeOptimalDistance(input). */
  k?: number;
  /** Starting temperature (max displacement per iteration). Defaults to k. */
  temperature?: number;
  /** Cooling factor per iteration (0..1). Defaults to FORCE_REFINE_ALPHA. */
  cooling?: number;
}

/**
 * Optimal edge length derived from input geometry — deterministic and shared
 * between the worker's chunks so k does not drift between refinement stages.
 */
export function computeOptimalDistance(input: LayoutInput): number {
  const n = Math.max(input.nodes.length, 1);
  let totalArea = 0;
  for (const node of input.nodes) {
    totalArea += (node.width ?? 160) * (node.height ?? 28);
  }
  // ~2.4× the average node footprint — generous overview spacing that keeps
  // 160×28 chips from overlapping while clustering connected tables tightly.
  return Math.max(48, Math.sqrt(totalArea / n) * 2.4);
}

/**
 * Run `iterations` Fruchterman-Reingold passes over `initial`, returning a new
 * complete position map. Input node ids are iterated in input order (stable),
 * repulsion is accelerated with a uniform grid (3×3 neighborhood), and the
 * result is re-centered each pass so the graph does not drift.
 */
export function refinePositions(
  input: LayoutInput,
  initial: Map<string, LayoutPosition>,
  options: ForceRefineOptions,
): Map<string, LayoutPosition> {
  const { iterations, cooling = FORCE_REFINE_ALPHA } = options;
  const k = options.k ?? computeOptimalDistance(input);
  let temperature = options.temperature ?? k;

  const pos = new Map<string, LayoutPosition>();
  for (const node of input.nodes) {
    const p = initial.get(node.id);
    pos.set(node.id, p ? { x: p.x, y: p.y } : { x: 0, y: 0 });
  }
  const ids = input.nodes.map((n) => n.id);
  const cellSize = k * 2;

  for (let iteration = 0; iteration < iterations; iteration++) {
    const disp = new Map<string, { x: number; y: number }>();
    for (const id of ids) disp.set(id, { x: 0, y: 0 });

    // Uniform grid for repulsion — O(N·density) instead of O(N²).
    const grid = new Map<string, string[]>();
    for (const id of ids) {
      const p = pos.get(id)!;
      const key = `${Math.floor(p.x / cellSize)},${Math.floor(p.y / cellSize)}`;
      const bucket = grid.get(key);
      if (bucket) bucket.push(id);
      else grid.set(key, [id]);
    }

    // Repulsion within the 3×3 cell neighborhood.
    for (const id of ids) {
      const p = pos.get(id)!;
      const cx = Math.floor(p.x / cellSize);
      const cy = Math.floor(p.y / cellSize);
      for (let gx = cx - 1; gx <= cx + 1; gx++) {
        for (let gy = cy - 1; gy <= cy + 1; gy++) {
          const bucket = grid.get(`${gx},${gy}`);
          if (!bucket) continue;
          for (const other of bucket) {
            if (other === id) continue;
            const q = pos.get(other)!;
            let dx = p.x - q.x;
            let dy = p.y - q.y;
            let d2 = dx * dx + dy * dy;
            if (d2 < 1e-6) {
              // De-overlap colliding nodes with a deterministic push.
              d2 = 1e-6;
              dx = 1;
              dy = 0;
            }
            const d = Math.sqrt(d2);
            const force = (k * k) / d;
            const ux = dx / d;
            const uy = dy / d;
            const dSelf = disp.get(id)!;
            dSelf.x += ux * force;
            dSelf.y += uy * force;
            const dOther = disp.get(other)!;
            dOther.x -= ux * force;
            dOther.y -= uy * force;
          }
        }
      }
    }

    // Attraction along edges (spring force).
    for (const edge of input.edges) {
      const s = pos.get(edge.source);
      const t = pos.get(edge.target);
      if (!s || !t) continue;
      let dx = s.x - t.x;
      let dy = s.y - t.y;
      let d2 = dx * dx + dy * dy;
      if (d2 < 1e-6) {
        d2 = 1e-6;
        dx = 1;
        dy = 0;
      }
      const d = Math.sqrt(d2);
      const force = (d * d) / k;
      const ux = dx / d;
      const uy = dy / d;
      const dSource = disp.get(edge.source)!;
      dSource.x -= ux * force;
      dSource.y -= uy * force;
      const dTarget = disp.get(edge.target)!;
      dTarget.x += ux * force;
      dTarget.y += uy * force;
    }

    // Apply displacements capped by the cooling temperature, then re-center.
    let sumX = 0;
    let sumY = 0;
    for (const id of ids) {
      const p = pos.get(id)!;
      const d = disp.get(id)!;
      const len = Math.hypot(d.x, d.y);
      if (len > 0) {
        const mag = Math.min(len, temperature);
        p.x += (d.x / len) * mag;
        p.y += (d.y / len) * mag;
      }
      sumX += p.x;
      sumY += p.y;
    }
    const centerX = sumX / ids.length;
    const centerY = sumY / ids.length;
    for (const id of ids) {
      const p = pos.get(id)!;
      p.x -= centerX;
      p.y -= centerY;
    }

    temperature *= cooling;
  }

  return pos;
}

/**
 * Mean Euclidean edge length over the input's edges — the quality metric the
 * harness/tests use to prove a refined layout is "better" (connected tables
 * pulled together ⇒ shorter edges) than the circle it started from.
 */
export function meanEdgeLength(input: LayoutInput, positions: Map<string, LayoutPosition>): number {
  let total = 0;
  let count = 0;
  for (const edge of input.edges) {
    const s = positions.get(edge.source);
    const t = positions.get(edge.target);
    if (!s || !t) continue;
    total += Math.hypot(s.x - t.x, s.y - t.y);
    count++;
  }
  return count > 0 ? total / count : 0;
}
