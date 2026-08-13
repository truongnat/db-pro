import { describe, it, expect } from "vitest";
import {
  buildAdjacencyMap,
  getConnectedComponent,
  getNeighborhood,
  suggestStartingPoints,
} from "../utils/neighborhood";
import { generateErFixture } from "./er-fixture";

/** Reference BFS distances — independent of `getNeighborhood` internals. */
function distancesFrom(
  adj: Map<string, Set<string>>,
  seed: string,
  maxHops: number,
): Map<string, number> {
  const dist = new Map<string, number>([[seed, 0]]);
  const queue = [seed];
  while (queue.length > 0) {
    const node = queue.shift()!;
    const d = dist.get(node)!;
    if (d >= maxHops) continue;
    for (const n of adj.get(node) ?? []) {
      if (!dist.has(n)) {
        dist.set(n, d + 1);
        queue.push(n);
      }
    }
  }
  return dist;
}

const fk = (from: string, to: string, name = "fk") => ({
  name,
  fromTable: from,
  fromColumns: ["id"],
  toTable: to,
  toColumns: ["id"],
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

describe("getNeighborhood — complexity is O(hops × degree), not O(all tables) (P3.6)", () => {
  it("1-hop from an isolated table in a 500-table schema returns only the seed", () => {
    const data = generateErFixture(500);
    // A table with no FK edges has no adjacency entry; the computation must
    // touch only the seed, never scan the other 499 tables.
    const adj = buildAdjacencyMap(data.foreignKeys);
    expect(getNeighborhood(adj, "public.isolated_tbl", 3)).toEqual(
      new Set(["public.isolated_tbl"]),
    );
  });

  it("2-hop result on a 1000-table graph equals the exact reachable set (parity)", () => {
    const data = generateErFixture(1000);
    const adj = buildAdjacencyMap(data.foreignKeys);

    // Largest-degree hub is the hardest case (widest frontier).
    let seed = "";
    let maxDegree = -1;
    for (const [key, neighbors] of adj) {
      if (neighbors.size > maxDegree) {
        maxDegree = neighbors.size;
        seed = key;
      }
    }

    const hood = getNeighborhood(adj, seed, 2);
    const expected = new Set(distancesFrom(adj, seed, 2).keys());
    expect(hood).toEqual(expected);
    // Bounded, not O(all tables): 2 hops over a sparse graph reaches a small
    // fraction of the tables. Ratio bound (not an absolute) so the assertion
    // stays robust to fixture-generator changes; parity above is the hard pin.
    expect(hood.size).toBeLessThan(data.tables.length / 4);
  });

  it("computes 2-hop on a 1000-table graph well under budget", () => {
    const data = generateErFixture(1000);
    const adj = buildAdjacencyMap(data.foreignKeys);
    const seed = "public.app";

    const times: number[] = [];
    for (let i = 0; i < 20; i++) {
      const start = performance.now();
      getNeighborhood(adj, seed, 2);
      times.push(performance.now() - start);
    }
    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    expect(avg).toBeLessThan(5);
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
