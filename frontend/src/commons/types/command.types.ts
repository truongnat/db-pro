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
  label: string;
  icon?: ComponentType<{ className?: string }>;
  keybinding?: Keybinding;
  group?: string;
  when?: () => boolean;
  execute: () => void | Promise<void>;
}
