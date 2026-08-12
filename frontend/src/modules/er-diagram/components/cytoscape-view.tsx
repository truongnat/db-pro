import { useEffect, useMemo, useRef } from "react";
import { Search, Maximize2, Table2, Loader2 } from "lucide-react";

import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useResolvedTheme } from "@/commons/stores/theme.store";
import { NeighborhoodExplorer, type NeighborhoodExplorerProps } from "./neighborhood-explorer";
import { CytoscapeErRenderer } from "../renderer/cytoscape-renderer";
import type {
  ErGraphModel,
  ErPosition,
  ErThemeTokens,
  ErViewport,
  TableId,
} from "../renderer/types";
import { computeApproximateOverviewLayout } from "../utils/approximate-layout";

export interface CytoscapeErViewProps {
  model: ErGraphModel;
  /**
   * Atomic position set from the shared layout engine (null while the worker
   * computes). P1-1: when null, the view paints an approximate layout
   * immediately and upgrades via `updatePositions` once dagre commits.
   */
  positions: Map<TableId, ErPosition> | null;
  /** P1-3 — true when positions came from the approximate fallback (worker unavailable). */
  degraded: boolean;
  /**
   * Option C — true when `positions` is the final committed layout. While
   * false (progressive refine stages), the view applies positions in place
   * WITHOUT re-fitting, so the user's viewport is not yanked every few frames.
   */
  layoutReady: boolean;
  onViewportChange: (viewport: ErViewport) => void;
  onNodeClick: (nodeId: TableId) => void;
  explorer: NeighborhoodExplorerProps;
  searchQuery: string;
  onSearchChange: (query: string) => void;
}

/**
 * P2-1 — resolve canonical design tokens into concrete colors for the canvas.
 * Canvas paints don't resolve CSS `var()`; `data-theme` on <html> selects the
 * active theme (light/dark), so reading computed style gives the right values.
 */
function resolveThemeTokens(): ErThemeTokens {
  const s = typeof document !== "undefined" ? getComputedStyle(document.documentElement) : null;
  const get = (name: string, fallback: string) => {
    const v = s?.getPropertyValue(name).trim();
    return v || fallback;
  };
  return {
    nodeBg: get("--surface-panel", "#1e293b"),
    nodeBorder: get("--border-default", "#475569"),
    nodeLabel: get("--text-primary", "#cbd5e1"),
    selectedNodeBorder: get("--accent", "#7dd3fc"),
    selectedNodeBg: get("--surface-active", "#1e3a5f"),
    neighborNodeBorder: get("--accent-hover", "#38bdf8"),
    edgeColor: get("--border-subtle", "#334155"),
    edgeArrowColor: get("--border-default", "#475569"),
    neighborEdgeColor: get("--border-strong", "#475569"),
  };
}

/**
 * P1.9 — large-schema overview renderer (canvas).
 *
 * The full graph is drawn as canvas primitives by `CytoscapeErRenderer`
 * (≈ dozens of DOM elements regardless of graph size). Layout positions come
 * from the same P1.7 worker pipeline the React Flow path uses. P1-1: the
 * overview mounts as soon as the view exists — with a fast approximate layout
 * when dagre is still computing — and upgrades in place when the worker
 * commits. P2-1: colors follow the active theme (resolved tokens), and swap
 * at runtime on theme change without destroying the graph.
 */
