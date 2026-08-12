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
 * Pure helpers (`computeFrameStats`, `computeVisibleNodeIds`) are unit-tested;
 * `ErPerfMonitor` is `performance`/DOM-backed and used by the dev HUD overlay.
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

const DEFAULT_NODE_WIDTH = 220;
const DEFAULT_NODE_HEIGHT = 120;

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

export interface ErViewport {
  x: number;
  y: number;
  zoom: number;
  width: number;
  height: number;
}

export interface ErViewportNode {
  id: string;
  position: { x: number; y: number };
  measured?: { width?: number; height?: number };
}

/**
 * IDs of nodes whose bounding box intersects the current viewport.
 *
 * Pure function — unit-tested. This is the spatial-query precursor that P1.5
 * will turn into a proper Spatial Index; for now it backs the `viewportTables`
 * instrumentation metric. Note: unmeasured nodes fall back to a fixed default
 * size, so the count is approximate for tall tier-2 nodes — acceptable for a
 * dev HUD, not for the P1.5 spatial index.
 */
export function computeVisibleNodeIds(nodes: ErViewportNode[], viewport: ErViewport): Set<string> {
  if (viewport.width <= 0 || viewport.height <= 0 || viewport.zoom <= 0) return new Set();

  const viewLeft = -viewport.x / viewport.zoom;
  const viewTop = -viewport.y / viewport.zoom;
  const viewRight = viewLeft + viewport.width / viewport.zoom;
  const viewBottom = viewTop + viewport.height / viewport.zoom;

  const visible = new Set<string>();
  for (const node of nodes) {
    const w = node.measured?.width ?? DEFAULT_NODE_WIDTH;
    const h = node.measured?.height ?? DEFAULT_NODE_HEIGHT;
    const left = node.position.x;
    const top = node.position.y;
    const right = left + w;
    const bottom = top + h;
    if (right >= viewLeft && left <= viewRight && bottom >= viewTop && top <= viewBottom) {
      visible.add(node.id);
    }
  }
  return visible;
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

  /** Nodes currently rendering at full detail (tier 2). */
  countDetailedNodes(): number {
    return this.container()?.querySelectorAll('[data-tier="2"]').length ?? 0;
  }
}
