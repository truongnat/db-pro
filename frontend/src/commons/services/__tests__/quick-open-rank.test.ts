import { describe, expect, it } from "vitest";

import {
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

describe("matchScore", () => {
  it("exact match ranks highest", () => {
    expect(matchScore("client", "client")).toBeGreaterThan(matchScore("client_attributes", "client"));
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
      dbObj({ objectName: "my_client_archive", resourceKey: "a", searchText: "my_client_archive public.my_client_archive public Local" }),
      dbObj({ objectName: "client", resourceKey: "b", searchText: "client public.client public Local" }),
    ];
    const ranked = rankQuickOpenItems(items, ctx({ query: "client" }));
    expect(ranked[0].item.resourceKey).toBe("b");
  });

  it("boosts open resources above catalog objects with equal match", () => {
    const openKey = "dbobj:public.client:conn-1";
    const items: QuickOpenItem[] = [
      dbObj({ objectName: "orders", resourceKey: "dbobj:public.orders:conn-1", searchText: "orders public.orders public Local" }),
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
      dbObj({ objectName: "orders", resourceKey: "dbobj:public.orders:conn-1", searchText: "orders public.orders public Local" }),
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
      dbObj({ connectionId: "conn-2", connectionName: "Production", objectName: "users", searchText: "users auth.users auth Production" }),
      dbObj({ connectionId: "conn-1", connectionName: "Local", objectName: "users", searchText: "users public.users public Local" }),
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
      dbObj({ connectionId: "conn-1", connectionName: "Local", objectName: "users", searchText: "users public.users public Local" }),
      dbObj({ connectionId: "conn-2", connectionName: "Production", objectName: "users", searchText: "users auth.users auth Production" }),
    ];
    const ranked = rankQuickOpenItems(
      items,
      ctx({ query: "users", activeConnectionId: "conn-2" }),
    );
    expect(ranked[0].item.connectionId).toBe("conn-2");
  });

  it("treats active and explorer connection boosts as separate axes", () => {
    const items: QuickOpenItem[] = [
      dbObj({ connectionId: "conn-1", connectionName: "Local", objectName: "users", searchText: "users public.users public Local" }),
      dbObj({ connectionId: "conn-2", connectionName: "Production", objectName: "users", searchText: "users auth.users auth Production" }),
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
    const items: QuickOpenItem[] = [dbObj({ objectName: "orders", searchText: "orders public.orders" })];
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
