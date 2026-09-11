import type { QuickOpenItem } from "@/commons/types/quick-open.types";

export interface QuickOpenRankContext {
  query: string;
  activeTabId: string | null;
  activeConnectionId: string | null;
  explorerConnectionId: string | null;
  openResourceKeys: Set<string>;
  recentResourceKeys: Set<string>;
}

export interface RankedQuickOpenItem {
  item: QuickOpenItem;
  score: number;
  /** Indices of matched characters — may refer to searchText when match fell through. */
  matchIndices: number[];
  /** Indices within the primary title text only — empty when match was in searchText, not title. */
  titleMatchIndices: number[];
}

function primaryText(item: QuickOpenItem): string {
  switch (item.kind) {
    case "tab":
      return item.title;
    case "db-object":
      return item.objectName;
    case "schema":
      return item.schema;
    case "connection":
      return item.connectionName;
  }
}

function searchTextOf(item: QuickOpenItem): string {
  return item.searchText;
}

/**
 * Fuzzy match: finds query characters sequentially in text.
 * Returns matched indices (for highlighting) and a score.
 * Returns { indices: [], score: 0 } if not all query chars are found.
 */
export function fuzzyMatch(text: string, query: string): { indices: number[]; score: number } {
  const q = query.trim().toLowerCase();
  if (!q) return { indices: [], score: 0 };
  const t = text.toLowerCase();

  // Exact match — highest score
  if (t === q) {
    return { indices: Array.from({ length: q.length }, (_, i) => i), score: 1000 };
  }

  // Prefix match
  if (t.startsWith(q)) {
    return { indices: Array.from({ length: q.length }, (_, i) => i), score: 900 };
  }

  // Token-level match (space-separated tokens)
  const tokens = t.split(/\s+/);
  const exactToken = tokens.indexOf(q);
  if (exactToken >= 0) {
    const offset = tokens.slice(0, exactToken).join(" ").length + 1;
    return {
      indices: Array.from({ length: q.length }, (_, i) => offset + i),
      score: 800,
    };
  }

  // Token prefix match
  const prefixToken = tokens.find((token) => token.startsWith(q));
  if (prefixToken) {
    const offset = t.indexOf(prefixToken);
    return {
      indices: Array.from({ length: q.length }, (_, i) => offset + i),
      score: 700,
    };
  }

  // Fuzzy sequential match: each query char must appear in order
  let qi = 0;
  const indices: number[] = [];
  let consecutive = 0;
  let maxConsecutive = 0;
  let lastMatchIdx = -2;

  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      indices.push(ti);
      if (ti === lastMatchIdx + 1) {
        consecutive++;
        maxConsecutive = Math.max(maxConsecutive, consecutive);
      } else {
        consecutive = 1;
      }
      lastMatchIdx = ti;
      qi++;
    }
  }

  // All query chars must be found
  if (qi < q.length) return { indices: [], score: 0 };

  // Score: base 500, bonuses for consecutive matches, early positions, word boundaries
  let score = 500;

  // Consecutive bonus (up to +150)
  score += maxConsecutive * 15;

  // Early position bonus (up to +100)
  if (indices.length > 0) {
    score += Math.max(0, 100 - indices[0] * 5);
  }

  // Word boundary bonus: matches at start of word (after separator or uppercase)
  for (const idx of indices) {
    if (idx === 0 || text[idx - 1] === "." || text[idx - 1] === "_" || text[idx - 1] === " ") {
      score += 20;
    }
  }

  // Compactness bonus: fewer gaps between first and last match
  if (indices.length > 1) {
    const span = indices[indices.length - 1] - indices[0] + 1;
    const density = indices.length / span;
    score += Math.round(density * 80);
  }

  return { indices, score };
}

/** Backward-compatible substring match (used for searchText fallback). */
export function matchScore(text: string, query: string): number {
  const result = fuzzyMatch(text, query);
  return result.score;
}

function boostScore(item: QuickOpenItem, ctx: QuickOpenRankContext): number {
  let boost = 0;

  if (item.kind === "tab") {
    if (item.tabId === ctx.activeTabId) boost += 100;
    if (ctx.openResourceKeys.has(item.resourceKey)) boost += 50;
  }

  if (item.kind === "db-object") {
    if (ctx.openResourceKeys.has(item.resourceKey)) boost += 60;
    if (ctx.recentResourceKeys.has(item.resourceKey)) boost += 40;
    if (item.connectionId === ctx.explorerConnectionId) boost += 30;
    if (item.connectionId === ctx.activeConnectionId) boost += 25;
  }

  if (item.kind === "schema") {
    if (item.connectionId === ctx.explorerConnectionId) boost += 30;
    if (item.connectionId === ctx.activeConnectionId) boost += 25;
  }

  if (item.kind === "connection") {
    if (item.connectionId === ctx.activeConnectionId) boost += 30;
    if (item.connectionId === ctx.explorerConnectionId) boost += 25;
    if (ctx.recentResourceKeys.has(item.resourceKey)) boost += 20;
  }

  return boost;
}

export function rankQuickOpenItems(
  items: QuickOpenItem[],
  ctx: QuickOpenRankContext,
): RankedQuickOpenItem[] {
  const q = ctx.query.trim().toLowerCase();

  const ranked: RankedQuickOpenItem[] = [];
  for (const item of items) {
    if (!q) {
      const base =
        item.kind === "tab"
          ? 400
          : item.kind === "db-object"
            ? 300
            : item.kind === "schema"
              ? 200
              : 100;
      ranked.push({
        item,
        score: base + boostScore(item, ctx),
        matchIndices: [],
        titleMatchIndices: [],
      });
      continue;
    }

    // Try primary text first (better highlight), then full searchText
    const primaryResult = fuzzyMatch(primaryText(item), q);
    const fullResult =
      primaryResult.score > 0 ? { indices: [], score: 0 } : fuzzyMatch(searchTextOf(item), q);

    const bestScore = Math.max(primaryResult.score, fullResult.score);
    if (bestScore === 0) continue;

    const matchedTitle = primaryResult.score >= fullResult.score;
    const matchIndices = matchedTitle ? primaryResult.indices : fullResult.indices;

    ranked.push({
      item,
      score: bestScore + boostScore(item, ctx),
      matchIndices,
      titleMatchIndices: matchedTitle ? primaryResult.indices : [],
    });
  }

  return ranked.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    return primaryText(a.item).localeCompare(primaryText(b.item));
  });
}
