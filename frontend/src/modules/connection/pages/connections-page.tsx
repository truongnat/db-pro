import { useNavigate } from "@tanstack/react-router";

import { useTranslation } from "@/commons/locales/useTranslation";

import { ConnectionList } from "../components/connection-list";

export function ConnectionsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold" style={{ color: "var(--color-text)" }}>
          {t("connection.title")}
        </h1>
        <button
          className="rounded-[var(--radius-sm)] px-4 py-2 text-sm text-white transition-colors hover:opacity-90"
          style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
          onClick={() => navigate({ to: "/connection-editor" })}
        >
          {t("connection.new")}
        </button>
      </div>

      <ConnectionList
        onEdit={(id) => navigate({ to: "/connection-editor", search: { id } })}
      />
    </div>
  );
}
