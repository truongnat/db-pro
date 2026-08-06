import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaColumnDto } from "../types/schema.types";
import { sortColumnsForDisplay } from "../types/schema.types";

interface ColumnListProps {
  columns: SchemaColumnDto[];
}

export function ColumnList({ columns }: ColumnListProps) {
  const { t } = useTranslation();

  if (columns.length === 0) {
    return (
      <div
        className="p-4 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("schema.noColumns")}
      </div>
    );
  }

  const sorted = sortColumnsForDisplay(columns);

  const headerStyle: React.CSSProperties = {
    color: "var(--color-text-secondary)",
    borderBottom: "1px solid var(--color-border)",
  };

  return (
    <table className="w-full text-sm">
      <thead>
        <tr>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.columnName")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.columnDataType")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.columnNullable")}</th>
          <th className="px-3 py-2 text-left font-medium" style={headerStyle}>{t("schema.columnDefault")}</th>
          <th className="px-3 py-2 text-center font-medium" style={headerStyle}>{t("schema.columnPk")}</th>
        </tr>
      </thead>
      <tbody>
        {sorted.map((col) => (
          <tr
            key={col.name}
            className="transition-colors hover:bg-[var(--color-bg)]"
          >
            <td className="px-3 py-1.5 font-mono text-sm">{col.name}</td>
            <td className="px-3 py-1.5 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {col.dataType}
            </td>
            <td className="px-3 py-1.5" style={{ color: "var(--color-text-secondary)" }}>
              {col.nullable ? "YES" : "NO"}
            </td>
            <td className="px-3 py-1.5 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {col.defaultValue ?? "\u2014"}
            </td>
            <td className="px-3 py-1.5 text-center">
              {col.isPrimaryKey ? "\uD83D\uDD11" : ""}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
