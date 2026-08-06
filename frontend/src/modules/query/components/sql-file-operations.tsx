import { useCallback } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

import { useTranslation } from "@/commons/locales/useTranslation";

interface SqlFileOperationsProps {
  sql: string;
  onSqlLoaded: (sql: string) => void;
}

export function SqlFileOperations({ sql, onSqlLoaded }: SqlFileOperationsProps) {
  const { t } = useTranslation();

  const handleImport = useCallback(async () => {
    try {
      const selected = await open({
        filters: [{ name: "SQL Files", extensions: ["sql"] }],
        multiple: false,
      });

      if (!selected || typeof selected !== "string") return;

      const content = await readTextFile(selected);
      onSqlLoaded(content);
    } catch (err) {
      console.error("Failed to import SQL file:", err);
    }
  }, [onSqlLoaded]);

  const handleExport = useCallback(async () => {
    if (!sql.trim()) return;

    try {
      const filePath = await save({
        filters: [{ name: "SQL Files", extensions: ["sql"] }],
        defaultPath: "query.sql",
      });

      if (!filePath) return;

      await writeTextFile(filePath, sql);
    } catch (err) {
      console.error("Failed to export SQL file:", err);
    }
  }, [sql]);

  return (
    <div className="flex gap-2">
      <button
        type="button"
        onClick={handleImport}
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
        style={{
          borderColor: "var(--color-border)",
          color: "var(--color-text)",
        }}
      >
        {t("query.importSql")}
      </button>
      <button
        type="button"
        onClick={handleExport}
        disabled={!sql.trim()}
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg)] disabled:opacity-50"
        style={{
          borderColor: "var(--color-border)",
          color: "var(--color-text)",
        }}
      >
        {t("query.exportSql")}
      </button>
    </div>
  );
}
