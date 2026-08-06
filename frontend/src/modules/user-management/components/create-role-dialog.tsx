import { useState } from "react";
import { useTranslation } from "react-i18next";

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
        className="flex w-96 flex-col gap-4 rounded-[var(--radius-md)] border p-6"
        style={{
          backgroundColor: "var(--color-bg)",
          borderColor: "var(--color-border)",
        }}
      >
        <h2
          className="text-lg font-semibold"
          style={{ color: "var(--color-text)" }}
        >
          {t("userManagement.createRole")}
        </h2>

        <label className="flex flex-col gap-1">
          <span
            className="text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("userManagement.roleName")}
          </span>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
            style={{
              backgroundColor: "var(--color-surface)",
              borderColor: "var(--color-border)",
              color: "var(--color-text)",
            }}
            autoFocus
          />
        </label>

        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={login}
            onChange={(e) => setLogin(e.target.checked)}
          />
          <span className="text-sm" style={{ color: "var(--color-text)" }}>
            {t("userManagement.canLogin")}
          </span>
        </label>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-[var(--radius-sm)] border px-4 py-2 text-sm transition-colors hover:bg-[var(--color-bg)]"
            style={{
              borderColor: "var(--color-border)",
              color: "var(--color-text)",
            }}
          >
            {t("common.actions.cancel")}
          </button>
          <button
            type="submit"
            disabled={!name.trim()}
            className="rounded-[var(--radius-sm)] px-4 py-2 text-sm text-white transition-colors disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary)" }}
          >
            {t("common.actions.create")}
          </button>
        </div>
      </form>
    </div>
  );
}
