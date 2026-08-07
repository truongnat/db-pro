import { describe, expect, it } from "vitest";

import { cycleColumnSort } from "../utils/sort";

describe("cycleColumnSort", () => {
  it("first click sets asc", () => {
    expect(cycleColumnSort([], "id")).toEqual([{ column: "id", direction: "asc" }]);
  });

  it("second click flips to desc", () => {
    expect(
      cycleColumnSort([{ column: "id", direction: "asc" }], "id"),
    ).toEqual([{ column: "id", direction: "desc" }]);
  });

  it("third click clears sort", () => {
    expect(
      cycleColumnSort([{ column: "id", direction: "desc" }], "id"),
    ).toEqual([]);
  });

  it("sorting a new column replaces prior sorts (single-column header behavior)", () => {
    const sorts = [{ column: "age", direction: "desc" }];
    expect(cycleColumnSort(sorts, "id")).toEqual([{ column: "id", direction: "asc" }]);
  });
});
