import { useTranslation } from "@/commons/locales/useTranslation";
import type { TreeNode } from "../types/schema.types";

interface SchemaTreeProps {
  treeNodes: TreeNode[];
  expandedNodes: Set<string>;
  selectedNodeId: string | null;
  onToggleNode: (nodeId: string) => void;
  onSelectNode: (schema: string, name: string, type: "table" | "view") => void;
}

export function SchemaTree({
  treeNodes,
  expandedNodes,
  selectedNodeId,
  onToggleNode,
  onSelectNode,
}: SchemaTreeProps) {
  const { t } = useTranslation();

  if (treeNodes.length === 0) {
    return (
      <div
        className="flex flex-1 items-center justify-center p-4 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("common.states.empty")}
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-auto py-1">
      {treeNodes.map((node) => (
        <TreeNodeItem
          key={node.id}
          node={node}
          depth={0}
          expandedNodes={expandedNodes}
          selectedNodeId={selectedNodeId}
          onToggleNode={onToggleNode}
          onSelectNode={onSelectNode}
        />
      ))}
    </div>
  );
}

interface TreeNodeItemProps {
  node: TreeNode;
  depth: number;
  expandedNodes: Set<string>;
  selectedNodeId: string | null;
  onToggleNode: (nodeId: string) => void;
  onSelectNode: (schema: string, name: string, type: "table" | "view") => void;
}

function TreeNodeItem({
  node,
  depth,
  expandedNodes,
  selectedNodeId,
  onToggleNode,
  onSelectNode,
}: TreeNodeItemProps) {
  const isExpanded = expandedNodes.has(node.id);
  const isSelected = selectedNodeId === node.id;

  const handleClick = () => {
    if (node.type === "schema") {
      onToggleNode(node.id);
    } else if (node.schemaName && node.tableName) {
      onSelectNode(node.schemaName, node.tableName, node.type);
    }
  };

  const icon =
    node.type === "schema"
      ? isExpanded
        ? "\u25BE"
        : "\u25B8"
      : node.type === "table"
        ? "\u25A6"
        : "\u25C7";

  return (
    <>
      <button
        className="flex w-full items-center gap-1.5 px-3 py-1 text-left text-sm transition-colors hover:bg-[var(--color-bg)]"
        style={{
          paddingLeft: `${12 + depth * 16}px`,
          backgroundColor: isSelected ? "var(--color-primary-light, rgba(59,130,246,0.1))" : undefined,
          color: isSelected ? "var(--color-primary, #3b82f6)" : "var(--color-text)",
          fontWeight: node.type === "schema" ? 500 : 400,
        }}
        onClick={handleClick}
        type="button"
      >
        <span className="w-4 text-center text-xs" style={{ color: "var(--color-text-secondary)" }}>
          {icon}
        </span>
        <span className="truncate">{node.label}</span>
      </button>

      {node.type === "schema" && isExpanded && node.children?.map((child) => (
        <TreeNodeItem
          key={child.id}
          node={child}
          depth={depth + 1}
          expandedNodes={expandedNodes}
          selectedNodeId={selectedNodeId}
          onToggleNode={onToggleNode}
          onSelectNode={onSelectNode}
        />
      ))}
    </>
  );
}
