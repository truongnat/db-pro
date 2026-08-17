import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Search, Maximize2, Table2, Loader2, ArrowLeft } from "lucide-react";

import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useResolvedTheme } from "@/commons/stores/theme.store";
import { OverviewExplorer, type OverviewExplorerProps } from "./overview-explorer";
import { CytoscapeErRenderer } from "../renderer/cytoscape-renderer";
import type {
  ErGraphModel,
  ErPosition,
  ErThemeTokens,
  ErViewport,
  TableId,
} from "../renderer/types";
import { computeApproximateOverviewLayout } from "../utils/approximate-layout";
import { findTableMatches, resolveHighlightSet } from "../utils/overview-search";

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
  /**
   * Explicit "open table detail" action (PR#12 re-review P1). Fired ONLY on
   * double-click or the focused-table "Open table" button — a single click
   * focuses the neighborhood and must never navigate away from the overview.
   */
  onOpenTable: (nodeId: TableId) => void;
  explorer: OverviewExplorerProps;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  /** #46 E3 — explicit Back navigation from overview to neighborhood. */
  onBackToNeighborhood?: () => void;
  /** #46 E3 — explicit Back navigation from overview to search. */
  onBackToSearch?: () => void;
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
    searchNodeBorder: get("--state-danger", "#dc2626"),
  };
}

/**
 * P1.9 — large-schema overview renderer (canvas).
 *
 * UX pivot (opass.html): the FULL graph is the default view — no landing, no
 * neighborhood mode, no filter. Layout positions come from the same P1.7
 * worker pipeline (P1-1 approximate paint → Option C refine stages → dagre
 * final). Search FOCUSES (rings matches + centers the viewport, opass-style)
 * and a click FADES the rest while highlighting the hop-scoped neighborhood.
 * Navigation is EXPLICIT only: double-click a node or use the side card's
 * "Open table" action (PR#12 re-review P1 — a single click never navigates).
 * P2-1: colors follow the active theme and swap at runtime.
 */
