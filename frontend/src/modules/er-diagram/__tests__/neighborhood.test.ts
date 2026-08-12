import { describe, it, expect } from "vitest";
import {
  buildAdjacencyMap,
  getConnectedComponent,
  getNeighborhood,
  suggestStartingPoints,
} from "../utils/neighborhood";

const fk = (from: string, to: string, name = "fk") => ({
  name,
  fromTable: from,
  fromColumn: "id",
  toTable: to,
  toColumn: "id",
  schema: "public",
  toSchema: "public",
});

describe("buildAdjacencyMap", () => {
  it("builds bidirectional adjacency from FKs", () => {
    const fks = [
      fk("orders", "users", "fk_orders_user"),
      fk("orders", "products", "fk_orders_product"),
    ];
    const adj = buildAdjacencyMap(fks);

    expect(adj.get("public.orders")?.has("public.users")).toBe(true);
    expect(adj.get("public.orders")?.has("public.products")).toBe(true);
    expect(adj.get("public.users")?.has("public.orders")).toBe(true);
    expect(adj.get("public.products")?.has("public.orders")).toBe(true);
  });

  it("handles empty FK list", () => {
    const adj = buildAdjacencyMap([]);
    expect(adj.size).toBe(0);
  });
});

describe("getNeighborhood", () => {
  it("returns seed only at 0 hops", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("b", "c")]);
    const hood = getNeighborhood(adj, "public.a", 0);
    expect(hood).toEqual(new Set(["public.a"]));
  });

  it("returns direct neighbors at 1 hop", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("b", "c"), fk("a", "d")]);
    const hood = getNeighborhood(adj, "public.a", 1);
    expect(hood).toEqual(new Set(["public.a", "public.b", "public.d"]));
  });

  it("returns 2-hop neighborhood", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("b", "c"), fk("c", "d")]);
    const hood = getNeighborhood(adj, "public.a", 2);
    expect(hood).toEqual(new Set(["public.a", "public.b", "public.c"]));
  });

  it("handles isolated node", () => {
    const adj = buildAdjacencyMap([fk("a", "b")]);
    const hood = getNeighborhood(adj, "public.isolated", 1);
    expect(hood).toEqual(new Set(["public.isolated"]));
  });

  it("does not revisit nodes in cyclic graphs", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("b", "a")]);
    const hood = getNeighborhood(adj, "public.a", 3);
    expect(hood).toEqual(new Set(["public.a", "public.b"]));
  });
});

describe("getConnectedComponent", () => {
  it("returns the whole connected component containing the seed", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("b", "c"), fk("c", "a")]);
    expect(getConnectedComponent(adj, "public.a")).toEqual(
      new Set(["public.a", "public.b", "public.c"]),
    );
  });

  it("does not cross component boundaries", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("x", "y")]);
    expect(getConnectedComponent(adj, "public.x")).toEqual(new Set(["public.x", "public.y"]));
  });

  it("handles isolated nodes", () => {
    const adj = buildAdjacencyMap([fk("a", "b")]);
    expect(getConnectedComponent(adj, "public.isolated")).toEqual(new Set(["public.isolated"]));
  });
});

describe("suggestStartingPoints", () => {
  const tables = ["orders", "users", "payments", "items"].map((name) => ({
    name,
    schema: "public",
  }));

  it("ranks tables by FK degree", () => {
    const adj = buildAdjacencyMap([
      fk("orders", "users"),
      fk("orders", "items"),
      fk("orders", "payments"),
      fk("users", "payments"),
    ]);
    // orders has 3 neighbors, users/payments 2 each, items 1.
    expect(suggestStartingPoints(adj, tables, 5)[0]).toBe("public.orders");
  });

  it("returns at most count entries", () => {
    const adj = buildAdjacencyMap([fk("a", "b"), fk("a", "c")]);
    expect(suggestStartingPoints(adj, tables, 2)).toHaveLength(2);
  });

  it("breaks degree ties alphabetically", () => {
    const adj = buildAdjacencyMap([fk("orders", "users"), fk("orders", "items")]);
    // users and items both have degree 1 → alphabetical: items before users.
    const sorted = suggestStartingPoints(adj, tables, 5);
    expect(sorted.indexOf("public.items")).toBeLessThan(sorted.indexOf("public.users"));
  });

  it("returns empty for an empty schema", () => {
    expect(suggestStartingPoints(buildAdjacencyMap([]), [], 5)).toEqual([]);
  });
});
