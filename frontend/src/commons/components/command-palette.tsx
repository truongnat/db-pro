import { Command } from "cmdk";
import { useEffect, useMemo } from "react";

import {
  Dialog,
  DialogContent,
} from "@/components/ui/dialog";
import { useCommandStore } from "@/commons/stores/command.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionList, useConnect } from "@/modules/connection/queries/connection.queries";
import type { Keybinding } from "@/commons/types/command.types";

function formatKeybinding(kb: Keybinding): string {
  const isMac = navigator.platform.toLowerCase().includes("mac");
  const parts: string[] = [];

  if (kb.ctrlKey) parts.push(isMac ? "⌃" : "Ctrl");
  if (kb.metaKey) parts.push(isMac ? "⌘" : "Win");
  if (kb.shiftKey) parts.push(isMac ? "⇧" : "Shift");
  if (kb.altKey) parts.push(isMac ? "⌥" : "Alt");

  const key = kb.key.length === 1 ? kb.key.toUpperCase() : kb.key;
  parts.push(key);

  return isMac ? parts.join("") : parts.join("+");
}

export function CommandPalette() {
  const { t } = useTranslation();
  const isOpen = useCommandStore((s) => s.isOpen);
  const commands = useCommandStore((s) => s.getAvailableCommands());
  const close = useCommandStore((s) => s.close);
  const execute = useCommandStore((s) => s.executeCommand);

  const recentConnections = useRecentStore((s) => s.recentConnections);
  const addRecentConnection = useRecentStore((s) => s.addRecentConnection);
  const { data: connections } = useConnectionList();
  const connectMutation = useConnect();

  const connectionMap = useMemo(() => {
    const map = new Map<string, string>();
    if (connections) {
      for (const conn of connections) {
        map.set(conn.id, conn.name);
      }
    }
    return map;
  }, [connections]);

  const recentItems = useMemo(
    () =>
      recentConnections
        .filter((rc) => connectionMap.has(rc.connectionId))
        .map((rc) => ({
          connectionId: rc.connectionId,
          name: connectionMap.get(rc.connectionId)!,
        })),
    [recentConnections, connectionMap],
  );

  const grouped = useMemo(() => {
    const map = new Map<string, typeof commands>();
    for (const cmd of commands) {
      const groupKey = cmd.groupKey ?? "commands.groups.general";
      if (!map.has(groupKey)) map.set(groupKey, []);
      map.get(groupKey)!.push(cmd);
    }
    return Array.from(map.entries());
  }, [commands]);

  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close();
      }
    };

    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, [isOpen, close]);

  if (!isOpen) return null;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && close()}>
      <DialogContent className="overflow-hidden p-0 shadow-lg">
        <Command
          className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-muted-foreground [&_[cmdk-group]]:px-2 [&_[cmdk-group]]:py-1 [&_[cmdk-input-wrapper]]:px-3 [&_[cmdk-input-wrapper]]:py-2 [&_[cmdk-item]]:px-3 [&_[cmdk-item]]:py-2"
          label="Command palette"
        >
          <Command.Input
            placeholder={t("commandPalette.placeholder")}
            className="flex h-10 w-full border-b border-border bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
          <Command.List className="max-h-[400px] overflow-y-auto overflow-x-hidden py-2">
            <Command.Empty className="px-3 py-6 text-center text-sm text-muted-foreground">
              {t("commandPalette.noResults")}
            </Command.Empty>
            {recentItems.length > 0 && (
              <Command.Group heading={t("commands.groups.recent")}>
                {recentItems.map((item) => (
                  <Command.Item
                    key={`recent-${item.connectionId}`}
                    value={`recent:${item.name}`}
                    onSelect={() => {
                      connectMutation.mutate(item.connectionId, {
                        onSuccess: () => addRecentConnection(item.connectionId),
                      });
                      close();
                    }}
                    className="relative flex cursor-default select-none items-center gap-2 rounded-sm text-sm outline-none aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground"
                  >
                    <span>{item.name}</span>
                  </Command.Item>
                ))}
              </Command.Group>
            )}
            {grouped.map(([groupKey, cmds]) => (
              <Command.Group key={groupKey} heading={t(groupKey)}>
                {cmds.map((cmd) => (
                  <Command.Item
                    key={cmd.id}
                    value={cmd.id}
                    onSelect={() => {
                      execute(cmd.id);
                      close();
                    }}
                    className="relative flex cursor-default select-none items-center gap-2 rounded-sm text-sm outline-none aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground"
                  >
                    {cmd.icon && <cmd.icon className="h-4 w-4" />}
                    <span>{t(cmd.labelKey)}</span>
                    {cmd.keybinding && (
                      <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
                        {formatKeybinding(cmd.keybinding)}
                      </kbd>
                    )}
                  </Command.Item>
                ))}
              </Command.Group>
            ))}
          </Command.List>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