export function CytoscapeErView({
  model,
  positions,
  degraded,
  layoutReady,
  onViewportChange,
  onNodeClick,
  explorer,
  searchQuery,
  onSearchChange,
}: CytoscapeErViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<CytoscapeErRenderer | null>(null);
  const selectedRef = useRef<TableId | null>(null);
  const theme = useResolvedTheme();

  // P1-1 — deterministic approximate positions so the canvas paints in the
  // first frame, even while dagre computes for 8–122 s on a cold open.
  const approximatePositions = useMemo(() => computeApproximateOverviewLayout(model), [model]);

  // Latest callbacks via refs so the renderer is created exactly once per view
  // lifecycle — a changing callback identity must never re-init the canvas
  // (which would orphan the mount effect's renderer reference).
  const onNodeClickRef = useRef(onNodeClick);
  onNodeClickRef.current = onNodeClick;
  const onViewportChangeRef = useRef(onViewportChange);
  onViewportChangeRef.current = onViewportChange;

  // Create/dispose the canvas renderer once per view lifecycle.
  useEffect(() => {
    if (!containerRef.current) return;
    const renderer = new CytoscapeErRenderer({
      container: containerRef.current,
      theme: resolveThemeTokens(),
      onNodeClick: (nodeId) => {
        selectedRef.current = nodeId;
        // Keep the canvas neighborhood highlight in sync on every tap, not
        // just on the initial mount.
        rendererRef.current?.updateSelection({ nodeIds: [nodeId] });
        onNodeClickRef.current(nodeId);
      },
      onViewportChange: (viewport) => onViewportChangeRef.current(viewport),
    });
    rendererRef.current = renderer;
    return () => {
      renderer.dispose();
      rendererRef.current = null;
    };
  }, []);

  // P2-1 — re-resolve tokens on every theme change and swap them in place.
  useEffect(() => {
    rendererRef.current?.updateTheme(resolveThemeTokens());
  }, [theme]);

  // P1-1 — mount as soon as any positions are available (approximate or dagre),
  // then upgrade in place when the real layout commits. Never an empty canvas:
  // cold opens paint the approximate circle immediately.
  const mountedRef = useRef<{ model: ErGraphModel; positions: Map<TableId, ErPosition> } | null>(
    null,
  );
  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    const prev = mountedRef.current;
    if (prev === null || prev.model !== model) {
      // Model switch: any `positions` prop still in flight belongs to the
      // PREVIOUS graph (the hook preserves positions across a computing
      // transition) — never mount a new graph at stale coordinates. Start from
      // the model-derived approximate circle; committed dagre / progressive
      // stages arrive via the upgrade path below.
      const mountPositions =
        prev !== null ? approximatePositions : (positions ?? approximatePositions);
      renderer.mount(model, mountPositions);
      mountedRef.current = { model, positions: mountPositions };
      if (selectedRef.current) {
        renderer.updateSelection({ nodeIds: [selectedRef.current] });
      }
      return;
    }

    // Already mounted on this model: upgrade to the new positions without
    // re-mounting (no viewport reset, no element churn). Option C — progressive
    // refine stages apply in place without re-fitting (the user may be panning);
    // only the final committed layout re-fits.
    if (positions && positions !== prev.positions) {
      renderer.updatePositions(positions, { fit: layoutReady });
      mountedRef.current = { model, positions };
      if (selectedRef.current) {
        renderer.updateSelection({ nodeIds: [selectedRef.current] });
      }
    }
  }, [model, positions, approximatePositions, layoutReady]);

  // Option C — surface a subtle "refining" hint while the worker posts
  // progressive stages (layout improving, dagre not committed yet).
  const refining = positions !== null && !layoutReady && !degraded;

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div ref={containerRef} className="min-h-0 flex-1" />

      {/* Overlay: search + exploration controls (canvas renders beneath). */}
      <div className="pointer-events-none absolute left-2 top-2 z-10 flex flex-col items-start gap-2">
        <div className="pointer-events-auto flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-secondary)]" />
            <Input
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Search tables..."
              className="h-7 w-48 rounded-md pl-7 text-[12px]"
            />
          </div>
          <Badge variant="outline" className="h-7 bg-popover text-[11px]">
            <Table2 className="mr-1 h-3 w-3" />
            {explorer.totalTables} tables
          </Badge>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-7 w-7 p-0"
            onClick={() => rendererRef.current?.fit()}
          >
            <Maximize2 className="h-3.5 w-3.5" />
          </Button>
        </div>
        <div className="pointer-events-auto">
          <NeighborhoodExplorer {...explorer} />
        </div>
        {refining && (
          <div className="pointer-events-auto flex items-center gap-1.5 rounded-md border border-sky-500/40 bg-sky-500/10 px-2 py-1 text-[11px] text-sky-600">
            <Loader2 className="h-3 w-3 animate-spin" />
            Refining layout…
          </div>
        )}
        {degraded && (
          <div className="pointer-events-auto rounded-md border border-amber-500/40 bg-amber-500/10 px-2 py-1 text-[11px] text-amber-600">
            Approximate layout — layout worker unavailable
          </div>
        )}
      </div>
    </div>
  );
}
