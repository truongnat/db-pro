import { Boxes, Layers, Table2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import type { NeighborhoodScope } from "../utils/neighborhood";

export interface OverviewExplorerProps {
  totalTables: number;
  relationCount: number;
  columnCount: number;
  /** Highlight radius for search/click focus (opass-style neighborhood ring). */
  hops: NeighborhoodScope;
  onSelectHops: (hops: NeighborhoodScope) => void;
}

const HOP_SCOPES: { value: NeighborhoodScope; key: string }[] = [
  { value: 1, key: "oneHop" },
  { value: 2, key: "twoHops" },
  { value: 3, key: "threeHops" },
  { value: "domain", key: "domain" },
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
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-2 rounded-md border border-[var(--border-default)] bg-popover p-1.5 shadow-sm">
      <div className="flex items-center gap-1">
        <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[11px]">
          <Table2 className="h-2.5 w-2.5" />
          <span aria-label={t("schemaWorkspace.tableCount", { count: totalTables })}>
            {totalTables}
          </span>
        </Badge>
        <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[11px]">
          <Layers className="h-2.5 w-2.5" />
          <span aria-label={t("schemaWorkspace.relationCount", { count: relationCount })}>
            {relationCount}
          </span>
        </Badge>
        <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[11px]">
          <Boxes className="h-2.5 w-2.5" />
          <span aria-label={t("schemaWorkspace.columnCount", { count: columnCount })}>
            {columnCount}
          </span>
        </Badge>
      </div>
      <span className="px-0.5 text-[11px] font-medium text-[var(--text-secondary)]">
        {t("schemaWorkspace.highlight")}
      </span>
      <div className="flex items-center gap-0.5">
        {HOP_SCOPES.map((s) => (
          <Button
            key={s.value}
            type="button"
            variant={hops === s.value ? "secondary" : "ghost"}
            size="sm"
            className="h-6 px-1.5 text-[11px]"
            onClick={() => onSelectHops(s.value)}
          >
            {t(`schemaWorkspace.hops.${s.key}`)}
          </Button>
        ))}
      </div>
    </div>
  );
}
