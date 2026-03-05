import { useMemo, useState } from "react";
import { FileCode2, Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { DbEngine } from "@/types";

import { SQL_TEMPLATES } from "./templates";

type SqlTemplateBarProps = {
  engine: DbEngine | undefined;
  busy: boolean;
  onApplyTemplate: (sql: string, label: string) => void;
};

export function SqlTemplateBar({
  engine,
  busy,
  onApplyTemplate,
}: SqlTemplateBarProps) {
  const [selectedTemplateId, setSelectedTemplateId] = useState(SQL_TEMPLATES[0]?.id ?? "");

  const selectedTemplate = useMemo(
    () => SQL_TEMPLATES.find((template) => template.id === selectedTemplateId),
    [selectedTemplateId],
  );

  return (
    <div className="rounded-md border border-border/70 bg-muted/35 px-3 py-2">
      <div className="mb-2 flex items-center gap-1 text-xs font-medium text-muted-foreground">
        <FileCode2 className="h-3.5 w-3.5" />
        SQL Templates
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Select
          value={selectedTemplateId}
          onValueChange={setSelectedTemplateId}
          disabled={busy}
        >
          <SelectTrigger className="h-8 min-w-[220px] flex-1 text-xs">
            <SelectValue placeholder="Select template" />
          </SelectTrigger>
          <SelectContent>
            {SQL_TEMPLATES.map((template) => (
              <SelectItem key={template.id} value={template.id}>
                {template.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-8"
          disabled={!selectedTemplate || busy}
          onClick={() => {
            if (!selectedTemplate) {
              return;
            }

            onApplyTemplate(
              selectedTemplate.buildSql(engine),
              selectedTemplate.label,
            );
          }}
        >
          <Plus className="mr-1 h-3.5 w-3.5" />
          Insert
        </Button>
      </div>

      {selectedTemplate && (
        <p className="mt-2 text-[11px] text-muted-foreground">
          {selectedTemplate.description}
        </p>
      )}
    </div>
  );
}
