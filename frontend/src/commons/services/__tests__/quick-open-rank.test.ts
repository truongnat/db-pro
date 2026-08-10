import { describe, expect, it } from "vitest";

import {
  fuzzyMatch,
  matchScore,
  rankQuickOpenItems,
  type QuickOpenRankContext,
} from "@/commons/services/quick-open-rank";
import type { QuickOpenItem } from "@/commons/types/quick-open.types";

function dbObj(overrides: Partial<QuickOpenItem & { connectionId: string }> = {}): QuickOpenItem {
  return {
    kind: "db-object",
    connectionId: "conn-1",
    connectionName: "Local",
    schema: "public",
    objectName: "client",
    objectType: "table",
    resourceKey: "dbobj:public.client:conn-1",
    searchText: "client public.client public Local",
    ...overrides,
  } as QuickOpenItem;
}

function ctx(overrides: Partial<QuickOpenRankContext> = {}): QuickOpenRankContext {
  return {
    query: "",
    activeTabId: null,
    activeConnectionId: null,
    explorerConnectionId: null,
    openResourceKeys: new Set(),
    recentResourceKeys: new Set(),
    ...overrides,
  };
}

describe("fuzzyMatch", () => {
  it("returns empty for empty query", () => {
    const result = fuzzyMatch("client", "");
    expect(result.score).toBe(0);
    expect(result.indices).toEqual([]);
  });

  it("exact match scores highest", () => {
    const result = fuzzyMatch("client", "client");
    expect(result.score).toBe(1000);
    expect(result.indices).toEqual([0, 1, 2, 3, 4, 5]);
  });

  it("prefix match scores second highest", () => {
    const result = fuzzyMatch("client_attributes", "client");
    expect(result.score).toBe(900);
    expect(result.indices).toEqual([0, 1, 2, 3, 4, 5]);
  });

  it("fuzzy sequential match finds characters in order", () => {
    const result = fuzzyMatch("client", "clt");
    expect(result.score).toBeGreaterThan(0);
    expect(result.indices).toEqual([0, 1, 5]); // c-l-i-e-n-t → c(0) l(1) t(5)
  });

  it("returns 0 when not all query chars found", () => {
    const result = fuzzyMatch("client", "xyz");
    expect(result.score).toBe(0);
    expect(result.indices).toEqual([]);
  });

  it("is case insensitive", () => {
    const result = fuzzyMatch("CLIENT", "client");
    expect(result.score).toBe(1000);
  });

  it("word boundary match gets bonus", () => {
    const underscore = fuzzyMatch("my_client", "client");
    const midString = fuzzyMatch("xclient", "client");
    // Word boundary (after _) should score higher than mid-string
    expect(underscore.score).toBeGreaterThan(midString.score);
  });

  it("consecutive chars get bonus over scattered", () => {
    const consecutive = fuzzyMatch("abcdef", "abc");
    const scattered = fuzzyMatch("axbxcdef", "abc");
    expect(consecutive.score).toBeGreaterThan(scattered.score);
  });

  it("compactness bonus: tighter matches score higher", () => {
    const tight = fuzzyMatch("ab__cd", "abcd");
    const loose = fuzzyMatch("a____b__c_d", "abcd");
    expect(tight.score).toBeGreaterThan(loose.score);
  });
});

describe("rankQuickOpenItems — matchIndices", () => {
  it("returns matchIndices for highlighted rendering", () => {
    const items: QuickOpenItem[] = [
      dbObj({ objectName: "client", resourceKey: "a", searchText: "client public.client" }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "cli" }));
    expect(ranked).toHaveLength(1);
    expect(ranked[0].matchIndices).toEqual([0, 1, 2]); // c-l-i
    expect(ranked[0].titleMatchIndices).toEqual([0, 1, 2]);
  });

  it("returns empty matchIndices when query is empty", () => {
    const items: QuickOpenItem[] = [dbObj()];
    const ranked = rankQuickOpenItems(items, ctx({ query: "" }));
    expect(ranked[0].matchIndices).toEqual([]);
    expect(ranked[0].titleMatchIndices).toEqual([]);
  });

  it("titleMatchIndices is empty when match falls through to searchText", () => {
    const items: QuickOpenItem[] = [
      dbObj({ objectName: "orders", resourceKey: "a", searchText: "orders public.orders Local" }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "public" }));
    expect(ranked).toHaveLength(1);
    expect(ranked[0].matchIndices.length).toBeGreaterThan(0);
    expect(ranked[0].titleMatchIndices).toEqual([]);
  });
});

describe("matchScore", () => {
  it("exact match ranks highest", () => {
    expect(matchScore("client", "client")).toBeGreaterThan(
      matchScore("client_attributes", "client"),
    );
  });

  it("prefix ranks above qualified", () => {
    const prefix = matchScore("client public.client", "client");
    const qualified = matchScore("my_client public.client", "client");
    expect(prefix).toBeGreaterThan(qualified);
  });

  it("qualified match ranks above plain substring", () => {
    const qualified = matchScore("my_client public.client", "public.client");
    const substring = matchScore("my_client_archive", "client");
    expect(qualified).toBeGreaterThan(substring);
  });

  it("substring still matches", () => {
    expect(matchScore("my_client_archive", "client")).toBeGreaterThan(0);
  });

  it("returns 0 when no match", () => {
    expect(matchScore("client", "orders")).toBe(0);
  });

  it("is case insensitive", () => {
    expect(matchScore("CLIENT", "client")).toBeGreaterThan(0);
  });
});

