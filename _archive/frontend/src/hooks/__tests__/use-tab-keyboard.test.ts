import { describe, expect, it } from "vitest";

import { getTabNavigationOrder } from "../use-tab-keyboard";

describe("getTabNavigationOrder", () => {
  it("matches the pinned-first tab bar order", () => {
    const tabs = [
      { id: "query-1", pinned: false },
      { id: "query-2", pinned: true },
      { id: "query-3", pinned: false },
      { id: "query-4", pinned: true },
    ];

    expect(getTabNavigationOrder(tabs).map((tab) => tab.id)).toEqual([
      "query-2",
      "query-4",
      "query-1",
      "query-3",
    ]);
  });
});
