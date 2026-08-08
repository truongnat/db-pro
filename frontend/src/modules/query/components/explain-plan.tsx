import { useState } from "react";

import type { ExplainPlan } from "../types/query.types";

interface ExplainPlanViewProps {
  plan: ExplainPlan;
}

interface PlanNode {
  type: string;
  [key: string]: unknown;
}

// ── Extraction helpers ──

function extractPlanNode(plan: unknown): PlanNode | null {
  if (!plan || typeof plan !== "object") return null;
  const obj = plan as Record<string, unknown>;
  if (obj.Plan && typeof obj.Plan === "object") return obj.Plan as PlanNode;
  if (obj["Node Type"] || obj["nodeType"]) return obj as PlanNode;
  return null;
}

function getNodeTypeName(node: PlanNode): string {
  return String(node["Node Type"] ?? node["nodeType"] ?? node.type ?? "Unknown");
}

function getCost(node: PlanNode): number | undefined {
  return (node["Total Cost"] ?? node["totalCost"] ?? node.cost) as number | undefined;
}

function getStartupCost(node: PlanNode): number | undefined {
  return (node["Startup Cost"] ?? node["startupCost"]) as number | undefined;
}

function getRows(node: PlanNode): number | undefined {
  return (node["Plan Rows"] ?? node["planRows"] ?? node.rows) as number | undefined;
}

function getWidth(node: PlanNode): number | undefined {
  return (node["Plan Width"] ?? node["planWidth"] ?? node.width) as number | undefined;
}

function getActualTime(node: PlanNode): number | undefined {
  return (node["Actual Total Time"] ?? node["actualTotalTime"]) as number | undefined;
}

function getActualRows(node: PlanNode): number | undefined {
  return (node["Actual Rows"] ?? node["actualRows"]) as number | undefined;
}

function getRelation(node: PlanNode): string {
  return String(node["Relation Name"] ?? node["relationName"] ?? node["table"] ?? "");
}

function getIndex(node: PlanNode): string {
  return String(node["Index Name"] ?? node["indexName"] ?? node["index"] ?? "");
}

function getSubplans(node: PlanNode): PlanNode[] {
  const plans = node.Plans ?? node.plans ?? node["Subplan"] ?? node["subplan"];
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

// ── Summary header ──

function PlanSummary({ node }: { node: PlanNode }) {
  const cost = getCost(node);
  const startup = getStartupCost(node);
  const rows = getRows(node);
  const width = getWidth(node);

  return (
    <div className="mb-4 flex flex-wrap gap-x-6 gap-y-2 rounded-md bg-[var(--app-surface-2)] px-4 py-3">
      <SummaryItem label="Total Cost" value={formatNumber(cost)} />
      <SummaryItem label="Startup Cost" value={formatNumber(startup)} />
      <SummaryItem label="Estimated Rows" value={formatNumber(rows)} />
      <SummaryItem label="Width" value={formatNumber(width)} />
    </div>
  );
}

function SummaryItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline gap-2">
      <span className="text-[11px] text-[var(--app-text-muted)]">{label}</span>
      <span className="text-[13px] font-medium tabular-nums text-foreground">{value}</span>
    </div>
  );
}

// ── Tree node ──

