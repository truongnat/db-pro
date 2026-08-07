import { PinIcon, XIcon } from "lucide-react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { WorkspaceTab } from "@/commons/types/workspace.types";

interface TabContextMenuProps {
  tab: WorkspaceTab;
  children: React.ReactNode;
  onClose: (id: string, opts?: { skipDirtyCheck?: boolean }) => void;
  onCloseMany: (ids: string[]) => void;
}

export function TabContextMenu({ tab, children, onClose, onCloseMany }: TabContextMenuProps) {
  const tabs = useWorkspaceStore((s) => s.tabs);
  const toggleTabPinned = useWorkspaceStore((s) => s.toggleTabPinned);
  const reopenLastClosed = useWorkspaceStore((s) => s.reopenLastClosed);
  const recentlyClosedCount = useWorkspaceStore((s) => s.recentlyClosed.length);

  const tabIdx = tabs.findIndex((t) => t.id === tab.id);
  const otherIds = tabs.filter((t) => t.id !== tab.id).map((t) => t.id);
  const rightIds = tabs.filter((_, i) => i > tabIdx).map((t) => t.id);

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={() => onClose(tab.id)}>
          <XIcon className="size-3.5" />
          Close
        </ContextMenuItem>
        {tabs.length > 1 && (
          <ContextMenuItem onClick={() => onCloseMany(otherIds)}>
            Close Others
          </ContextMenuItem>
        )}
        {rightIds.length > 0 && (
          <ContextMenuItem onClick={() => onCloseMany(rightIds)}>
            Close to Right
          </ContextMenuItem>
        )}
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => toggleTabPinned(tab.id)}>
          <PinIcon className="size-3.5" />
          {tab.pinned ? "Unpin" : "Pin"}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          onClick={() => reopenLastClosed()}
          disabled={recentlyClosedCount === 0}
        >
          Reopen Closed Tab
          <ContextMenuShortcut>Ctrl+Shift+T</ContextMenuShortcut>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
