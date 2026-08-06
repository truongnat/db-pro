import { useTranslation } from "@/commons/locales/useTranslation";

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
    <div
      className="flex items-center gap-3 border-t px-3 py-1.5 text-xs"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: "var(--color-surface)",
        color: "var(--color-text-secondary)",
      }}
    >
      <span>
        {t("dataGrid.page")} {page} {t("dataGrid.of")} {totalPages}
      </span>

      <div className="flex items-center gap-1">
        <button
          className="rounded px-1.5 py-0.5 transition-colors hover:bg-[var(--color-bg)] disabled:opacity-40"
          disabled={page <= 1}
          onClick={() => onPageChange(1)}
          type="button"
        >
          &laquo;
        </button>
        <button
          className="rounded px-1.5 py-0.5 transition-colors hover:bg-[var(--color-bg)] disabled:opacity-40"
          disabled={page <= 1}
          onClick={() => onPageChange(page - 1)}
          type="button"
        >
          &lsaquo;
        </button>
        <button
          className="rounded px-1.5 py-0.5 transition-colors hover:bg-[var(--color-bg)] disabled:opacity-40"
          disabled={page >= totalPages}
          onClick={() => onPageChange(page + 1)}
          type="button"
        >
          &rsaquo;
        </button>
        <button
          className="rounded px-1.5 py-0.5 transition-colors hover:bg-[var(--color-bg)] disabled:opacity-40"
          disabled={page >= totalPages}
          onClick={() => onPageChange(totalPages)}
          type="button"
        >
          &raquo;
        </button>
      </div>

      <select
        className="rounded border px-1.5 py-0.5 text-xs"
        style={{
          borderColor: "var(--color-border)",
          backgroundColor: "var(--color-surface)",
          color: "var(--color-text)",
        }}
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
