import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

import type { WorkspaceTab } from "@/commons/types/workspace.types";

interface SortableTabProps {
  tab: WorkspaceTab;
  isActive: boolean;
  onActivate: () => void;
  onClose: (id: string, opts?: { skipDirtyCheck?: boolean }) => void;
  children: (props: {
    tab: WorkspaceTab;
    isActive: boolean;
    onActivate: () => void;
    onClose: (id: string, opts?: { skipDirtyCheck?: boolean }) => void;
    dragAttributes: ReturnType<typeof useSortable>["attributes"];
    dragListeners: ReturnType<typeof useSortable>["listeners"];
    isDragging: boolean;
  }) => React.ReactNode;
}

export function SortableTab({ tab, isActive, onActivate, onClose, children }: SortableTabProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: tab.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : undefined,
  };

  return (
    <div ref={setNodeRef} style={style}>
      {children({
        tab,
        isActive,
        onActivate,
        onClose,
        dragAttributes: attributes,
        dragListeners: listeners,
        isDragging,
      })}
    </div>
  );
}
