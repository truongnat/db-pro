import {
  acceptCompletion,
  closeCompletion,
  moveCompletionSelection,
  startCompletion,
} from "@codemirror/autocomplete";
import type { KeyBinding } from "@codemirror/view";

type BuildSqlEditorKeymapInput = {
  busy: boolean;
  onRunQuery: () => void;
  onFormatSql: () => void | Promise<void>;
};

export function buildSqlEditorKeymap({
  busy,
  onRunQuery,
  onFormatSql,
}: BuildSqlEditorKeymapInput): KeyBinding[] {
  return [
    {
      key: "Mod-Enter",
      run: () => {
        if (busy) {
          return true;
        }
        onRunQuery();
        return true;
      },
    },
    {
      key: "Shift-Mod-f",
      run: () => {
        void onFormatSql();
        return true;
      },
    },
    {
      key: "Alt-Shift-f",
      run: () => {
        void onFormatSql();
        return true;
      },
    },
    {
      key: "Mod-Space",
      run: startCompletion,
    },
    {
      key: "ArrowDown",
      run: moveCompletionSelection(true),
    },
    {
      key: "ArrowUp",
      run: moveCompletionSelection(false),
    },
    {
      key: "PageDown",
      run: moveCompletionSelection(true, "page"),
    },
    {
      key: "PageUp",
      run: moveCompletionSelection(false, "page"),
    },
    {
      key: "Enter",
      run: acceptCompletion,
    },
    {
      key: "Tab",
      run: acceptCompletion,
    },
    {
      key: "Escape",
      run: closeCompletion,
    },
  ];
}
