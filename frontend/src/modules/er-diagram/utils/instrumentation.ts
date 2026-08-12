/**
 * P1.1 — ER diagram runtime instrumentation.
 *
 * Measures the locked P1 acceptance metrics:
 *   - time to interactive shell (init marks)
 *   - layout duration (main thread until P1.7 moves it to a Worker)
 *   - max main-thread long task
 *   - initial detailed DOM count
 *   - pan/zoom frame time
 *   - graph / viewport / rendered / detailed node counts + rendered edge count
 *
 * Pure helper `computeFrameStats` is unit-tested; `ErPerfMonitor` is
 * `performance`/DOM-backed and used by the dev HUD overlay.
 *
 * Viewport/spatial queries live in `../utils/spatial-index` (P1.5).
 */

export interface FrameStats {
  samples: number;
  avgMs: number;
  maxMs: number;
  p95Ms: number;
}

export interface LongTaskRecord {
  start: number;
  duration: number;
}

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const idx = Math.min(sorted.length - 1, Math.floor(sorted.length * p));
  return sorted[idx];
}

/** Frame-time statistics over a collected sample set (ms). */
export function computeFrameStats(samples: number[]): FrameStats {
  if (samples.length === 0) return { samples: 0, avgMs: 0, maxMs: 0, p95Ms: 0 };
  const sorted = [...samples].sort((a, b) => a - b);
  const sum = sorted.reduce((a, b) => a + b, 0);
  return {
    samples: samples.length,
    avgMs: sum / samples.length,
    maxMs: sorted[sorted.length - 1],
    p95Ms: percentile(sorted, 0.95),
  };
}

export class ErPerfMonitor {
  private container: () => HTMLElement | null;
  private longTasks: LongTaskRecord[] = [];
  private observer: PerformanceObserver | null = null;
  private frameSamples: number[] = [];
  private rafId: number | null = null;
  private lastFrameTs = 0;
  private sampling = false;
  private layoutMs: number | null = null;
  private initMs: number | null = null;

  constructor(container: () => HTMLElement | null) {
    this.container = container;
  }

  /** Start collecting long tasks. Safe to call once; no-op in non-browser envs. */
  start(): void {
    if (typeof PerformanceObserver === "undefined" || this.observer) return;
    try {
      this.observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.entryType === "longtask") {
            this.longTasks.push({ start: entry.startTime, duration: entry.duration });
          }
        }
      });
      this.observer.observe({ entryTypes: ["longtask"] });
    } catch {
      this.observer = null;
    }
  }

  stop(): void {
    this.observer?.disconnect();
    this.observer = null;
    this.endFrameSampling();
  }

  mark(name: string): void {
    if (typeof performance === "undefined" || typeof performance.mark !== "function") return;
    performance.mark(name);
  }

  recordLayout(ms: number): void {
    this.layoutMs = ms;
  }

  getLayoutMs(): number | null {
    return this.layoutMs;
  }

  recordInit(ms: number): void {
    this.initMs = ms;
  }

  getInitMs(): number | null {
    return this.initMs;
  }

  /** Current size of the diagram container (world-to-viewport math needs it). */
  getContainerSize(): { width: number; height: number } | null {
    const el = this.container();
    if (!el) return null;
    const rect = el.getBoundingClientRect();
    return { width: rect.width, height: rect.height };
  }

  getLongTasks(): LongTaskRecord[] {
    return [...this.longTasks];
  }

  /** Collect rAF frame deltas until `endFrameSampling()`. */
  beginFrameSampling(): void {
    this.frameSamples = [];
    this.lastFrameTs = performance.now();
    this.sampling = true;
    if (this.rafId !== null) return;

    const tick = (ts: number) => {
      if (!this.sampling) return;
      if (this.lastFrameTs > 0) this.frameSamples.push(ts - this.lastFrameTs);
      this.lastFrameTs = ts;
      this.rafId = requestAnimationFrame(tick);
    };
    this.rafId = requestAnimationFrame(tick);
  }

  endFrameSampling(): void {
    this.sampling = false;
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  getFrameStats(): FrameStats {
    return computeFrameStats(this.frameSamples);
  }

  /** Mounted `.react-flow__node` elements (what actually entered the DOM). */
  countRenderedNodes(): number {
    return this.container()?.querySelectorAll(".react-flow__node").length ?? 0;
  }

  /** Nodes currently rendering at full detail (LOD detail, data-tier=3). */
  countDetailedNodes(): number {
    return this.container()?.querySelectorAll('[data-tier="3"]').length ?? 0;
  }

  /** Mounted `.react-flow__edge` elements (after edge LOD aggregation). */
  countRenderedEdges(): number {
    return this.container()?.querySelectorAll(".react-flow__edge").length ?? 0;
  }
}
