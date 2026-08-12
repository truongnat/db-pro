import { memo } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { cn } from "@/lib/utils";

import { lodTier } from "../../utils/lod";
import type { TableNodeData } from "./types";

/**
 * LOD 0 — a single dot. Rendered when the whole schema fits the viewport
 * (zoom < 0.2). Purpose: keep pan/zoom smooth over hundreds of tables while
 * still conveying "something exists here".
 *
 * Keeps one generic source + one generic target handle so edges stay anchored
 * to the node boundary (React Flow drops edges whose handle ids are missing,
 * so the diagram wires edges to generic handles at this level — see P1.4).
 */
export const ErDotNode = memo(function ErDotNode({ data, selected }: NodeProps) {
  const { label } = data as TableNodeData;

  return (
    <div
      data-tier={lodTier("dot")}
      title={label}
      className={cn(
        "h-3.5 w-3.5 rounded-full border bg-primary/40 transition-colors",
        selected ? "border-primary ring-2 ring-primary/30" : "border-[var(--app-border)]",
      )}
    >
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
