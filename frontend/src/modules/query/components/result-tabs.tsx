import { Button } from "@/components/ui/button";

import { useQueryModuleStore } from "../state/query.store";

export function ResultTabs() {
  const multiResults = useQueryModuleStore((s) =>
    s.tabs.find((t) => t.id === s.activeTabId)?.multiResults ?? null,
  );
  const multiResultIndex = useQueryModuleStore((s) =>
    s.tabs.find((t) => t.id === s.activeTabId)?.multiResultIndex ?? 0,
  );
  const setMultiResultIndex = useQueryModuleStore((s) => s.setMultiResultIndex);
  const setResult = useQueryModuleStore((s) => s.setResult);

  if (!multiResults || multiResults.length <= 1) return null;

  return (
    <div className="flex gap-1 border-b border-border bg-card px-3 py-1">
      {multiResults.map((r, idx) => (
        <Button
          key={idx}
          type="button"
          variant="ghost"
          className={`rounded-t px-3 py-1 text-xs transition-colors ${
            idx === multiResultIndex
              ? "bg-background text-foreground"
              : "text-muted-foreground"
          } ${
            idx === multiResultIndex
              ? "border-b-2 border-primary"
              : "border-b-2 border-transparent"
          }`}
          onClick={() => {
            setMultiResultIndex(idx);
            setResult(multiResults[idx]);
          }}
        >
          Result {idx + 1} ({r.rowCount} rows)
        </Button>
      ))}
    </div>
  );
}
