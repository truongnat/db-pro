import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";

interface PaginationProps {
  page: number;
  pageSize: number;
  totalCount: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (size: number) => void;
}

const PAGE_SIZES = [25, 50, 100, 200];

export function Pagination({
  page,
  pageSize,
  totalCount,
  onPageChange,
  onPageSizeChange,
}: PaginationProps) {
  const { t } = useTranslation();
  const totalPages = Math.max(1, Math.ceil(totalCount / pageSize));

  return (
    <div className="flex items-center gap-3 border-t border-border bg-card px-3 py-1.5 text-xs text-muted-foreground">
      <span>
        {t("dataGrid.page")} {page} {t("dataGrid.of")} {totalPages}
      </span>

      <div className="flex items-center gap-1">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-1.5 py-0.5 text-xs hover:bg-background disabled:opacity-40"
          disabled={page <= 1}
          onClick={() => onPageChange(1)}
        >
          &laquo;
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-1.5 py-0.5 text-xs hover:bg-background disabled:opacity-40"
          disabled={page <= 1}
          onClick={() => onPageChange(page - 1)}
        >
          &lsaquo;
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-1.5 py-0.5 text-xs hover:bg-background disabled:opacity-40"
          disabled={page >= totalPages}
          onClick={() => onPageChange(page + 1)}
        >
          &rsaquo;
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-1.5 py-0.5 text-xs hover:bg-background disabled:opacity-40"
          disabled={page >= totalPages}
          onClick={() => onPageChange(totalPages)}
        >
          &raquo;
        </Button>
      </div>

      <select
        className="rounded border border-border bg-card px-1.5 py-0.5 text-xs text-foreground"
        value={pageSize}
        onChange={(e) => onPageSizeChange(Number(e.target.value))}
      >
        {PAGE_SIZES.map((size) => (
          <option key={size} value={size}>
            {size} {t("dataGrid.pageSize")}
          </option>
        ))}
      </select>

      <span>
        {totalCount} {t("query.rowsAffected", { count: totalCount }).replace(/^\d+\s*/, "")}
      </span>
    </div>
  );
}
