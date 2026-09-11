import { Command } from "cmdk";
import { useEffect, useMemo } from "react";

import { Dialog, DialogContent } from "@/components/ui/dialog";
import { useCommandStore } from "@/commons/stores/command.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { formatShortcut } from "@/commons/utils/platform";

export function CommandPalette() {
  const { t } = useTranslation();
  const isOpen = useCommandStore((s) => s.isOpen);
  const allCommands = useCommandStore((s) => s.commands);
  const close = useCommandStore((s) => s.close);
  const execute = useCommandStore((s) => s.executeCommand);

  const commands = useMemo(() => allCommands.filter((c) => !c.when || c.when()), [allCommands]);

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
      <DialogContent className="!top-[16vh] !w-[640px] !max-w-[640px] -translate-x-1/2 !-translate-y-0 overflow-hidden p-0 shadow-lg">
        <Command
          className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-[var(--text-secondary)] [&_[cmdk-group]]:px-2 [&_[cmdk-group]]:py-1 [&_[cmdk-input-wrapper]]:px-3 [&_[cmdk-input-wrapper]]:py-2 [&_[cmdk-item]]:px-3 [&_[cmdk-item]]:py-2.5"
          label="Command palette"
        >
          <Command.Input
            placeholder={t("commandPalette.placeholder")}
            className="flex h-11 w-full border-b border-[var(--border-subtle)] bg-transparent px-3 text-[13px] outline-none placeholder:text-[var(--text-tertiary)]"
          />
          <Command.List className="max-h-[520px] overflow-y-auto overflow-x-hidden py-2">
            <Command.Empty className="px-3 py-6 text-center text-sm text-[var(--text-secondary)]">
              {t("commandPalette.noResults")}
            </Command.Empty>
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
                      <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-[var(--border-strong)] bg-muted px-1.5 font-mono text-[11px] font-medium text-[var(--text-secondary)]">
                        {formatShortcut(cmd.keybinding)}
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
