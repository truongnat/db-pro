import type { ExplainPlan } from "../types/query.types";

interface ExplainPlanViewProps {
  plan: ExplainPlan;
}

interface PlanNode {
  type: string;
  cost?: number;
  rows?: number;
  width?: number;
  actualTime?: number;
  actualRows?: number;
  [key: string]: unknown;
}

function extractPlanNode(plan: unknown): PlanNode | null {
  if (!plan || typeof plan !== "object") return null;
  const obj = plan as Record<string, unknown>;
  
  // PostgreSQL format: { "Plan": { "Node Type": "Seq Scan", ... } }
  if (obj.Plan && typeof obj.Plan === "object") {
    return obj.Plan as PlanNode;
  }
  
  // Direct node format
  if (obj["Node Type"] || obj["nodeType"]) {
    return obj as PlanNode;
  }
  
  return null;
}

function getNodeTypeName(node: PlanNode): string {
  return (node["Node Type"] || node["nodeType"] || node.type || "Unknown") as string;
}

function getCost(node: PlanNode): number | undefined {
  return (node["Total Cost"] ?? node["totalCost"] ?? node.cost) as number | undefined;
}

function getRows(node: PlanNode): number | undefined {
  return (node["Plan Rows"] ?? node["planRows"] ?? node.rows) as number | undefined;
}

function getActualTime(node: PlanNode): number | undefined {
  return (node["Actual Total Time"] ?? node["actualTotalTime"] ?? node.actualTime) as number | undefined;
}

function getActualRows(node: PlanNode): number | undefined {
  return (node["Actual Rows"] ?? node["actualRows"] ?? node.actualRows) as number | undefined;
}

function getSubplans(node: PlanNode): PlanNode[] {
  const plans = node.Plans || node.plans || node["Subplan"] || node["subplan"];
  if (Array.isArray(plans)) {
    return plans.map(extractPlanNode).filter((n): n is PlanNode => n !== null);
  }
  if (plans && typeof plans === "object") {
    const sub = extractPlanNode(plans);
    return sub ? [sub] : [];
  }
  return [];
}

function formatNumber(n: number | undefined): string {
  if (n === undefined) return "—";
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toFixed(n < 10 ? 2 : 0);
}

function PlanNodeView({ node, depth }: { node: PlanNode; depth: number }) {
  const nodeType = getNodeTypeName(node);
  const cost = getCost(node);
  const rows = getRows(node);
  const actualTime = getActualTime(node);
  const actualRows = getActualRows(node);
  const subplans = getSubplans(node);
  
  const relation = String(node["Relation Name"] || node["relationName"] || node["table"] || "");
  const index = String(node["Index Name"] || node["indexName"] || node["index"] || "");
  const condition = String(node["Filter"] || node["filter"] || node["Index Cond"] || node["indexCond"] || "");
  
  return (
    <div style={{ paddingLeft: depth * 20 }} className="py-1">
      <div className="flex items-center gap-2">
        <span className="font-medium text-foreground">{nodeType}</span>
        {relation && (
          <span className="rounded bg-blue-100 px-1.5 py-0.5 text-xs text-blue-800 dark:bg-blue-900/30 dark:text-blue-300">
            {relation}
          </span>
        )}
        {index && (
          <span className="rounded bg-green-100 px-1.5 py-0.5 text-xs text-green-800 dark:bg-green-900/30 dark:text-green-300">
            {index}
          </span>
        )}
      </div>
      
      <div className="flex gap-4 text-xs text-[var(--app-text-muted)]">
        {cost !== undefined && (
          <span>cost: {formatNumber(cost)}</span>
        )}
        {rows !== undefined && (
          <span>rows: {formatNumber(rows)}</span>
        )}
        {actualTime !== undefined && (
          <span className="text-amber-600 dark:text-amber-400">
            time: {formatNumber(actualTime)}ms
          </span>
        )}
        {actualRows !== undefined && (
          <span className="text-amber-600 dark:text-amber-400">
            actual: {formatNumber(actualRows)}
          </span>
        )}
      </div>
      
      {condition && (
        <div className="text-xs text-[var(--app-text-muted)] italic">
          {condition}
        </div>
      )}
      
      {subplans.length > 0 && (
        <div className="mt-1">
          {subplans.map((sub, i) => (
            <PlanNodeView key={i} node={sub} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

function renderGenericNode(key: string, value: unknown, depth: number): React.ReactNode {
  if (value === null || value === undefined) {
    return (
      <div key={key} style={{ paddingLeft: depth * 16 }}>
        <span className="text-[var(--app-text-muted)]">{key}: </span>
        <span className="italic text-[var(--app-text-muted)]">null</span>
      </div>
    );
  }

  if (typeof value === "object" && !Array.isArray(value)) {
    const entries = Object.entries(value as Record<string, unknown>);
    return (
      <details key={key} open style={{ paddingLeft: depth * 16 }}>
        <summary className="cursor-pointer select-none py-0.5 text-foreground">
          {key}
        </summary>
        <div>{entries.map(([k, v]) => renderGenericNode(k, v, depth + 1))}</div>
      </details>
    );
  }

  if (Array.isArray(value)) {
    return (
      <details key={key} open style={{ paddingLeft: depth * 16 }}>
        <summary className="cursor-pointer select-none py-0.5 text-foreground">
          {key} [{value.length}]
        </summary>
        <div>
          {value.map((item, i) => renderGenericNode(`[${i}]`, item, depth + 1))}
        </div>
      </details>
    );
  }

  return (
    <div key={key} style={{ paddingLeft: depth * 16 }} className="py-0.5">
      <span className="text-[var(--app-text-muted)]">{key}: </span>
      <span className="text-foreground">{String(value)}</span>
    </div>
  );
}

export function ExplainPlanView({ plan }: ExplainPlanViewProps) {
  const rootNode = extractPlanNode(plan);
  
  if (rootNode) {
    return (
      <div className="h-full overflow-auto p-4 text-sm" style={{ fontFamily: "monospace" }}>
        <PlanNodeView node={rootNode} depth={0} />
      </div>
    );
  }
  
  // Fallback to generic JSON tree view
  const entries = Object.entries(plan);
  return (
    <div className="h-full overflow-auto p-4 text-sm" style={{ fontFamily: "monospace" }}>
      {entries.map(([key, value]) => renderGenericNode(key, value, 0))}
    </div>
  );
}
