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
import { formatShortcut } from "@/commons/utils/platform";
import { useTranslation } from "@/commons/locales/useTranslation";

interface TabContextMenuProps {
  tab: {
    id: string;
    title: string;
    pinned: boolean;
    kind: string;
    resourceName: string;
  };
  children: React.ReactNode;
  onClose: (id: string, opts?: { skipDirtyCheck?: boolean }) => void;
  onCloseMany: (ids: string[]) => void;
}

export function TabContextMenu({ tab, children, onClose, onCloseMany }: TabContextMenuProps) {
  const { t } = useTranslation();
  const tabOrderKey = useWorkspaceStore((s) =>
    s.tabs.map((t) => `${t.id}:${Number(t.pinned)}`).join("|"),
  );
  const toggleTabPinned = useWorkspaceStore((s) => s.toggleTabPinned);
  const reopenLastClosed = useWorkspaceStore((s) => s.reopenLastClosed);

  const recentlyClosedCount = useWorkspaceStore((s) => s.recentlyClosed.length);

  const rawTabEntries = tabOrderKey.split("|").map((entry) => {
    const [id, pinned] = entry.split(":");
    return { id, pinned: pinned === "1" };
  });
  const tabEntries = [
    ...rawTabEntries.filter((tab) => tab.pinned),
    ...rawTabEntries.filter((tab) => !tab.pinned),
  ];

  const tabIdx = tabEntries.findIndex((t) => t.id === tab.id);
  const otherIds = tabEntries.filter((t) => t.id !== tab.id && !t.pinned).map((t) => t.id);
  const rightIds = tabEntries
    .filter((_, i) => i > tabIdx)
    .filter((t) => !t.pinned)
    .map((t) => t.id);
  const allUnpinnedIds = tabEntries.filter((t) => !t.pinned).map((t) => t.id);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(() => {});
  };

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={() => onClose(tab.id)}>
          <XIcon className="size-3.5" />
          {t("tabs.contextMenu.close")}
          <ContextMenuShortcut>{formatShortcut({ primary: true, key: "W" })}</ContextMenuShortcut>
        </ContextMenuItem>
        {otherIds.length > 0 && (
          <ContextMenuItem onClick={() => onCloseMany(otherIds)}>
            {t("tabs.contextMenu.closeOthers")}
          </ContextMenuItem>
        )}
        {rightIds.length > 0 && (
          <ContextMenuItem onClick={() => onCloseMany(rightIds)}>
            {t("tabs.contextMenu.closeToRight")}
          </ContextMenuItem>
        )}
        {allUnpinnedIds.length > 1 && (
          <ContextMenuItem onClick={() => onCloseMany(allUnpinnedIds)}>
            {t("tabs.contextMenu.closeAll")}
          </ContextMenuItem>
        )}
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => toggleTabPinned(tab.id)}>
          <PinIcon className="size-3.5" />
          {tab.pinned ? t("tabs.contextMenu.unpin") : t("tabs.contextMenu.pin")}
          <ContextMenuShortcut>
            {formatShortcut({ altKey: true, shiftKey: true, key: "P" })}
          </ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => copyToClipboard(tab.title)}>
          <CopyIcon className="size-3.5" />
          {t("tabs.contextMenu.copyTitle")}
        </ContextMenuItem>
        {tab.kind === "db-object" && (
          <ContextMenuItem onClick={() => copyToClipboard(tab.resourceName)}>
            <CopyIcon className="size-3.5" />
            {t("tabs.contextMenu.copyResource")}
          </ContextMenuItem>
        )}
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => reopenLastClosed()} disabled={recentlyClosedCount === 0}>
          {t("tabs.contextMenu.reopenClosed")}
          <ContextMenuShortcut>
            {formatShortcut({ primary: true, shiftKey: true, key: "T" })}
          </ContextMenuShortcut>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
