import { useCallback, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useExecuteDdl, useIntrospect } from "../queries/schema.queries";
import { buildCreateTrigger, buildDropTrigger } from "../services/ddl-builder";
import type { TriggerDto } from "../types/schema.types";

interface TriggerManagerProps {
  connectionId: string;
  schema: string;
  table: string;
}

export function TriggerManager({ connectionId, schema, table }: TriggerManagerProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);
  const introspect = useIntrospect(connectionId);

  const tableTriggers = (introspect.data?.triggers ?? []).filter(
    (tr) => tr.tableName === table && tr.schema === schema,
  );

  const [triggerName, setTriggerName] = useState("");
  const [timing, setTiming] = useState("BEFORE");
  const [event, setEvent] = useState("INSERT");
  const [body, setBody] = useState("BEGIN\n  -- trigger body\nEND;");

  const handleCreate = useCallback(() => {
    if (!triggerName.trim() || !body.trim()) return;
    const sql = buildCreateTrigger(schema, table, triggerName.trim(), timing, event, body);
    executeDdl.mutate(sql, {
      onSuccess: () => {
        setTriggerName("");
        setBody("BEGIN\n  -- trigger body\nEND;");
      },
    });
  }, [triggerName, timing, event, body, schema, table, executeDdl]);

  const handleDrop = useCallback(() => {
    if (!triggerName.trim()) return;
    const sql = buildDropTrigger(schema, table, triggerName.trim());
    executeDdl.mutate(sql);
  }, [triggerName, schema, table, executeDdl]);

  return (
    <div className="flex flex-col gap-4 p-3">
      {/* Existing triggers list */}
      <div className="rounded-sm border border-[var(--app-border-subtle)] p-3">
        <h4 className="mb-2 text-xs font-semibold text-foreground">
          {t("schema.triggers")} ({tableTriggers.length})
        </h4>
        {tableTriggers.length === 0 ? (
          <p className="text-xs text-[var(--app-text-muted)]">{t("schema.noTriggers")}</p>
        ) : (
          <div className="space-y-1.5">
            {tableTriggers.map((tr) => (
              <TriggerRow
                key={tr.name}
                trigger={tr}
                onDrop={() => {
                  setTriggerName(tr.name);
                }}
                isPending={executeDdl.isPending}
              />
            ))}
          </div>
        )}
      </div>

      {/* CREATE / DROP form */}
      <div className="rounded-sm border border-[var(--app-border-subtle)] p-3">
        <h4 className="mb-2 text-xs font-semibold text-foreground">{t("schema.createTrigger")}</h4>

        <div className="space-y-2">
          <div>
            <Label
              htmlFor="trigger-name"
              className="mb-1 block text-xs text-[var(--app-text-muted)]"
            >
              {t("schema.triggerName")}
            </Label>
            <Input
              id="trigger-name"
              type="text"
              value={triggerName}
              onChange={(e) => setTriggerName(e.target.value)}
              className="w-full rounded-sm border border-[var(--app-border)] bg-background px-2 py-1 text-xs text-foreground outline-none focus:border-primary"
            />
          </div>

          <div className="flex gap-2">
            <div className="flex-1">
              <Label
                htmlFor="trigger-timing"
                className="mb-1 block text-xs text-[var(--app-text-muted)]"
              >
                {t("schema.triggerTiming")}
              </Label>
              <Select value={timing} onValueChange={setTiming}>
                <SelectTrigger
                  id="trigger-timing"
                  className="w-full rounded-sm border border-[var(--app-border)] bg-background px-2 py-1 text-xs text-foreground"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="BEFORE">BEFORE</SelectItem>
                  <SelectItem value="AFTER">AFTER</SelectItem>
                  <SelectItem value="INSTEAD OF">INSTEAD OF</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex-1">
              <Label
                htmlFor="trigger-event"
                className="mb-1 block text-xs text-[var(--app-text-muted)]"
              >
                {t("schema.triggerEvent")}
              </Label>
              <Select value={event} onValueChange={setEvent}>
                <SelectTrigger
                  id="trigger-event"
                  className="w-full rounded-sm border border-[var(--app-border)] bg-background px-2 py-1 text-xs text-foreground"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="INSERT">INSERT</SelectItem>
                  <SelectItem value="UPDATE">UPDATE</SelectItem>
                  <SelectItem value="DELETE">DELETE</SelectItem>
                  <SelectItem value="INSERT OR UPDATE">INSERT OR UPDATE</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <div>
            <Label
              htmlFor="trigger-body"
              className="mb-1 block text-xs text-[var(--app-text-muted)]"
            >
              {t("schema.triggerBody")}
            </Label>
            <Textarea
              id="trigger-body"
              value={body}
              onChange={(e) => setBody(e.target.value)}
              rows={5}
              className="w-full resize-y rounded-sm border border-[var(--app-border)] bg-background px-2 py-1.5 font-mono text-xs text-foreground outline-none focus:border-primary"
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

function TriggerRow({
  trigger,
  onDrop,
  isPending,
}: {
  trigger: TriggerDto;
  onDrop: () => void;
  isPending: boolean;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center justify-between rounded-sm border border-[var(--app-border-subtle)] px-2.5 py-1.5">
      <div className="flex flex-col gap-0.5">
        <span className="text-xs font-medium text-foreground">{trigger.name}</span>
        <div className="flex gap-1.5">
          <Badge variant="secondary" className="text-[10px]">
            {trigger.timing}
          </Badge>
          <Badge variant="secondary" className="text-[10px]">
            {trigger.event}
          </Badge>
          {!trigger.enabled && (
            <Badge variant="destructive" className="text-[10px]">
              DISABLED
            </Badge>
          )}
        </div>
      </div>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-6 text-[10px]"
        disabled={isPending}
        onClick={onDrop}
      >
        {t("schema.selectForDrop")}
      </Button>
    </div>
  );
}
