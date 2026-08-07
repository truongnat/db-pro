import type { ComponentType } from "react";

export interface Keybinding {
  ctrlKey?: boolean;
  metaKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  key: string;
}

export interface Command {
  id: string;
  labelKey: string;
  icon?: ComponentType<{ className?: string }>;
  keybinding?: Keybinding;
  groupKey?: string;
  when?: () => boolean;
  execute: () => void | Promise<void>;
}
