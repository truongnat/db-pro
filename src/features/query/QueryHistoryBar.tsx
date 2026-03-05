import { useEffect, useState } from "react";
import { History, Play, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import type { QueryHistoryEntry } from "./history";

type QueryHistoryBarProps = {
  items: QueryHistoryEntry[];
  busy: boolean;
  onApply: (historyId: string) => void;
  onRun: (historyId: string) => void;
  onClear: () => void;
};

const MAX_SQL_PREVIEW_LENGTH = 72;

function toSqlPreview(sql: string): string {
  const compact = sql.replace(/\s+/g, " ").trim();
  if (compact.length <= MAX_SQL_PREVIEW_LENGTH) {
    return compact;
  }
  return `${compact.slice(0, MAX_SQL_PREVIEW_LENGTH - 1)}…`;
}

function toHistoryOptionLabel(entry: QueryHistoryEntry): string {
  const timestamp = new Date(entry.recordedAt).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  return `[${timestamp}] ${entry.runSource} • ${toSqlPreview(entry.sql)}`;
}

export function QueryHistoryBar({
  items,
  busy,
  onApply,
  onRun,
  onClear,
}: QueryHistoryBarProps) {
  const hasItems = items.length > 0;
  const [selectedHistoryId, setSelectedHistoryId] = useState<string>("");

  useEffect(() => {
    if (!hasItems) {
      setSelectedHistoryId("");
      return;
    }

    if (!selectedHistoryId || !items.some((entry) => entry.id === selectedHistoryId)) {
      setSelectedHistoryId(items[0].id);
    }
  }, [hasItems, items, selectedHistoryId]);

  return (
    <div className="rounded-md border border-border/70 bg-muted/35 px-3 py-2">
      <div className="mb-2 flex items-center justify-between">
        <span className="inline-flex items-center gap-1 text-xs font-medium text-muted-foreground">
          <History className="h-3.5 w-3.5" />
          Query History
        </span>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-7 px-2 text-xs"
          onClick={onClear}
          disabled={!hasItems || busy}
        >
          <Trash2 className="mr-1 h-3.5 w-3.5" />
          Clear
        </Button>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Select
          value={selectedHistoryId || undefined}
          onValueChange={setSelectedHistoryId}
          disabled={!hasItems || busy}
        >
          <SelectTrigger className="h-8 min-w-[240px] flex-1 text-xs">
            <SelectValue placeholder="No history yet" />
          </SelectTrigger>
          <SelectContent>
            {items.map((entry) => (
              <SelectItem key={entry.id} value={entry.id}>
                {toHistoryOptionLabel(entry)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-8"
          disabled={!hasItems || busy}
          onClick={() => {
            if (!selectedHistoryId) {
              return;
            }
            onApply(selectedHistoryId);
          }}
        >
          Load
        </Button>

        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-8"
          disabled={!hasItems || busy}
          onClick={() => {
            if (!selectedHistoryId) {
              return;
            }
            onRun(selectedHistoryId);
          }}
        >
          <Play className="mr-1 h-3.5 w-3.5" />
          Run
        </Button>
      </div>
    </div>
  );
}
