import type { Completion } from "@codemirror/autocomplete";

import { getCompletionRecencyBoost } from "./completion-recency";

const MAX_RENDERED_OPTIONS = 80;
const OBJECT_CONTEXT_PATTERN = /\b(from|join|update|into|table|view|describe|desc)\s+[\w$".]*$/i;
const COLUMN_CONTEXT_PATTERN =
  /\b(select|where|and|or|on|set|having|group\s+by|order\s+by)\s+[\w$".]*$/i;

export type CompletionIntent = "keyword" | "object" | "column" | "general";

export function detectCompletionIntent(
  textBeforeCursor: string,
  hasScopedSuggestions: boolean,
): CompletionIntent {
  if (hasScopedSuggestions) {
    return "column";
  }

  if (OBJECT_CONTEXT_PATTERN.test(textBeforeCursor)) {
    return "object";
  }

  if (COLUMN_CONTEXT_PATTERN.test(textBeforeCursor)) {
    return "column";
  }

  if (/^\s*[\w"]*$/.test(textBeforeCursor)) {
    return "keyword";
  }

  return "general";
}

export function rankCompletions(
  items: Completion[],
  rawPrefix: string,
  intent: CompletionIntent,
): Completion[] {
  const prefix = normalizeSqlIdentifier(rawPrefix);
  const now = Date.now();

  const ranked = items
    .map((item, index) => {
      const label = normalizeSqlIdentifier(item.label ?? "");
      const prefixScore = getPrefixScore(label, prefix);
      if (prefix && prefixScore === null) {
        return null;
      }

      const typeScore = getIntentScore(item.type, intent);
      const boostScore = item.boost ?? 0;
      const recencyScore = getCompletionRecencyBoost(item.label ?? "", item.type, now);
      const totalScore = (prefixScore ?? 28) + typeScore + boostScore + recencyScore;

      return {
        item,
        label,
        index,
        typeScore,
        totalScore,
      };
    })
    .filter((entry): entry is NonNullable<typeof entry> => entry !== null);

  ranked.sort((left, right) => {
    if (right.totalScore !== left.totalScore) {
      return right.totalScore - left.totalScore;
    }
    if (right.typeScore !== left.typeScore) {
      return right.typeScore - left.typeScore;
    }
    const labelOrder = left.label.localeCompare(right.label);
    if (labelOrder !== 0) {
      return labelOrder;
    }
    return left.index - right.index;
  });

  return ranked.slice(0, MAX_RENDERED_OPTIONS).map((entry) => entry.item);
}

function getPrefixScore(label: string, prefix: string): number | null {
  if (!prefix) {
    return 28;
  }

  if (!label) {
    return null;
  }

  if (label === prefix) {
    return 140;
  }

  if (label.startsWith(prefix)) {
    return 110 - Math.min(20, label.length - prefix.length);
  }

  if (label.includes(prefix)) {
    return 52;
  }

  return null;
}

function getIntentScore(
  completionType: string | undefined,
  intent: CompletionIntent,
): number {
  const category = resolveTypeCategory(completionType);

  switch (intent) {
    case "keyword":
      return (
        {
          keyword: 44,
          object: 24,
          column: 18,
          namespace: 14,
          other: 10,
        } as const
      )[category];
    case "object":
      return (
        {
          keyword: 12,
          object: 52,
          column: 22,
          namespace: 36,
          other: 12,
        } as const
      )[category];
    case "column":
      return (
        {
          keyword: 10,
          object: 28,
          column: 54,
          namespace: 18,
          other: 12,
        } as const
      )[category];
    default:
      return (
        {
          keyword: 24,
          object: 32,
          column: 30,
          namespace: 20,
          other: 14,
        } as const
      )[category];
  }
}

function resolveTypeCategory(
  completionType: string | undefined,
): "keyword" | "object" | "column" | "namespace" | "other" {
  const firstType = completionType?.trim().split(/\s+/)[0]?.toLowerCase() ?? "";

  if (firstType === "keyword") {
    return "keyword";
  }
  if (firstType === "property") {
    return "column";
  }
  if (firstType === "namespace") {
    return "namespace";
  }
  if (firstType === "class" || firstType === "interface" || firstType === "type") {
    return "object";
  }
  return "other";
}

function normalizeSqlIdentifier(value: string): string {
  return value.trim().replace(/"/g, "").toLowerCase();
}
