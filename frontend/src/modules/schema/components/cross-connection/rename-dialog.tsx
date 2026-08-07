import { useState } from "react";

import { Button } from "@/components/ui/button";
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
    <div className="fixed inset-0 z-[var(--z-overlay)] flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-md border border-border bg-background p-6">
        <h3 className="mb-4 text-lg font-medium text-foreground">
          {t("schema.crossConn.rename")} {objectType}
        </h3>

        <div className="mb-4">
          <p className="text-sm text-muted-foreground">
            {schema}.{oldName}
          </p>
        </div>

        <div className="mb-4">
          <label className="mb-1 block text-sm font-medium text-foreground">
            {t("schema.crossConn.newName")}
          </label>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            className="w-full rounded-sm border border-border px-3 py-2 text-sm text-foreground"
            placeholder={oldName}
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSubmit();
            }}
          />
        </div>

        {mutation.error && (
          <div className="mb-4 rounded-sm bg-destructive px-3 py-2 text-sm text-white">
            {(mutation.error as Error).message}
          </div>
        )}

        <div className="flex justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={onClose}
          >
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="button"
            onClick={handleSubmit}
            disabled={!newName.trim() || mutation.isPending}
          >
            {mutation.isPending ? t("common.states.loading") : t("schema.crossConn.rename")}
          </Button>
        </div>
      </div>
    </div>
  );
}
