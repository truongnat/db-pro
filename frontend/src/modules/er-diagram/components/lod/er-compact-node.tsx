import { memo } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { cn } from "@/lib/utils";

import { lodTier } from "../../utils/lod";
import type { TableNodeData } from "./types";

/**
 * LOD 1 — table name only. Zoom 0.2–0.45. No column rows exist in the DOM;
 * this is a render-tree switch, not CSS hiding.
 */
export const ErCompactNode = memo(function ErCompactNode({ data, selected }: NodeProps) {
  const { label } = data as TableNodeData;

  return (
    <div
      data-tier={lodTier("compact")}
      className={cn(
        "flex min-w-[120px] max-w-[200px] items-center gap-1.5 rounded-md border bg-popover px-2.5 py-1.5 shadow-sm transition-shadow",
        selected ? "border-primary ring-1 ring-primary/30" : "border-[var(--border-default)]",
      )}
    >
      <span className="h-2 w-2 shrink-0 rounded-sm bg-primary/60" />
      <span className="truncate text-[12px] font-semibold">{label}</span>

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
