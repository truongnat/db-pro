import { useState, useMemo } from "react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import { UnifiedGrid } from "@/modules/unified-grid/components/unified-grid";
import type { GridSort } from "@/modules/unified-grid/types";

import type { ColumnMeta, Row } from "../types/query.types";
import type { SortState } from "@/commons/types/workspace.types";
import { ColumnMetadataPopover } from "./column-metadata-popover";
import { ZoomControls } from "./zoom-controls";

interface ResultGridProps {
  columns: ColumnMeta[];
  rows: Row[];
  sort: SortState;
  onSort: (column: string) => void;
  durationMs: number;
  rowCount: number;
}

export function ResultGrid({
  columns,
  rows,
  sort,
  onSort,
  durationMs,
  rowCount,
}: ResultGridProps) {
  const { t } = useTranslation();
  const [zoom, setZoom] = useState(100);
  const [metadataColumn, setMetadataColumn] = useState<{ column: ColumnMeta; el: HTMLElement } | null>(null);

  /* Adapt single SortState → GridSort[] for UnifiedGrid */
  const sorts = useMemo<GridSort[]>(
    () =>
      sort.column && sort.direction
        ? [{ column: sort.column, direction: sort.direction }]
        : [],
    [sort],
  );

  return (
    <>
      <UnifiedGrid
        columns={columns}
        rows={rows}
        sorts={sorts}
        onSort={onSort}
        contentStyle={zoom !== 100 ? { zoom: zoom / 100 } : undefined}
        emptyState={
          <p className="text-sm text-[var(--app-text-muted)]">
            {t("query.noResults")}
          </p>
        }
        renderHeaderExtra={(col) => (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="shrink-0 rounded px-1 text-[11px] text-[var(--app-text-muted)]"
            title={t("query.metadata.info")}
            onClick={(e) => {
              e.stopPropagation();
              setMetadataColumn({ column: col, el: e.currentTarget });
            }}
          >
            i
          </Button>
        )}
        footer={
          <>
            <span>{t("query.rowsAffected", { count: rowCount })}</span>
            <span>{t("query.duration", { duration: durationMs })}</span>
            <div className="flex-1" />
            <ZoomControls zoom={zoom} onZoomChange={setZoom} />
          </>
        }
      />

      {metadataColumn && (
        <ColumnMetadataPopover
          column={metadataColumn.column}
          anchorEl={metadataColumn.el}
          onClose={() => setMetadataColumn(null)}
        />
      )}
    </>
  );
}
