import { useCallback, useState } from "react";

import { Button } from "@/components/ui/button";
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
      <div className="rounded-sm border border-border p-3">
        <h4 className="mb-2 text-xs font-semibold text-foreground">
          {t("schema.createTrigger")}
        </h4>

        <div className="space-y-2">
          <div>
            <label className="mb-1 block text-xs text-muted-foreground">
              {t("schema.triggerName")}
            </label>
            <input
              type="text"
              value={triggerName}
              onChange={(e) => setTriggerName(e.target.value)}
              className="w-full rounded-sm border border-border bg-background px-2 py-1 text-xs text-foreground outline-none focus:border-primary"
            />
          </div>

          <div className="flex gap-2">
            <div className="flex-1">
              <label className="mb-1 block text-xs text-muted-foreground">
                {t("schema.triggerTiming")}
              </label>
              <select
                value={timing}
                onChange={(e) => setTiming(e.target.value)}
                className="w-full rounded-sm border border-border bg-background px-2 py-1 text-xs text-foreground"
              >
                <option value="BEFORE">BEFORE</option>
                <option value="AFTER">AFTER</option>
                <option value="INSTEAD OF">INSTEAD OF</option>
              </select>
            </div>
            <div className="flex-1">
              <label className="mb-1 block text-xs text-muted-foreground">
                {t("schema.triggerEvent")}
              </label>
              <select
                value={event}
                onChange={(e) => setEvent(e.target.value)}
                className="w-full rounded-sm border border-border bg-background px-2 py-1 text-xs text-foreground"
              >
                <option value="INSERT">INSERT</option>
                <option value="UPDATE">UPDATE</option>
                <option value="DELETE">DELETE</option>
                <option value="INSERT OR UPDATE">INSERT OR UPDATE</option>
              </select>
            </div>
          </div>

          <div>
            <label className="mb-1 block text-xs text-muted-foreground">
              {t("schema.triggerBody")}
            </label>
            <textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              rows={5}
              className="w-full resize-y rounded-sm border border-border bg-background px-2 py-1.5 font-mono text-xs text-foreground outline-none focus:border-primary"
            />
          </div>

          <div className="flex gap-2">
            <Button
              type="button"
              size="sm"
              disabled={!triggerName.trim() || !body.trim() || executeDdl.isPending}
              onClick={handleCreate}
            >
              {t("schema.createTrigger")}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="text-destructive"
              disabled={!triggerName.trim() || executeDdl.isPending}
              onClick={handleDrop}
            >
              {t("schema.dropTrigger")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
