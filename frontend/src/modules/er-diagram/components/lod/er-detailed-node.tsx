import { memo, useCallback } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Key, Link2 } from "lucide-react";
import { cn } from "@/lib/utils";

import { lodTier } from "../../utils/lod";
import type { TableNodeData } from "./types";

/**
 * LOD 3 — full column list with per-column PK/FK handles and navigation.
 * Zoom > 0.7, or any selected/neighborhood node at detail zoom.
 */
export const ErDetailedNode = memo(function ErDetailedNode({ data, selected }: NodeProps) {
  const { label, columns } = data as TableNodeData;

  const handleColumnClick = useCallback(
    (e: React.MouseEvent, colName: string) => {
      e.stopPropagation();
      const event = new CustomEvent("er-column-click", {
        detail: { tableName: label, columnName: colName },
        bubbles: true,
      });
      (e.currentTarget as HTMLElement).dispatchEvent(event);
    },
    [label],
  );

  return (
    <div
      data-tier={lodTier("detail")}
      className={cn(
        "min-w-[180px] max-w-[260px] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-sm transition-shadow",
        selected ? "border-primary ring-1 ring-primary/30" : "border-[var(--border-default)]",
      )}
    >
      {/* Header */}
      <div className="flex items-center gap-1.5 border-b bg-muted/50 px-2.5 py-1.5">
        <span className="h-2 w-2 shrink-0 rounded-sm bg-primary/60" />
        <span className="truncate text-[12px] font-semibold">{label}</span>
      </div>

      {/* Full column list */}
      <div className="flex flex-col">
        {columns.map((col) => (
          <div
            key={col.name}
            className="group flex cursor-pointer items-center gap-1 px-2 py-[3px] text-[11px] hover:bg-[var(--surface-hover)]"
            data-column={col.name}
            onClick={(e) => handleColumnClick(e, col.name)}
            title={`Open ${label}.${col.name} in Columns`}
          >
            <span className="flex w-4 shrink-0 items-center justify-center">
              {col.isPrimaryKey && <Key className="h-2.5 w-2.5 text-primary" />}
              {col.isForeignKey && !col.isPrimaryKey && <Link2 className="h-2.5 w-2.5 text-info" />}
            </span>

            <span
              className={cn(
                "flex-1 truncate font-mono",
                col.isPrimaryKey ? "font-medium text-foreground" : "text-foreground",
              )}
            >
              {col.name}
            </span>

            <span className="shrink-0 font-mono text-[10px] text-[var(--text-secondary)]">
              {col.dataType}
            </span>

            {col.nullable && <span className="text-[9px] text-[var(--text-tertiary)]">?</span>}

            {col.isPrimaryKey && (
              <Handle
                type="source"
                position={Position.Right}
                id={`pk:${col.name}`}
                className="!h-1.5 !w-1.5 !border-0 !bg-primary/40 !opacity-0 group-hover:!opacity-100"
              />
            )}

            {col.isForeignKey && (
              <Handle
                type="target"
                position={Position.Left}
                id={`fk:${col.name}`}
                className="!h-1.5 !w-1.5 !border-0 !bg-info/40 !opacity-0 group-hover:!opacity-100"
              />
            )}
          </div>
        ))}
      </div>
    </div>
  );
});
