/**
 * Platform-aware helpers for displaying keyboard shortcuts in the UI.
 *
 * macOS  → symbol notation (⌘⇧⌃⌥)
 * Other  → human-readable (Ctrl+Shift+…)
 */

export const isMac: boolean =
  typeof navigator !== "undefined" && navigator.platform.toLowerCase().includes("mac");

/** Modifier keys in the canonical display order. */
interface ShortcutMods {
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
}

/**
 * Format a keyboard shortcut for display.
 *
 * ```ts
 * formatShortcut({ ctrlKey: true, key: "Enter" })
 * // → "⌘↵" on macOS, "Ctrl+Enter" elsewhere
 * ```
 */
export function formatShortcut(kb: ShortcutMods & { key: string }): string {
  const parts: string[] = [];

  if (isMac) {
    if (kb.ctrlKey) parts.push("⌃");
    if (kb.metaKey) parts.push("⌘");
    if (kb.shiftKey) parts.push("⇧");
    if (kb.altKey) parts.push("⌥");

    const key = SYMBOL_KEYS[kb.key] ?? (kb.key.length === 1 ? kb.key.toUpperCase() : kb.key);
    parts.push(key);
    return parts.join("");
  }

  if (kb.ctrlKey) parts.push("Ctrl");
  if (kb.shiftKey) parts.push("Shift");
  if (kb.altKey) parts.push("Alt");
  if (kb.metaKey) parts.push("Win");

  parts.push(kb.key);
  return parts.join("+");
}

const SYMBOL_KEYS: Record<string, string> = {
  Enter: "↵",
  Escape: "Esc",
  Backspace: "⌫",
  Delete: "⌦",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Tab: "⇥",
  " ": "Space",
};
