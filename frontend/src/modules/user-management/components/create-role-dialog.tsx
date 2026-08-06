import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";

interface CreateRoleDialogProps {
  open: boolean;
  onClose: () => void;
  onCreate: (name: string, login: boolean) => void;
}

export function CreateRoleDialog({
  open,
  onClose,
  onCreate,
}: CreateRoleDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [login, setLogin] = useState(true);

  if (!open) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    onCreate(name.trim(), login);
    setName("");
    setLogin(true);
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
    >
      <form
        onSubmit={handleSubmit}
        className="flex w-96 flex-col gap-4 rounded-md border border-border bg-background p-6"
      >
        <h2 className="text-lg font-semibold text-foreground">
          {t("userManagement.createRole")}
        </h2>

        <label className="flex flex-col gap-1">
          <span className="text-sm text-muted-foreground">
            {t("userManagement.roleName")}
          </span>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="rounded-sm border border-border bg-card px-3 py-2 text-sm text-foreground"
            autoFocus
          />
        </label>

        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={login}
            onChange={(e) => setLogin(e.target.checked)}
          />
          <span className="text-sm text-foreground">
            {t("userManagement.canLogin")}
          </span>
        </label>

        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.actions.cancel")}
          </Button>
          <Button type="submit" disabled={!name.trim()}>
            {t("common.actions.create")}
          </Button>
        </div>
      </form>
    </div>
  );
}