describe("rankQuickOpenItems", () => {
  it("ranks exact prefix above substring", () => {
    const items: QuickOpenItem[] = [
      dbObj({
        objectName: "my_client_archive",
        resourceKey: "a",
        searchText: "my_client_archive public.my_client_archive public Local",
      }),
      dbObj({
        objectName: "client",
        resourceKey: "b",
        searchText: "client public.client public Local",
      }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "client" }));
    expect(ranked[0].item.resourceKey).toBe("b");
  });

  it("boosts open resources above catalog objects with equal match", () => {
    const openKey = "dbobj:public.client:conn-1";
    const items: QuickOpenItem[] = [
      dbObj({
        objectName: "orders",
        resourceKey: "dbobj:public.orders:conn-1",
        searchText: "orders public.orders public Local",
      }),
      dbObj({ resourceKey: openKey }),
    ];
    const ranked = rankQuickOpenItems(
      items,
      ctx({ query: "client", openResourceKeys: new Set([openKey]) }),
    );
    expect(ranked[0].item.resourceKey).toBe(openKey);
  });

  it("boosts recent resources", () => {
    const recentKey = "dbobj:public.client:conn-1";
    const items: QuickOpenItem[] = [
      dbObj({
        objectName: "orders",
        resourceKey: "dbobj:public.orders:conn-1",
        searchText: "orders public.orders public Local",
      }),
      dbObj({ resourceKey: recentKey }),
    ];
    const ranked = rankQuickOpenItems(
      items,
      ctx({ query: "client", recentResourceKeys: new Set([recentKey]) }),
    );
    expect(ranked[0].item.resourceKey).toBe(recentKey);
  });

  it("boosts explorer connection without hiding others", () => {
    const items: QuickOpenItem[] = [
      dbObj({
        connectionId: "conn-2",
        connectionName: "Production",
        objectName: "users",
        searchText: "users auth.users auth Production",
      }),
      dbObj({
        connectionId: "conn-1",
        connectionName: "Local",
        objectName: "users",
        searchText: "users public.users public Local",
      }),
    ];
    const ranked = rankQuickOpenItems(
      items,
      ctx({ query: "users", explorerConnectionId: "conn-2" }),
    );
    expect(ranked[0].item.connectionId).toBe("conn-2");
    expect(ranked.some((r) => r.item.connectionId === "conn-1")).toBe(true);
  });

  it("boosts active tab connection independently of explorer connection", () => {
    const items: QuickOpenItem[] = [
      dbObj({
        connectionId: "conn-1",
        connectionName: "Local",
        objectName: "users",
        searchText: "users public.users public Local",
      }),
      dbObj({
        connectionId: "conn-2",
        connectionName: "Production",
        objectName: "users",
        searchText: "users auth.users auth Production",
      }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "users", activeConnectionId: "conn-2" }));
    expect(ranked[0].item.connectionId).toBe("conn-2");
  });

  it("treats active and explorer connection boosts as separate axes", () => {
    const items: QuickOpenItem[] = [
      dbObj({
        connectionId: "conn-1",
        connectionName: "Local",
        objectName: "users",
        searchText: "users public.users public Local",
      }),
      dbObj({
        connectionId: "conn-2",
        connectionName: "Production",
        objectName: "users",
        searchText: "users auth.users auth Production",
      }),
    ];
    const ranked = rankQuickOpenItems(
      items,
      ctx({ query: "users", activeConnectionId: "conn-2", explorerConnectionId: "conn-1" }),
    );
    expect(ranked[0].item.connectionId).toBe("conn-1");
    expect(ranked[1].item.connectionId).toBe("conn-2");
  });

  it("open tab ranks at top for empty query", () => {
    const items: QuickOpenItem[] = [
      dbObj(),
      {
        kind: "tab",
        tabId: "t1",
        title: "client",
        connectionId: "conn-1",
        connectionName: "Local",
        resourceKey: "dbobj:public.client:conn-1",
        searchText: "client Local",
      },
    ];
    const ranked = rankQuickOpenItems(items, ctx());
    expect(ranked[0].item.kind).toBe("tab");
  });

  it("drops non-matching items", () => {
    const items: QuickOpenItem[] = [
      dbObj({ objectName: "orders", searchText: "orders public.orders" }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "client" }));
    expect(ranked).toHaveLength(0);
  });

  it("sorts ties by name", () => {
    const items: QuickOpenItem[] = [
      dbObj({ objectName: "zeta", searchText: "zeta public.zeta" }),
      dbObj({ objectName: "alpha", searchText: "alpha public.alpha" }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "a" }));
    expect(ranked[0].item.objectName).toBe("alpha");
  });
});
