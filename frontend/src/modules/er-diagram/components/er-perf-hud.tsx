import { useEffect, useState } from "react";
import { Panel, type Node, type Viewport } from "@xyflow/react";

import { computeVisibleNodeIds, type ErPerfMonitor } from "../utils/instrumentation";

interface ErPerfHudProps {
  monitor: ErPerfMonitor;
  /** Nodes currently held by React Flow state (positions + measured sizes). */
  nodes: Node[];
  edgeCount: number;
  /** Live viewport ref — read on the HUD's own refresh tick, never via state. */
  viewportRef: React.MutableRefObject<Viewport | null>;
}

/**
 * P1.1 — live runtime-instrumentation overlay.
 *
 * Displays the locked P1 acceptance metrics in the top-right corner:
 * time-to-interactive, layout duration, max long task, pan/zoom frame stats,
 * and the graph / viewport / rendered / detailed node + edge counts.
 *
 * Enabled via `localStorage.setItem("er-perf-hud", "1")` — dev tool, not shipped UI.
 */
export function ErPerfHud({ monitor, nodes, edgeCount, viewportRef }: ErPerfHudProps) {
  // Re-render periodically so DOM-derived counts stay live while panning/zooming.
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => setTick((t) => t + 1), 500);
    return () => window.clearInterval(id);
  }, []);

  const containerSize = monitor.getContainerSize();
  const viewport = viewportRef.current;
  const erViewport = viewport && containerSize ? { ...viewport, ...containerSize } : null;
  const viewportCount = erViewport ? computeVisibleNodeIds(nodes, erViewport).size : 0;

  const initMs = monitor.getInitMs();
  const layoutMs = monitor.getLayoutMs();
  const longTasks = monitor.getLongTasks();
  const maxLongTaskMs = longTasks.reduce((max, t) => Math.max(max, t.duration), 0);
  const frameStats = monitor.getFrameStats();
  const renderedNodes = monitor.countRenderedNodes();
  const detailedNodes = monitor.countDetailedNodes();
  const renderedEdges = monitor.countRenderedEdges();

  const row = (label: string, value: string) => (
    <div className="flex items-center justify-between gap-3">
      <span className="text-[var(--app-text-muted)]">{label}</span>
      <span className="tabular-nums text-[var(--app-text-primary)]">{value}</span>
    </div>
  );

  return (
    <Panel position="top-right" className="m-2">
      <div className="pointer-events-none w-56 select-none rounded-md border border-[var(--app-border)] bg-popover/95 p-2 font-mono text-[10px] leading-4 shadow-sm backdrop-blur">
        <div className="mb-1 border-b border-[var(--app-border)] pb-1 font-sans text-[10px] font-semibold uppercase tracking-wide text-[var(--app-text-muted)]">
          ER perf · P1.1
        </div>
        {row("init", initMs === null ? "—" : `${initMs.toFixed(1)}ms`)}
        {row("layout", layoutMs === null ? "—" : `${layoutMs.toFixed(1)}ms`)}
        {row("long task max", maxLongTaskMs === 0 ? "—" : `${maxLongTaskMs.toFixed(1)}ms`)}
        {row(
          "frame avg / p95",
          frameStats.samples === 0
            ? "—"
            : `${frameStats.avgMs.toFixed(1)} / ${frameStats.p95Ms.toFixed(1)}ms`,
        )}
        {row("graph tables", String(nodes.length))}
        {row("viewport tables", String(viewportCount))}
        {row("rendered tables", String(renderedNodes))}
        {row("detailed tables", String(detailedNodes))}
        {row("graph edges", String(edgeCount))}
        {row("rendered edges", String(renderedEdges))}
      </div>
    </Panel>
  );
}
