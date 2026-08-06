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
    <div
      className="flex gap-1 border-b px-3 py-1"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: "var(--color-surface)",
      }}
    >
      {multiResults.map((r, idx) => (
        <button
          key={idx}
          className="rounded-t px-3 py-1 text-xs transition-colors"
          style={{
            backgroundColor:
              idx === multiResultIndex ? "var(--color-bg)" : "transparent",
            color:
              idx === multiResultIndex
                ? "var(--color-text)"
                : "var(--color-text-secondary)",
            borderBottom:
              idx === multiResultIndex
                ? "2px solid var(--color-primary,#3b82f6)"
                : "2px solid transparent",
          }}
          onClick={() => {
            setMultiResultIndex(idx);
            setResult(multiResults[idx]);
          }}
        >
          Result {idx + 1} ({r.rowCount} rows)
        </button>
      ))}
    </div>
  );
}
