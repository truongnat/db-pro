import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import { SortableContext, horizontalListSortingStrategy } from "@dnd-kit/sortable";
import { Code2, Loader2, PinIcon, TableIcon, XIcon } from "lucide-react";
import { useCallback, useState } from "react";
import type { DraggableAttributes } from "@dnd-kit/core";
import type { SyntheticListenerMap } from "@dnd-kit/core/dist/hooks/utilities";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";

import { SortableTab } from "@/commons/components/sortable-tab";
import { TabContextMenu } from "@/commons/components/tab-context-menu";
import {
  TabOverflowMenu,
  TabScrollLeft,
  TabScrollRight,
} from "@/commons/components/tab-scroll-controls";
import { useOverflowDetection } from "@/hooks/use-overflow-detection";
import { useTabCloseGuard } from "@/hooks/use-tab-close-guard";
import { useTabKeyboard } from "@/hooks/use-tab-keyboard";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { WorkspaceTab } from "@/commons/types/workspace.types";

function TabKindIcon({ tab }: { tab: WorkspaceTab }) {
  if (tab.kind === "query") {
    return <Code2 className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)]" />;
  }
  if (tab.data.objectType === "view") {
    return <TableIcon className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)] opacity-70" />;
  }
  return <TableIcon className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)]" />;
}

function TabItem({
  tab,
  isActive,
  onActivate,
  onClose,
  dragListeners,
  dragAttributes,
}: {
  tab: WorkspaceTab;
  isActive: boolean;
  onActivate: () => void;
  onClose: (id: string, opts?: { skipDirtyCheck?: boolean }) => void;
  dragListeners?: SyntheticListenerMap;
  dragAttributes?: DraggableAttributes;
}) {
  return (
    <div
      className={cn(
        "group relative flex shrink-0 cursor-pointer items-center gap-1.5 px-3 py-2 text-[13px] transition-colors",
        "min-w-[120px] max-w-[220px]",
        isActive
          ? "bg-[var(--app-surface-3)] text-foreground font-medium"
          : "text-[var(--app-text-muted)] hover:bg-[var(--app-surface-2)] hover:text-foreground",
      )}
      onClick={onActivate}
      title={tab.title}
      onAuxClick={(e) => {
        if (e.button === 1) {
          e.preventDefault();
          onClose(tab.id);
        }
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onActivate();
        }
        if (e.key === "ArrowRight") {
          const next = (e.currentTarget.nextElementSibling ??
            e.currentTarget.parentElement?.firstElementChild) as HTMLElement | null;
          next?.focus();
        }
        if (e.key === "ArrowLeft") {
          const prev = (e.currentTarget.previousElementSibling ??
            e.currentTarget.parentElement?.lastElementChild) as HTMLElement | null;
          prev?.focus();
        }
        if (e.key === "Home") {
          const first = e.currentTarget.parentElement?.firstElementChild as HTMLElement | null;
          first?.focus();
        }
        if (e.key === "End") {
          const last = e.currentTarget.parentElement?.lastElementChild as HTMLElement | null;
          last?.focus();
        }
      }}
      role="tab"
      aria-selected={isActive}
      tabIndex={isActive ? 0 : -1}
      {...dragListeners}
      {...dragAttributes}
    >
      {/* Active indicator — top 2px line */}
      {isActive && (
        <span className="absolute inset-x-0 top-0 h-[2px] bg-primary" />
      )}
      {/* Kind icon / running spinner / error dot */}
      {tab.kind === "query" && tab.data.status === "running" ? (
        <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-primary" />
      ) : tab.kind === "query" && tab.data.status === "error" ? (
        <span className="h-2 w-2 shrink-0 rounded-full bg-[var(--app-danger)]" />
      ) : (
        <TabKindIcon tab={tab} />
      )}
      {/* Dirty dot — replaces close button when dirty and not hovered */}
      {tab.dirty && !tab.pinned && (
        <span
          className="h-2 w-2 shrink-0 rounded-full bg-primary group-hover:hidden"
          aria-label="Unsaved changes"
        />
      )}
      {tab.pinned && (
        <PinIcon className="h-3 w-3 shrink-0 text-[var(--app-text-muted)]" aria-label="Pinned" />
      )}
      <span className={cn("flex-1 truncate text-[13px]", tab.preview && "italic opacity-70")}>
        {tab.title}
      </span>
      {/* Close button — visible on hover, hidden when dirty (dot shows instead) */}
      {!tab.pinned && (
        <button
          type="button"
          className={cn(
            "flex h-4 w-4 shrink-0 items-center justify-center rounded text-[var(--app-text-muted)] transition-opacity hover:bg-[var(--app-active)] hover:text-foreground",
            tab.dirty ? "opacity-0 group-hover:opacity-100" : "opacity-0 group-hover:opacity-100",
          )}
          onClick={(e) => {
            e.stopPropagation();
            onClose(tab.id);
          }}
          aria-label={`Close ${tab.title}`}
        >
          <XIcon className="h-3 w-3" />
        </button>
      )}
    </div>
  );
}

