import { ChevronLeftIcon, ChevronRightIcon, ListIcon } from "lucide-react";
import { useState } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { cn } from "@/lib/utils";

interface TabScrollControlsProps {
  canScrollLeft: boolean;
  canScrollRight: boolean;
  isOverflowing: boolean;
  onScrollLeft: () => void;
  onScrollRight: () => void;
}

export function TabScrollLeft({ canScrollLeft, onScrollLeft }: Pick<TabScrollControlsProps, "canScrollLeft" | "onScrollLeft">) {
  if (!canScrollLeft) return null;
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      className="h-full shrink-0 rounded-none border-r border-border px-1"
      onClick={onScrollLeft}
      aria-label="Scroll tabs left"
    >
      <ChevronLeftIcon className="size-3.5" />
    </Button>
  );
}

export function TabScrollRight({ canScrollRight, onScrollRight }: Pick<TabScrollControlsProps, "canScrollRight" | "onScrollRight">) {
  if (!canScrollRight) return null;
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      className="h-full shrink-0 rounded-none border-l border-border px-1"
      onClick={onScrollRight}
      aria-label="Scroll tabs right"
    >
      <ChevronRightIcon className="size-3.5" />
    </Button>
  );
}

export function TabOverflowMenu({ isOverflowing }: Pick<TabScrollControlsProps, "isOverflowing">) {
  const [open, setOpen] = useState(false);
  const tabs = useWorkspaceStore((s) => s.tabs);
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const activateTab = useWorkspaceStore((s) => s.activateTab);

  if (!isOverflowing) return null;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-full shrink-0 rounded-none border-l border-border px-1"
          aria-label="Show all tabs"
        >
          <ListIcon className="size-3.5" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-56 p-1">
        <div className="flex flex-col">
          {tabs.map((tab) => (
            <Button
              key={tab.id}
              type="button"
              variant="ghost"
              className={cn(
                "flex h-auto w-full items-center gap-2 justify-start rounded-md px-2 py-1.5 text-left text-sm transition-colors",
                tab.id === activeTabId && "bg-accent text-accent-foreground",
              )}
              onClick={() => {
                activateTab(tab.id);
                setOpen(false);
              }}
            >
              {tab.dirty && (
                <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-primary" />
              )}
              {tab.pinned && <span className="text-[10px]">📌</span>}
              <span className={cn("truncate", tab.preview && "italic")}>
                {tab.title}
              </span>
            </Button>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
