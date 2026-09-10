import { memo } from "react";
import type { NodeProps } from "@xyflow/react";

import { ErDotNode } from "./er-dot-node";
import { ErCompactNode } from "./er-compact-node";
import { ErSummaryNode } from "./er-summary-node";
import { ErDetailedNode } from "./er-detailed-node";
import type { TableNodeData } from "./types";

/**
 * LOD dispatcher — the single React Flow node type registered for tables.
 *
 * Switches the render tree between four separate leaf components based on the
 * node's resolved `lod`. Because each branch mounts a different component,
 * React unmounts the hidden DOM entirely (true LOD, never CSS-hidden).
 *
 * The leaves are memoized, so the switch itself is the only cost React Flow
 * pays per node at any zoom level.
 */
export const ErTableNode = memo(function ErTableNode(props: NodeProps) {
  const lod = (props.data as TableNodeData | undefined)?.lod ?? "detail";
  const data = props.data as TableNodeData | undefined;
  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    document.dispatchEvent(
      new CustomEvent("er-node-focus", {
        detail: { nodeKey: props.id },
      }),
    );
  };

  if (!data) return null;

  const node = (() => {
    switch (lod) {
      case "dot":
        return <ErDotNode {...props} />;
      case "compact":
        return <ErCompactNode {...props} />;
      case "summary":
        return <ErSummaryNode {...props} />;
      case "detail":
        return <ErDetailedNode {...props} />;
    }
  })();

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`${data.schema}.${data.label}`}
      className="contents"
      onKeyDown={handleKeyDown}
    >
      {node}
    </div>
  );
});

export type { TableNodeData } from "./types";
