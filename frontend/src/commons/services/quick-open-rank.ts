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

export function matchScore(text: string, query: string): number {
  const q = query.trim().toLowerCase();
  if (!q) return 0;
  const t = text.toLowerCase();

  if (t === q) return 1000;
  if (t.startsWith(q)) return 900;

  const tokens = t.split(/\s+/);
  if (tokens.includes(q)) return 800;

  const prefixToken = tokens.find((token) => token.startsWith(q));
  if (prefixToken) return 700;

  const qualified = t.includes(".");
  if (qualified && t.includes(q)) return 650;

  const idx = t.indexOf(q);
  if (idx >= 0) return 500 - idx * 0.5;

  return 0;
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
      ranked.push({ item, score: base + boostScore(item, ctx) });
      continue;
    }

    const primary = matchScore(primaryText(item), q);
    if (primary === 0) continue;

    const full = matchScore(searchTextOf(item), q);
    const score = Math.max(primary, full) + boostScore(item, ctx);
    ranked.push({ item, score });
  }

  return ranked.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    return primaryText(a.item).localeCompare(primaryText(b.item));
  });
}
