import { Boxes, Layers, Table2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import type { NeighborhoodScope } from "../utils/neighborhood";

export interface OverviewExplorerProps {
  totalTables: number;
  relationCount: number;
  columnCount: number;
  /** Highlight radius for search/click focus (opass-style neighborhood ring). */
  hops: NeighborhoodScope;
  onSelectHops: (hops: NeighborhoodScope) => void;
}

const HOP_SCOPES: { value: NeighborhoodScope; label: string }[] = [
  { value: 1, label: "1 hop" },
  { value: 2, label: "2 hops" },
  { value: 3, label: "3 hops" },
  { value: "domain", label: "Domain" },
];

/**
 * Large-schema overview explorer (UX pivot — full graph is now the default).
 *
 * The canvas ALWAYS shows the entire schema; there is no landing mode, no
 * suggested-points gate and no "All N tables" toggle. This compact pill shows
 * the schema stats and the highlight hop radius applied when the user focuses
 * a table via search or a click (opass.html behavior: type to focus, click to
 * highlight the neighborhood).
 */
export function OverviewExplorer({
  totalTables,
  relationCount,
  columnCount,
  hops,
  onSelectHops,
}: OverviewExplorerProps) {
  return (
    <div className="flex items-center gap-2 rounded-md border border-[var(--border-default)] bg-popover p-1.5 shadow-sm">
      <div className="flex items-center gap-1">
        <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px]">
          <Table2 className="h-2.5 w-2.5" />
          {totalTables}
        </Badge>
        <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px]">
          <Layers className="h-2.5 w-2.5" />
          {relationCount}
        </Badge>
        <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px]">
          <Boxes className="h-2.5 w-2.5" />
          {columnCount}
        </Badge>
      </div>
      <span className="px-0.5 text-[10px] font-medium text-[var(--text-secondary)]">
        Highlight:
      </span>
      <div className="flex items-center gap-0.5">
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
      </div>
    </div>
  );
}
