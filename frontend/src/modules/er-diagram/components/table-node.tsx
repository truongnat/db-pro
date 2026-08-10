import { memo, useCallback } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Key, Link2 } from "lucide-react";
import { cn } from "@/lib/utils";

export interface TableNodeData {
  label: string;
  schema: string;
  columns: {
    name: string;
    dataType: string;
    nullable: boolean;
    isPrimaryKey: boolean;
    isForeignKey: boolean;
  }[];
  compact: boolean;
  [key: string]: unknown;
}

/**
 * Custom React Flow node that renders a database table
 * with PK/FK indicators and column types.
 */
export const TableNode = memo(function TableNode({ data, selected }: NodeProps) {
  const { label, columns, compact } = data as TableNodeData;

  const handleColumnClick = useCallback(
    (e: React.MouseEvent, colName: string) => {
      e.stopPropagation();
      // Dispatch a custom event so the parent ErDiagram can handle navigation
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
      className={cn(
        "min-w-[180px] max-w-[260px] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-sm transition-shadow",
        selected ? "border-primary ring-1 ring-primary/30" : "border-[var(--app-border)]",
      )}
    >
      {/* Header */}
      <div className="flex items-center gap-1.5 border-b bg-muted/50 px-2.5 py-1.5">
        <span className="h-2 w-2 shrink-0 rounded-sm bg-primary/60" />
        <span className="truncate text-[12px] font-semibold">{label}</span>
      </div>

      {/* Columns */}
      {!compact && (
        <div className="flex flex-col">
          {columns.map((col) => (
            <div
              key={col.name}
              className="group flex cursor-pointer items-center gap-1 px-2 py-[3px] text-[11px] hover:bg-[var(--app-hover)]"
              data-column={col.name}
              onClick={(e) => handleColumnClick(e, col.name)}
              title={`Open ${label}.${col.name} in Columns`}
            >
              {/* PK / FK indicators */}
              <span className="flex w-4 shrink-0 items-center justify-center">
                {col.isPrimaryKey && <Key className="h-2.5 w-2.5 text-primary" />}
                {col.isForeignKey && !col.isPrimaryKey && (
                  <Link2 className="h-2.5 w-2.5 text-info" />
                )}
              </span>

              {/* Column name */}
              <span
                className={cn(
                  "flex-1 truncate font-mono",
                  col.isPrimaryKey ? "font-medium text-foreground" : "text-foreground",
                )}
              >
                {col.name}
              </span>

              {/* Data type */}
              <span className="shrink-0 font-mono text-[10px] text-[var(--app-text-muted)]">
                {col.dataType}
              </span>

              {/* Nullable marker */}
              {col.nullable && (
                <span className="text-[9px] text-[var(--app-text-dim)]">?</span>
              )}

              {/* Handle for PK columns (source for edges) */}
              {col.isPrimaryKey && (
                <Handle
                  type="source"
                  position={Position.Right}
                  id={`pk:${col.name}`}
                  className="!h-1.5 !w-1.5 !border-0 !bg-primary/40 !opacity-0 group-hover:!opacity-100"
                />
              )}

              {/* Handle for FK columns (target for edges) */}
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
      )}

      {/* Compact mode: just show count */}
      {compact && (
        <div className="px-2.5 py-1 text-[10px] text-[var(--app-text-muted)]">
          {columns.length} columns
        </div>
      )}
    </div>
  );
});
