import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags } from "@lezer/highlight";

export const sqlHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: "#2563eb", fontWeight: "700" },
  { tag: [tags.string, tags.special(tags.string)], color: "#b45309" },
  { tag: [tags.number, tags.bool], color: "#7c3aed" },
  { tag: [tags.comment, tags.lineComment], color: "#64748b", fontStyle: "italic" },
  { tag: [tags.variableName, tags.name], color: "#0f172a" },
  { tag: [tags.typeName, tags.className], color: "#0ea5e9" },
  { tag: tags.operator, color: "#475569" },
  { tag: tags.punctuation, color: "#64748b" },
]);

export const sqlEditorTheme = EditorView.theme({
  "&": {
    height: "100%",
    fontSize: "12px",
  },
  ".cm-scroller": {
    fontFamily:
      '"SFMono-Regular", "JetBrains Mono", "Menlo", "Monaco", "Consolas", monospace',
    lineHeight: "1.7",
  },
  ".cm-gutters": {
    backgroundColor: "hsl(var(--muted) / 0.45)",
    color: "hsl(var(--muted-foreground))",
    borderRight: "1px solid hsl(var(--border))",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "hsl(var(--accent) / 0.45)",
  },
  ".cm-activeLine": {
    backgroundColor: "hsl(var(--accent) / 0.28)",
  },
  ".cm-cursor": {
    borderLeftColor: "hsl(var(--primary))",
  },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection": {
    backgroundColor: "hsl(var(--primary) / 0.25)",
  },
  ".cm-tooltip-autocomplete": {
    borderRadius: "0.75rem",
    border: "1px solid hsl(var(--border))",
    backgroundColor: "hsl(var(--background) / 0.98)",
    boxShadow:
      "0 18px 50px -24px rgba(15,23,42,0.45), 0 10px 24px -18px rgba(15,23,42,0.35)",
    backdropFilter: "blur(10px)",
    overflow: "hidden",
  },
  ".cm-tooltip-autocomplete ul li[aria-selected]": {
    backgroundColor: "hsl(var(--accent))",
    color: "hsl(var(--foreground))",
  },
});

export const sqlEditorStyling = [
  syntaxHighlighting(sqlHighlightStyle),
  sqlEditorTheme,
] as const;