export function CytoscapeErView({
  model,
  positions,
  degraded,
  layoutReady,
  onViewportChange,
  onOpenTable,
  explorer,
  searchQuery,
  onSearchChange,
  onBackToNeighborhood,
  onBackToSearch,
}: CytoscapeErViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<CytoscapeErRenderer | null>(null);
  const theme = useResolvedTheme();

  // The focused table (search single-match or click). `null` = full graph,
  // no fade. The seed effect applies the hop-scoped highlight + fade.
  const [seed, setSeed] = useState<TableId | null>(null);

  // P1-1 — deterministic approximate positions so the canvas paints in the
  // first frame, even while dagre computes for 8–122 s on a cold open.
  const approximatePositions = useMemo(() => computeApproximateOverviewLayout(model), [model]);

  // Latest callbacks via refs so the renderer is created exactly once per view
  // lifecycle — a changing callback identity must never re-init the canvas
  // (which would orphan the mount effect's renderer reference).
  const onOpenTableRef = useRef(onOpenTable);
  onOpenTableRef.current = onOpenTable;
  const onViewportChangeRef = useRef(onViewportChange);
  onViewportChangeRef.current = onViewportChange;

  // Tap a table → FOCUS it only: fade the rest + highlight its neighborhood.
  // Navigation is deliberately NOT here (PR#12 re-review P1) — a single click
  // must never unmount the overview before the focus renders.
  const onNodeTapRef = useRef<(nodeId: TableId) => void>(() => {});
  onNodeTapRef.current = (nodeId) => {
    setSeed(nodeId);
  };
  // Tap empty canvas → clear focus/fade only. Search rings are search STATE
  // and survive while the query is non-empty (PR#12 re-review P2).
  const onBackgroundTapRef = useRef<() => void>(() => {});
  onBackgroundTapRef.current = () => {
    rendererRef.current?.clearSelection();
    setSeed(null);
  };

  // Create/dispose the canvas renderer once per view lifecycle.
  useEffect(() => {
    if (!containerRef.current) return;
    const renderer = new CytoscapeErRenderer({
      container: containerRef.current,
      theme: resolveThemeTokens(),
      // Single tap = focus. Double tap = the explicit open-table action.
      onNodeClick: (nodeId) => onNodeTapRef.current(nodeId),
      onNodeDoubleClick: (nodeId) => onOpenTableRef.current(nodeId),
      onBackgroundTap: () => onBackgroundTapRef.current(),
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
      return;
    }

    // Already mounted on this model: upgrade to the new positions without
    // re-mounting (no viewport reset, no element churn). Option C — progressive
    // refine stages apply in place without re-fitting (the user may be panning);
    // only the final committed layout re-fits.
    if (positions && positions !== prev.positions) {
      renderer.updatePositions(positions, { fit: layoutReady });
      mountedRef.current = { model, positions };
    }
  }, [model, positions, approximatePositions, layoutReady]);

  // Seed → opass-style focus: fade everything outside the hop-scoped
  // neighborhood of the focused table. `null` clears the fade.
  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;
    if (!seed || !model.tables.some((t) => t.id === seed)) {
      // No seed, or the seed belongs to a previous model (schema switch) —
      // never fade a graph over a phantom table.
      renderer.clearSelection();
      return;
    }
    const hl = resolveHighlightSet(model, seed, explorer.hops);
    renderer.updateSelection({
      nodeIds: [seed],
      highlightNodeIds: [...hl].filter((id) => id !== seed),
      fadeRest: true,
    });
  }, [seed, model, explorer.hops]);

  // Search = focus + highlight (opass), never filter: matches get a red ring
  // and the viewport centers on them; a single match also focuses its
  // neighborhood. Empty query clears the ring and the fade.
  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;
    const q = searchQuery.trim().toLowerCase();
    if (!q) {
      renderer.clearSearchHighlight();
      setSeed(null);
      return;
    }
    const matches = findTableMatches(model, q);
    if (matches.length === 0) {
      renderer.clearSearchHighlight();
      setSeed(null);
      return;
    }
    if (matches.length === 1) {
      renderer.highlightSearch(matches, { focus: true });
      setSeed(matches[0]);
      return;
    }
    // Multiple matches: ring them all, drop any previous focus fade.
    renderer.highlightSearch(matches, { focus: true });
    setSeed(null);
  }, [searchQuery, model]);

  // The focused table (search single-match or click) — drives the side
  // inspector's explicit "Open table" action (PR#12 re-review P1).
  const focusedTable = useMemo(
    () => (seed ? (model.tables.find((t) => t.id === seed) ?? null) : null),
    [seed, model],
  );

  // Option C — surface a subtle "refining" hint while the worker posts
  // progressive stages (layout improving, dagre not committed yet).
  const refining = positions !== null && !layoutReady && !degraded;

  const handleSearchKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Escape") {
        onSearchChange("");
      }
    },
    [onSearchChange],
  );

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div ref={containerRef} className="min-h-0 flex-1" />

      {/* Overlay: search + overview controls (canvas renders beneath). */}
      <div className="pointer-events-none absolute left-2 top-2 z-10 flex flex-col items-start gap-2">
        <div className="pointer-events-auto flex items-center gap-2">
          {onBackToNeighborhood && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="h-7 gap-1 px-2 text-[11px]"
              data-testid="er-back-to-neighborhood"
              onClick={onBackToNeighborhood}
            >
              <ArrowLeft className="h-3 w-3" />
              Neighborhood
            </Button>
          )}
          {onBackToSearch && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-7 gap-1 px-2 text-[11px] text-[var(--text-secondary)]"
              data-testid="er-back-to-search"
              onClick={onBackToSearch}
            >
              <ArrowLeft className="h-3 w-3" />
              Search
            </Button>
          )}
          <div className="relative">
            <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-secondary)]" />
            <Input
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              onKeyDown={handleSearchKeyDown}
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
          <OverviewExplorer {...explorer} />
        </div>
        {focusedTable && (
          <div className="pointer-events-auto flex items-center gap-2 rounded-md border border-[var(--border-default)] bg-popover p-1.5 shadow-sm">
            <span className="max-w-40 truncate text-[11px] font-medium">{focusedTable.label}</span>
            <span className="text-[10px] text-[var(--text-secondary)]">
              {focusedTable.columnCount} columns · {focusedTable.fkCount} FK
            </span>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              className="h-6 px-2 text-[10px]"
              onClick={() => onOpenTableRef.current(focusedTable.id)}
            >
              Open table
            </Button>
          </div>
        )}
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
        <div className="pointer-events-auto text-[10px] text-[var(--text-secondary)]">
          Click to focus · double-click or Open table to view
        </div>
      </div>
    </div>
  );
}
