import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useRenameSchemaObject } from "../../queries/schema.queries";

interface RenameDialogProps {
  connectionId: string | null;
  objectType: string;
  schema: string;
  oldName: string;
  onClose: () => void;
}

export function RenameDialog({
  connectionId,
  objectType,
  schema,
  oldName,
  onClose,
}: RenameDialogProps) {
  const { t } = useTranslation();
  const [newName, setNewName] = useState("");
  const mutation = useRenameSchemaObject(connectionId);

  const handleSubmit = () => {
    if (!newName.trim()) return;
    mutation.mutate(
      { objectType, schema, oldName, newName: newName.trim() },
      { onSuccess: () => onClose() },
    );
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div
        className="w-full max-w-md rounded-[var(--radius-md)] border p-6"
        style={{ backgroundColor: "var(--color-bg)", borderColor: "var(--color-border)" }}
      >
        <h3 className="mb-4 text-lg font-medium" style={{ color: "var(--color-text)" }}>
          {t("schema.crossConn.rename")} {objectType}
        </h3>

        <div className="mb-4">
          <p className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
            {schema}.{oldName}
          </p>
        </div>

        <div className="mb-4">
          <label className="mb-1 block text-sm font-medium" style={{ color: "var(--color-text)" }}>
            {t("schema.crossConn.newName")}
          </label>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            className="w-full rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
            style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
            placeholder={oldName}
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSubmit();
            }}
          />
        </div>

        {mutation.error && (
          <div className="mb-4 rounded-[var(--radius-sm)] px-3 py-2 text-sm" style={{ backgroundColor: "var(--color-error,#ef4444)", color: "white" }}>
            {(mutation.error as Error).message}
          </div>
        )}

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-[var(--radius-sm)] border px-4 py-2 text-sm transition-colors hover:bg-[var(--color-surface)]"
            style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
          >
            {t("common.actions.cancel")}
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={!newName.trim() || mutation.isPending}
            className="rounded-[var(--radius-sm)] px-4 py-2 text-sm text-white disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
          >
            {mutation.isPending ? t("common.states.loading") : t("schema.crossConn.rename")}
          </button>
        </div>
      </div>
    </div>
  );
}
