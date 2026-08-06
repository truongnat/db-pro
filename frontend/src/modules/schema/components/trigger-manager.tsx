import { useCallback, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useExecuteDdl } from "../queries/schema.queries";

interface TriggerManagerProps {
  connectionId: string;
  schema: string;
  table: string;
}

export function TriggerManager({ connectionId, schema, table }: TriggerManagerProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);

  const [triggerName, setTriggerName] = useState("");
  const [timing, setTiming] = useState("BEFORE");
  const [event, setEvent] = useState("INSERT");
  const [body, setBody] = useState("BEGIN\n  -- trigger body\nEND;");

  const handleCreate = useCallback(() => {
    if (!triggerName.trim() || !body.trim()) return;
    const sql = `CREATE TRIGGER "${triggerName.trim()}"\n  ${timing} ${event} ON "${schema}"."${table}"\n  ${body}`;
    executeDdl.mutate(sql, {
      onSuccess: () => {
        setTriggerName("");
        setBody("BEGIN\n  -- trigger body\nEND;");
      },
    });
  }, [triggerName, timing, event, body, schema, table, executeDdl]);

  const handleDrop = useCallback(() => {
    if (!triggerName.trim()) return;
    const sql = `DROP TRIGGER "${triggerName.trim()}" ON "${schema}"."${table}"`;
    executeDdl.mutate(sql);
  }, [triggerName, schema, table, executeDdl]);

  return (
    <div className="flex flex-col gap-4 p-3">
      <div
        className="rounded-[var(--radius-sm)] border p-3"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h4 className="mb-2 text-xs font-semibold" style={{ color: "var(--color-text)" }}>
          {t("schema.createTrigger")}
        </h4>

        <div className="space-y-2">
          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("schema.triggerName")}
            </label>
            <input
              type="text"
              value={triggerName}
              onChange={(e) => setTriggerName(e.target.value)}
              className="w-full rounded-[var(--radius-sm)] border px-2 py-1 text-xs outline-none focus:border-[var(--color-primary,#3b82f6)]"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
            />
          </div>

          <div className="flex gap-2">
            <div className="flex-1">
              <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
                {t("schema.triggerTiming")}
              </label>
              <select
                value={timing}
                onChange={(e) => setTiming(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border px-2 py-1 text-xs"
                style={{
                  borderColor: "var(--color-border)",
                  backgroundColor: "var(--color-bg)",
                  color: "var(--color-text)",
                }}
              >
                <option value="BEFORE">BEFORE</option>
                <option value="AFTER">AFTER</option>
                <option value="INSTEAD OF">INSTEAD OF</option>
              </select>
            </div>
            <div className="flex-1">
              <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
                {t("schema.triggerEvent")}
              </label>
              <select
                value={event}
                onChange={(e) => setEvent(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border px-2 py-1 text-xs"
                style={{
                  borderColor: "var(--color-border)",
                  backgroundColor: "var(--color-bg)",
                  color: "var(--color-text)",
                }}
              >
                <option value="INSERT">INSERT</option>
                <option value="UPDATE">UPDATE</option>
                <option value="DELETE">DELETE</option>
                <option value="INSERT OR UPDATE">INSERT OR UPDATE</option>
              </select>
            </div>
          </div>

          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("schema.triggerBody")}
            </label>
            <textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              rows={5}
              className="w-full resize-y rounded-[var(--radius-sm)] border px-2 py-1.5 font-mono text-xs outline-none focus:border-[var(--color-primary,#3b82f6)]"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
            />
          </div>

          <div className="flex gap-2">
            <button
              type="button"
              disabled={!triggerName.trim() || !body.trim() || executeDdl.isPending}
              className="rounded-[var(--radius-sm)] px-3 py-1.5 text-xs text-white disabled:opacity-50"
              style={{ backgroundColor: "var(--color-primary, #3b82f6)" }}
              onClick={handleCreate}
            >
              {t("schema.createTrigger")}
            </button>
            <button
              type="button"
              disabled={!triggerName.trim() || executeDdl.isPending}
              className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs disabled:opacity-50"
              style={{
                borderColor: "var(--color-border)",
                color: "var(--color-error, #ef4444)",
              }}
              onClick={handleDrop}
            >
              {t("schema.dropTrigger")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