export function WorkspaceTabBar() {
  const { t } = useTranslation();
  const tabs = useWorkspaceStore((s) => s.tabs);
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const activateTab = useWorkspaceStore((s) => s.activateTab);
  const reorderTabs = useWorkspaceStore((s) => s.reorderTabs);

  const { containerRef, isOverflowing, canScrollLeft, canScrollRight, scrollLeft, scrollRight } =
    useOverflowDetection();
  const { dialogOpen, dirtyCount, onConfirm, onCancel, requestClose, requestCloseMany } =
    useTabCloseGuard();

  useTabKeyboard();

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));
  const [draggingTab, setDraggingTab] = useState<WorkspaceTab | null>(null);

  const pinnedTabs = tabs.filter((t) => t.pinned);
  const unpinnedTabs = tabs.filter((t) => !t.pinned);
  const pinnedCount = pinnedTabs.length;

  const handleDragStart = useCallback(
    (event: DragEndEvent) => {
      const tab = tabs.find((t) => t.id === event.active.id);
      if (tab) setDraggingTab(tab);
    },
    [tabs],
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      setDraggingTab(null);
      const { active, over } = event;
      if (!over || active.id === over.id) return;

      const fromIdx = tabs.findIndex((t) => t.id === active.id);
      const toIdx = tabs.findIndex((t) => t.id === over.id);
      if (fromIdx === -1 || toIdx === -1) return;
      if (fromIdx < pinnedCount || toIdx < pinnedCount) return;

      reorderTabs(fromIdx, toIdx);
    },
    [tabs, pinnedCount, reorderTabs],
  );

  const handleDragCancel = useCallback(() => {
    setDraggingTab(null);
  }, []);

  if (tabs.length === 0) return null;

  return (
    <>
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onDragCancel={handleDragCancel}
      >
        <div className="flex items-center border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-1)]">
          <TabScrollLeft canScrollLeft={canScrollLeft} onScrollLeft={scrollLeft} />

          <div
            ref={containerRef}
            className="flex min-w-0 flex-1 overflow-x-auto scrollbar-none"
            role="tablist"
            aria-label="Workspace tabs"
            aria-orientation="horizontal"
          >
            {pinnedTabs.map((tab) => (
              <TabContextMenu
                key={tab.id}
                tab={tab}
                onClose={requestClose}
                onCloseMany={requestCloseMany}
              >
                <TabItem
                  tab={tab}
                  isActive={tab.id === activeTabId}
                  onActivate={() => activateTab(tab.id)}
                  onClose={requestClose}
                />
              </TabContextMenu>
            ))}

            <SortableContext
              items={unpinnedTabs.map((t) => t.id)}
              strategy={horizontalListSortingStrategy}
            >
              {unpinnedTabs.map((tab) => (
                <TabContextMenu
                  key={tab.id}
                  tab={tab}
                  onClose={requestClose}
                  onCloseMany={requestCloseMany}
                >
                  <SortableTab
                    tab={tab}
                    isActive={tab.id === activeTabId}
                    onActivate={() => activateTab(tab.id)}
                    onClose={requestClose}
                  >
                    {(props) => (
                      <TabItem
                        tab={props.tab}
                        isActive={props.isActive}
                        onActivate={props.onActivate}
                        onClose={props.onClose}
                        dragListeners={props.dragListeners}
                        dragAttributes={props.dragAttributes}
                      />
                    )}
                  </SortableTab>
                </TabContextMenu>
              ))}
            </SortableContext>
          </div>

          <TabScrollRight canScrollRight={canScrollRight} onScrollRight={scrollRight} />
          <TabOverflowMenu isOverflowing={isOverflowing} />
        </div>

        <DragOverlay>
          {draggingTab ? (
            <div className="flex items-center gap-1.5 border border-[var(--app-border)] bg-[var(--app-surface-3)] px-3 py-2 text-[13px] shadow-lg opacity-90">
              {draggingTab.kind === "query" && draggingTab.data.status === "running" ? (
                <Loader2 className="h-3.5 w-3.5 shrink-0 animate-spin text-primary" />
              ) : (
                <TabKindIcon tab={draggingTab} />
              )}
              {draggingTab.dirty && (
                <span className="h-2 w-2 shrink-0 rounded-full bg-primary" />
              )}
              <span className={cn("max-w-[180px] truncate", draggingTab.preview && "italic opacity-70")}>
                {draggingTab.title}
              </span>
            </div>
          ) : null}
        </DragOverlay>
      </DndContext>

      <AlertDialog open={dialogOpen} onOpenChange={(open) => !open && onCancel()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{dirtyCount > 1 ? t("tabs.closeDialog.titleMultiple") : t("tabs.closeDialog.title")}</AlertDialogTitle>
            <AlertDialogDescription>
              {dirtyCount === 1
                ? t("tabs.closeDialog.descriptionSingle")
                : t("tabs.closeDialog.descriptionMultiple", { count: dirtyCount })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.actions.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={onConfirm}>{t("common.actions.close")}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
