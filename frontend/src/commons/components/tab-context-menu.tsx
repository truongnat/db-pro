import { CopyIcon, PinIcon, XIcon } from "lucide-react";

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

function getTabResourceName(tab: WorkspaceTab): string {
  if (tab.kind === "db-object") {
    return `${tab.data.schema}.${tab.data.objectName}`;
  }
  return tab.title;
}

export function TabContextMenu({ tab, children, onClose, onCloseMany }: TabContextMenuProps) {
  const tabs = useWorkspaceStore((s) => s.tabs);
  const toggleTabPinned = useWorkspaceStore((s) => s.toggleTabPinned);
  const reopenLastClosed = useWorkspaceStore((s) => s.reopenLastClosed);
  const recentlyClosedCount = useWorkspaceStore((s) => s.recentlyClosed.length);

  const tabIdx = tabs.findIndex((t) => t.id === tab.id);
  const otherIds = tabs.filter((t) => t.id !== tab.id && !t.pinned).map((t) => t.id);
  const rightIds = tabs.filter((_, i) => i > tabIdx && !_.pinned).map((t) => t.id);
  const allUnpinnedIds = tabs.filter((t) => !t.pinned).map((t) => t.id);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(() => {});
  };

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={() => onClose(tab.id)}>
          <XIcon className="size-3.5" />
          Close
          <ContextMenuShortcut>Ctrl+W</ContextMenuShortcut>
        </ContextMenuItem>
        {otherIds.length > 0 && (
          <ContextMenuItem onClick={() => onCloseMany(otherIds)}>Close Others</ContextMenuItem>
        )}
        {rightIds.length > 0 && (
          <ContextMenuItem onClick={() => onCloseMany(rightIds)}>Close to Right</ContextMenuItem>
        )}
        {allUnpinnedIds.length > 1 && (
          <ContextMenuItem onClick={() => onCloseMany(allUnpinnedIds)}>Close All</ContextMenuItem>
        )}
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => toggleTabPinned(tab.id)}>
          <PinIcon className="size-3.5" />
          {tab.pinned ? "Unpin" : "Pin"}
          <ContextMenuShortcut>Alt+Shift+P</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => copyToClipboard(tab.title)}>
          <CopyIcon className="size-3.5" />
          Copy Tab Title
        </ContextMenuItem>
        {tab.kind === "db-object" && (
          <ContextMenuItem onClick={() => copyToClipboard(getTabResourceName(tab))}>
            <CopyIcon className="size-3.5" />
            Copy Resource Name
          </ContextMenuItem>
        )}
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => reopenLastClosed()} disabled={recentlyClosedCount === 0}>
          Reopen Closed Tab
          <ContextMenuShortcut>Ctrl+Shift+T</ContextMenuShortcut>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
