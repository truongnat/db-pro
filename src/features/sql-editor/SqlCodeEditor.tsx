import { pickedCompletion } from "@codemirror/autocomplete";
import CodeMirror from "@uiw/react-codemirror";
import type { Extension } from "@codemirror/state";

import { recordCompletionUsage } from "./completion-recency";
import type { SqlEditorSelection } from "./types";

type SqlCodeEditorProps = {
  value: string;
  extensions: Extension[];
  onValueChange: (nextValue: string) => void;
  onSelectionChange: (selection: SqlEditorSelection) => void;
};

export function SqlCodeEditor({
  value,
  extensions,
  onValueChange,
  onSelectionChange,
}: SqlCodeEditorProps) {
  return (
    <CodeMirror
      value={value}
      height="100%"
      minHeight="240px"
      basicSetup={{
        lineNumbers: true,
        foldGutter: false,
        highlightActiveLine: true,
        highlightActiveLineGutter: true,
        searchKeymap: true,
      }}
      extensions={extensions}
      onUpdate={(update) => {
        for (const transaction of update.transactions) {
          const completion = transaction.annotation(pickedCompletion);
          if (completion) {
            recordCompletionUsage(completion.label, completion.type);
          }
        }

        if (update.docChanged) {
          onValueChange(update.state.doc.toString());
        }

        const main = update.state.selection.main;
        onSelectionChange({
          anchor: main.anchor,
          head: main.head,
        });
      }}
    />
  );
}
