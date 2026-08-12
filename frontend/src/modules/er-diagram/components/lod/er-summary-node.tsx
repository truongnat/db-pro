import { memo } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Key, Link2 } from "lucide-react";
import { cn } from "@/lib/utils";

import { lodTier } from "../../utils/lod";
import type { TableNodeData } from "./types";

/**
 * LOD 2 — table name + "N cols · M FK" summary. Zoom 0.45–0.7. More
 * informative than compact while keeping the DOM tiny.
 */
export const ErSummaryNode = memo(function ErSummaryNode({ data, selected }: NodeProps) {
  const { label, columns } = data as TableNodeData;

  const fkCount = columns.filter((c) => c.isForeignKey).length;

  return (
    <div
      data-tier={lodTier("summary")}
      className={cn(
        "min-w-[150px] max-w-[220px] overflow-hidden rounded-md border bg-popover shadow-sm transition-shadow",
        selected ? "border-primary ring-1 ring-primary/30" : "border-[var(--app-border)]",
      )}
    >
      <div className="flex items-center gap-1.5 border-b bg-muted/50 px-2.5 py-1.5">
        <span className="h-2 w-2 shrink-0 rounded-sm bg-primary/60" />
        <span className="truncate text-[12px] font-semibold">{label}</span>
      </div>
      <div className="flex items-center gap-3 px-2.5 py-1 text-[10px] text-[var(--app-text-muted)]">
        <span className="tabular-nums">
          {columns.length} {columns.length === 1 ? "col" : "cols"}
        </span>
        {fkCount > 0 && (
          <span className="flex items-center gap-1 tabular-nums">
            <Link2 className="h-2.5 w-2.5 text-info" />
            {fkCount} FK
          </span>
        )}
        {columns.some((c) => c.isPrimaryKey) && (
          <span className="flex items-center gap-1">
            <Key className="h-2.5 w-2.5 text-primary" />
          </span>
        )}
      </div>

      {/* Generic anchors so edges stay visible at this LOD. */}
      <Handle
        type="source"
        position={Position.Right}
        id="source"
        className="!h-1 !w-1 !border-0 !bg-transparent !opacity-0"
      />
      <Handle
        type="target"
        position={Position.Left}
        id="target"
        className="!h-1 !w-1 !border-0 !bg-transparent !opacity-0"
      />
    </div>
  );
});