function TreeNode({
  node,
  path,
  selectedId,
  onSelect,
}: {
  node: PlanNode;
  path: string;
  selectedId: string | null;
  onSelect: (id: string, node: PlanNode) => void;
}) {
  const nodeType = getNodeTypeName(node);
  const relation = getRelation(node);
  const index = getIndex(node);
  const cost = getCost(node);
  const rows = getRows(node);
  const actualTime = getActualTime(node);
  const actualRows = getActualRows(node);
  const subplans = getSubplans(node);

  const isSelected = selectedId === path;

  const depth = path.split(".").length - 1;

  return (
    <div style={{ paddingLeft: depth * 24 }}>
      <div
        className={`cursor-pointer rounded-md px-3 py-2 transition-colors ${
          isSelected
            ? "bg-[var(--app-active)] ring-1 ring-inset ring-primary/30"
            : "hover:bg-[var(--app-hover)]"
        }`}
        onClick={() => onSelect(path, node)}
      >
        <div className="flex items-center gap-2">
          <span className="text-[13px] font-medium text-foreground">{nodeType}</span>
          {relation && (
            <span className="rounded bg-primary/10 px-1.5 py-0.5 text-[11px] text-primary">
              {relation}
            </span>
          )}
          {index && (
            <span className="rounded bg-emerald-500/10 px-1.5 py-0.5 text-[11px] text-emerald-500">
              {index}
            </span>
          )}
        </div>
        <div className="mt-1 flex flex-wrap gap-x-4 gap-y-0.5 text-[11px] text-[var(--app-text-muted)]">
          {cost !== undefined && <span>cost {formatNumber(cost)}</span>}
          {rows !== undefined && <span>rows {formatNumber(rows)}</span>}
          {actualTime !== undefined && (
            <span className="text-amber-500">time {formatNumber(actualTime)}ms</span>
          )}
          {actualRows !== undefined && (
            <span className="text-amber-500">actual {formatNumber(actualRows)}</span>
          )}
        </div>
      </div>

      {subplans.length > 0 && (
        <div className="mt-0.5">
          {subplans.map((sub, i) => (
            <TreeNode
              key={`${path}.${i}`}
              node={sub}
              path={`${path}.${i}`}
              selectedId={selectedId}
              onSelect={onSelect}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// ── Details panel ──

function NodeDetails({ node }: { node: PlanNode }) {
  const nodeType = getNodeTypeName(node);
  const relation = getRelation(node);

  // Collect detail properties (exclude sub-plans and already-shown fields)
  const skipKeys = new Set(["Plans", "plans", "Subplan", "subplan", "Node Type", "nodeType", "type"]);
  const properties = Object.entries(node).filter(
    ([key]) => !skipKeys.has(key) && typeof node[key] !== "object",
  );

  return (
    <div className="flex flex-col gap-3">
      <div>
        <h3 className="text-[13px] font-medium text-foreground">{nodeType}</h3>
        {relation && (
          <p className="text-[11px] text-[var(--app-text-muted)]">on {relation}</p>
        )}
      </div>
      <div className="flex flex-col gap-1">
        {properties.map(([key, value]) => (
          <div key={key} className="flex items-baseline justify-between gap-2 text-[12px]">
            <span className="text-[var(--app-text-muted)]">{humanizeKey(key)}</span>
            <span className="tabular-nums text-foreground">{formatValue(value)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function humanizeKey(key: string): string {
  return key
    .replace(/([A-Z])/g, " $1")
    .trim()
    .replace(/^(.)/, (c) => c.toUpperCase());
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (typeof value === "number") return formatNumber(value);
  return String(value);
}

// ── Generic JSON fallback ──

function renderGenericNode(key: string, value: unknown, depth: number): React.ReactNode {
  if (value === null || value === undefined) {
    return (
      <div key={key} style={{ paddingLeft: depth * 16 }} className="py-0.5 text-[12px]">
        <span className="text-[var(--app-text-muted)]">{key}: </span>
        <span className="italic text-[var(--app-text-muted)]">null</span>
      </div>
    );
  }
  if (typeof value === "object" && !Array.isArray(value)) {
    const entries = Object.entries(value as Record<string, unknown>);
    return (
      <details key={key} open style={{ paddingLeft: depth * 16 }}>
        <summary className="cursor-pointer select-none py-0.5 text-[12px] text-foreground">{key}</summary>
        <div>{entries.map(([k, v]) => renderGenericNode(k, v, depth + 1))}</div>
      </details>
    );
  }
  if (Array.isArray(value)) {
    return (
      <details key={key} open style={{ paddingLeft: depth * 16 }}>
        <summary className="cursor-pointer select-none py-0.5 text-[12px] text-foreground">{key} [{value.length}]</summary>
        <div>{value.map((item, i) => renderGenericNode(`[${i}]`, item, depth + 1))}</div>
      </details>
    );
  }
  return (
    <div key={key} style={{ paddingLeft: depth * 16 }} className="py-0.5 text-[12px]">
      <span className="text-[var(--app-text-muted)]">{key}: </span>
      <span className="text-foreground">{String(value)}</span>
    </div>
  );
}

// ── Main view ──

export function ExplainPlanView({ plan }: ExplainPlanViewProps) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<PlanNode | null>(null);

  const handleSelect = (id: string, node: PlanNode) => {
    setSelectedId(id);
    setSelectedNode(node);
  };

  const rootNode = extractPlanNode(plan);

  if (rootNode) {
    return (
      <div className="flex h-full overflow-hidden">
        {/* Left — plan tree (60%) */}
        <div className="flex min-w-0 flex-[3] flex-col overflow-auto">
          <div className="p-4">
            <PlanSummary node={rootNode} />
            <TreeNode node={rootNode} path="0" selectedId={selectedId} onSelect={handleSelect} />
          </div>
        </div>

        {/* Right — node details (40%) */}
        {selectedNode && (
          <div className="flex-[2] overflow-auto bg-[var(--app-surface-1)] p-4">
            <NodeDetails node={selectedNode} />
          </div>
        )}
      </div>
    );
  }

  // Fallback: generic JSON tree
  const entries = Object.entries(plan);
  return (
    <div className="h-full overflow-auto p-4 text-[12px]" style={{ fontFamily: "monospace" }}>
      {entries.map(([key, value]) => renderGenericNode(key, value, 0))}
    </div>
  );
}
