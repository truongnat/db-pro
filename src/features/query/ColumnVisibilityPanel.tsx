import { useEffect, useMemo, useRef, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type ColumnVisibilityPanelProps = {
  columns: string[];
  hiddenColumnSet: Set<string>;
  canHideAnyColumn: boolean;
  hiddenColumnCount: number;
  onToggleColumnVisibility: (columnName: string) => void;
  onShowAllColumns: () => void;
  onResetColumnLayout: () => void;
};

export function ColumnVisibilityPanel({
  columns,
  hiddenColumnSet,
  canHideAnyColumn,
  hiddenColumnCount,
  onToggleColumnVisibility,
  onShowAllColumns,
  onResetColumnLayout,
}: ColumnVisibilityPanelProps) {
  const [open, setOpen] = useState(false);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const visibleColumnCount = columns.length - hiddenColumnCount;

  useEffect(() => {
    if (!open) {
      return;
    }

    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (!target) {
        return;
      }

      if (panelRef.current && !panelRef.current.contains(target)) {
        setOpen(false);
      }
    };

    window.addEventListener("pointerdown", onPointerDown, true);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
    };
  }, [open]);

  const columnRows = useMemo(
    () =>
      columns.map((columnName) => {
        const visible = !hiddenColumnSet.has(columnName);
        const disableToggle = visible && !canHideAnyColumn;
        return { columnName, visible, disableToggle };
      }),
    [canHideAnyColumn, columns, hiddenColumnSet],
  );

  return (
    <div ref={panelRef} className="relative">
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="h-8"
        onClick={() => setOpen((previous) => !previous)}
      >
        Columns
        <Badge variant="secondary" className="ml-2">
          {visibleColumnCount}/{columns.length}
        </Badge>
      </Button>

      {open && (
        <div className="absolute right-0 top-9 z-30 w-72 rounded-md border bg-popover p-2 text-popover-foreground shadow-xl">
          <div className="mb-2 flex items-center justify-between px-1 text-xs text-muted-foreground">
            <span>Show / hide columns</span>
            <span>{hiddenColumnCount > 0 ? `${hiddenColumnCount} hidden` : "All visible"}</span>
          </div>

          <div className="max-h-56 overflow-auto rounded border bg-background">
            {columnRows.map(({ columnName, visible, disableToggle }) => (
              <label
                key={columnName}
                className="flex cursor-pointer items-center gap-2 border-b px-2 py-1.5 text-xs last:border-b-0 hover:bg-muted/40"
              >
                <input
                  type="checkbox"
                  checked={visible}
                  disabled={disableToggle}
                  onChange={() => onToggleColumnVisibility(columnName)}
                />
                <span className="truncate">{columnName}</span>
              </label>
            ))}
          </div>

          <div className="mt-2 flex items-center gap-2">
            <Button
              type="button"
              size="sm"
              variant="secondary"
              className="h-7 text-[11px]"
              onClick={() => {
                onShowAllColumns();
              }}
            >
              Show all
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-7 text-[11px]"
              onClick={() => {
                onResetColumnLayout();
              }}
            >
              Reset widths
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="ml-auto h-7 text-[11px]"
              onClick={() => setOpen(false)}
            >
              Close
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
