import { useEffect, useRef } from "react";
import { Search, Maximize2, Table2 } from "lucide-react";

import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { NeighborhoodExplorer, type NeighborhoodExplorerProps } from "./neighborhood-explorer";
import { CytoscapeErRenderer } from "../renderer/cytoscape-renderer";
import type { ErGraphModel, ErPosition, ErViewport, TableId } from "../renderer/types";
import type { LayoutStatus } from "../hooks/use-worker-layout";

export interface CytoscapeErViewProps {
  model: ErGraphModel;
  /** Atomic position set from the shared layout engine (null while computing). */
  positions: Map<TableId, ErPosition> | null;
  layoutStatus: LayoutStatus;
  onViewportChange: (viewport: ErViewport) => void;
  onNodeClick: (nodeId: TableId) => void;
  explorer: NeighborhoodExplorerProps;
  searchQuery: string;
  onSearchChange: (query: string) => void;
}

/**
 * P1.9 — large-schema overview renderer (canvas).
 *
 * The full graph is drawn as canvas primitives by `CytoscapeErRenderer`
 * (≈ dozens of DOM elements regardless of graph size). Layout positions come
 * from the same P1.7 worker pipeline the React Flow path uses. Only mounted
 * when `isLargeSchema && showAll` — the explicit "All N tables" overview.
 */
export function CytoscapeErView({
  model,
  positions,
  layoutStatus,
  onViewportChange,
  onNodeClick,
  explorer,
  searchQuery,
  onSearchChange,
}: CytoscapeErViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<CytoscapeErRenderer | null>(null);
  const selectedRef = useRef<TableId | null>(null);

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

  // Mount (or re-mount) the graph only when the layout engine committed a
  // stable position set — never partial positions.
  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;
    if (layoutStatus !== "ready" || !positions) return;
    renderer.mount(model, positions);
    if (selectedRef.current) {
      renderer.updateSelection({ nodeIds: [selectedRef.current] });
    }
  }, [model, positions, layoutStatus]);

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
      </div>
    </div>
  );
}
