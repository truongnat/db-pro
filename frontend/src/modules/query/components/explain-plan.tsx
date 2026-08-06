import type { ExplainPlan } from "../types/query.types";

interface ExplainPlanViewProps {
  plan: ExplainPlan;
}

function renderNode(key: string, value: unknown, depth: number): React.ReactNode {
  if (value === null || value === undefined) {
    return (
      <div key={key} style={{ paddingLeft: depth * 16 }}>
        <span style={{ color: "var(--color-text-secondary)" }}>{key}: </span>
        <span style={{ fontStyle: "italic", color: "var(--color-text-secondary)" }}>
          null
        </span>
      </div>
    );
  }

  if (typeof value === "object" && !Array.isArray(value)) {
    const entries = Object.entries(value as Record<string, unknown>);
    return (
      <details key={key} open style={{ paddingLeft: depth * 16 }}>
        <summary
          className="cursor-pointer select-none py-0.5"
          style={{ color: "var(--color-text)" }}
        >
          {key}
        </summary>
        <div>{entries.map(([k, v]) => renderNode(k, v, depth + 1))}</div>
      </details>
    );
  }

  if (Array.isArray(value)) {
    return (
      <details key={key} open style={{ paddingLeft: depth * 16 }}>
        <summary
          className="cursor-pointer select-none py-0.5"
          style={{ color: "var(--color-text)" }}
        >
          {key} [{value.length}]
        </summary>
        <div>
          {value.map((item, i) => renderNode(`[${i}]`, item, depth + 1))}
        </div>
      </details>
    );
  }

  return (
    <div key={key} style={{ paddingLeft: depth * 16 }} className="py-0.5">
      <span style={{ color: "var(--color-text-secondary)" }}>{key}: </span>
      <span style={{ color: "var(--color-text)" }}>{String(value)}</span>
    </div>
  );
}

export function ExplainPlanView({ plan }: ExplainPlanViewProps) {
  const entries = Object.entries(plan);

  return (
    <div
      className="h-full overflow-auto p-4 text-sm"
      style={{ fontFamily: "monospace" }}
    >
      {entries.map(([key, value]) => renderNode(key, value, 0))}
    </div>
  );
}
