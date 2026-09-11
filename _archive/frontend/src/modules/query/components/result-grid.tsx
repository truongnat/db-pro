import { useState, useMemo } from "react";
import { Info } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import { UnifiedGrid } from "@/modules/unified-grid/components/unified-grid";
import type { GridSort } from "@/modules/unified-grid/types";

import type { ColumnMeta, Row } from "../types/query.types";
import type { SortState } from "@/commons/types/workspace.types";
import { ColumnMetadataPopover } from "./column-metadata-popover";
import { ZoomControls } from "./zoom-controls";

const LARGE_SORT_THRESHOLD = 10_000;

interface ResultGridProps {
  columns: ColumnMeta[];
  rows: Row[];
  sort: SortState;
  onSort: (column: string) => void;
  durationMs: number;
  rowCount: number;
}

export function ResultGrid({ columns, rows, sort, onSort, durationMs, rowCount }: ResultGridProps) {
  const { t } = useTranslation();
  const [zoom, setZoom] = useState(100);
  const [metadataColumn, setMetadataColumn] = useState<{
    column: ColumnMeta;
    el: HTMLElement;
  } | null>(null);

  /* Adapt single SortState → GridSort[] for UnifiedGrid */
  const sorts = useMemo<GridSort[]>(
    () =>
      sort.column && sort.direction ? [{ column: sort.column, direction: sort.direction }] : [],
    [sort],
  );
  const isLargeSortedResult = rows.length >= LARGE_SORT_THRESHOLD && sorts.length > 0;

  return (
    <>
      {isLargeSortedResult && (
        <div
          role="status"
          className="border-b border-[var(--border-subtle)] bg-[var(--state-warning)]/10 px-3 py-1 text-[11px] text-[var(--state-warning)]"
        >
          {t("query.largeSortWarning", { count: rows.length })}
        </div>
      )}
      <UnifiedGrid
        columns={columns}
        rows={rows}
        sorts={sorts}
        onSort={onSort}
        contentStyle={zoom !== 100 ? { zoom: zoom / 100 } : undefined}
        emptyState={<p className="text-sm text-[var(--text-secondary)]">{t("query.noResults")}</p>}
        renderHeaderExtra={(col) => (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="shrink-0 rounded p-0 text-[var(--text-tertiary)] opacity-0 transition-opacity hover:text-primary focus-visible:opacity-100 group-hover/header:opacity-100"
            title={t("query.metadata.info")}
            aria-label={`${t("query.metadata.info")}: ${col.name}`}
            onClick={(e) => {
              e.stopPropagation();
              setMetadataColumn({ column: col, el: e.currentTarget });
            }}
          >
            <Info className="h-3 w-3" aria-hidden="true" />
          </Button>
        )}
        footer={
          <>
            <span>{t("query.rowsAffected", { count: rowCount })}</span>
            <span>{t("query.columnsCount", { count: columns.length })}</span>
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
