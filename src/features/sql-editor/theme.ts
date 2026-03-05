import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags } from "@lezer/highlight";

export const sqlHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: "#1d4ed8", fontWeight: "700" },
  { tag: [tags.string, tags.special(tags.string)], color: "#a16207" },
  { tag: [tags.number, tags.bool, tags.literal], color: "#7e22ce", fontWeight: "600" },
  { tag: [tags.comment, tags.lineComment], color: "#64748b", fontStyle: "italic" },
  { tag: [tags.variableName, tags.name], color: "#0f172a" },
  { tag: [tags.typeName, tags.className], color: "#0f766e", fontWeight: "600" },
  {
    tag: [tags.function(tags.variableName), tags.function(tags.propertyName)],
    color: "#db2777",
  },
  { tag: tags.definition(tags.variableName), color: "#0f172a", fontWeight: "700" },
  { tag: tags.operator, color: "#334155", fontWeight: "600" },
  { tag: tags.punctuation, color: "#64748b" },
  { tag: tags.invalid, color: "#dc2626", textDecoration: "underline wavy" },
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
  ".cm-tooltip-autocomplete > ul": {
    maxHeight: "18rem",
    padding: "0.25rem 0",
  },
  ".cm-tooltip-autocomplete ul li": {
    padding: "0.45rem 0.65rem",
    borderBottom: "1px solid hsl(var(--border) / 0.35)",
  },
  ".cm-tooltip-autocomplete ul li:last-child": {
    borderBottom: "none",
  },
  ".cm-tooltip-autocomplete ul li .cm-completionIcon": {
    opacity: "0.75",
  },
  ".cm-tooltip-autocomplete ul li .cm-completionLabel": {
    fontWeight: "600",
    letterSpacing: "0.01em",
  },
  ".cm-tooltip-autocomplete ul li .cm-completionMatchedText": {
    color: "hsl(var(--primary))",
    textDecoration: "none",
    fontWeight: "800",
  },
  ".cm-tooltip-autocomplete ul li .cm-completionDetail": {
    color: "hsl(var(--muted-foreground))",
    fontSize: "11px",
    marginLeft: "0.35rem",
  },
  ".cm-tooltip-autocomplete ul li[aria-selected]": {
    backgroundColor: "hsl(var(--accent))",
    color: "hsl(var(--foreground))",
  },
  ".cm-tooltip-autocomplete ul li[aria-selected] .cm-completionMatchedText": {
    color: "hsl(var(--primary))",
  },
});

export const sqlEditorStyling = [
  syntaxHighlighting(sqlHighlightStyle),
  sqlEditorTheme,
] as const;
