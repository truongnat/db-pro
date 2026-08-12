import { Boxes, Compass, Layers, Table2, X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import type { NeighborhoodScope } from "../utils/neighborhood";

export interface SuggestedPoint {
  key: string;
  label: string;
}

export interface NeighborhoodExplorerProps {
  totalTables: number;
  relationCount: number;
  columnCount: number;
  suggestedPoints: SuggestedPoint[];
  /** Currently focused table label, or null when in landing mode. */
  seedLabel: string | null;
  hops: NeighborhoodScope;
  showAll: boolean;
  onSelectPoint: (key: string) => void;
  onSelectHops: (hops: NeighborhoodScope) => void;
  onShowAll: () => void;
  onReset: () => void;
}

const HOP_SCOPES: { value: NeighborhoodScope; label: string }[] = [
  { value: 1, label: "1 hop" },
  { value: 2, label: "2 hops" },
  { value: 3, label: "3 hops" },
  { value: "domain", label: "Domain" },
];

/**
 * P1.6 — default neighborhood exploration UX for large schemas.
 *
 * Full schema is NOT the default experience. When a schema exceeds the large
 * threshold, the diagram opens in exploration mode: schema stats, suggested
 * starting points (hub tables by FK degree), and hop scopes [1][2][3][Domain]
 * plus an explicit "All N tables" escape hatch.
 */
export function NeighborhoodExplorer({
  totalTables,
  relationCount,
  columnCount,
  suggestedPoints,
  seedLabel,
  hops,
  showAll,
  onSelectPoint,
  onSelectHops,
  onShowAll,
  onReset,
}: NeighborhoodExplorerProps) {
  if (showAll) {
    return (
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="h-7 text-[11px]"
        onClick={onReset}
      >
        <Compass className="mr-1 h-3 w-3" />
        Neighborhood mode
      </Button>
    );
  }

  // Landing — no seed yet: stats + suggested starting points.
  if (!seedLabel) {
    return (
      <div className="flex w-[260px] flex-col gap-2 rounded-md border border-[var(--border-default)] bg-popover p-2.5 shadow-sm">
        <div className="flex items-center gap-2 text-[10px] text-[var(--text-secondary)]">
          <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px]">
            <Table2 className="h-2.5 w-2.5" />
            {totalTables}
          </Badge>
          <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px]">
            <Layers className="h-2.5 w-2.5" />
            {relationCount} relations
          </Badge>
          <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px]">
            <Boxes className="h-2.5 w-2.5" />
            {columnCount} cols
          </Badge>
        </div>

        <div className="text-[10px] font-semibold uppercase tracking-wide text-[var(--text-secondary)]">
          Suggested starting points
        </div>
        <div className="flex flex-wrap gap-1">
          {suggestedPoints.map((p) => (
            <Button
              key={p.key}
              type="button"
              variant="outline"
              size="sm"
              className="h-6 px-2 text-[11px]"
              onClick={() => onSelectPoint(p.key)}
            >
              {p.label}
            </Button>
          ))}
        </div>

        <div className="flex items-center gap-1">
          <span className="text-[10px] text-[var(--text-secondary)]">Explore:</span>
          {HOP_SCOPES.map((s) => (
            <Button
              key={s.value}
              type="button"
              variant="ghost"
              size="sm"
              className="h-6 px-1.5 text-[10px]"
              onClick={() => onSelectHops(s.value)}
            >
              {s.label}
            </Button>
          ))}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-6 px-1.5 text-[10px]"
            onClick={onShowAll}
          >
            All {totalTables}
          </Button>
        </div>
      </div>
    );
  }

  // Seed active — hop scope selector for the focused neighborhood.
  return (
    <div className="flex items-center gap-1 rounded-md border border-[var(--border-default)] bg-popover p-1 shadow-sm">
      <span className="px-1.5 text-[10px] font-medium text-[var(--text-secondary)]">
        {seedLabel}
      </span>
      {HOP_SCOPES.map((s) => (
        <Button
          key={s.value}
          type="button"
          variant={hops === s.value ? "secondary" : "ghost"}
          size="sm"
          className="h-6 px-1.5 text-[10px]"
          onClick={() => onSelectHops(s.value)}
        >
          {s.label}
        </Button>
      ))}
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-6 px-1.5 text-[10px]"
        onClick={onShowAll}
      >
        All {totalTables}
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-6 w-6 p-0 text-[10px]"
        onClick={onReset}
        aria-label="Reset exploration"
      >
        <X className="h-3 w-3" />
      </Button>
    </div>
  );
}
