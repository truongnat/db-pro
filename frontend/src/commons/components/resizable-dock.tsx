import type { ReactNode } from "react";

import { DockSeparator } from "@/commons/components/dock-separator";
import { useResizableDock } from "@/hooks/use-resizable-dock";

interface ResizableDockProps {
  children: [ReactNode, ReactNode];
  options?: Parameters<typeof useResizableDock>[0];
}

export function ResizableDock({ children, options }: ResizableDockProps) {
  const { topHeight, separatorProps, isCollapsed, containerRef } =
    useResizableDock(options);

  const [topPanel, bottomPanel] = children;

  return (
    <div ref={containerRef} className="flex min-h-0 flex-1 flex-col">
      {!isCollapsed && (
        <div className="overflow-hidden" style={{ height: topHeight }}>
          {topPanel}
        </div>
      )}
      <DockSeparator
        separatorProps={separatorProps}
        isCollapsed={isCollapsed}
      />
      <div className="flex min-h-0 flex-1 flex-col">{bottomPanel}</div>
    </div>
  );
}
