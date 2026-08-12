import type { IntrospectResult } from "@/modules/schema/types/schema.types";

export type AdjacencyMap = Map<string, Set<string>>;

/** Hop scopes offered by the exploration UI (P1.6). */
export type NeighborhoodScope = 1 | 2 | 3 | "domain";

export function buildAdjacencyMap(foreignKeys: IntrospectResult["foreignKeys"]): AdjacencyMap {
  const adj: AdjacencyMap = new Map();

  const ensure = (key: string) => {
    if (!adj.has(key)) adj.set(key, new Set());
  };

  for (const fk of foreignKeys) {
    const fromKey = `${fk.schema}.${fk.fromTable}`;
    const toKey = `${fk.toSchema}.${fk.toTable}`;
    ensure(fromKey);
    ensure(toKey);
    adj.get(fromKey)!.add(toKey);
    adj.get(toKey)!.add(fromKey);
  }

  return adj;
}

export function getNeighborhood(adjacency: AdjacencyMap, seed: string, hops: number): Set<string> {
  const visited = new Set<string>();
  let frontier = new Set<string>([seed]);

  for (let i = 0; i < hops; i++) {
    const next = new Set<string>();
    for (const node of frontier) {
      if (visited.has(node)) continue;
      visited.add(node);
      const neighbors = adjacency.get(node);
      if (neighbors) {
        for (const n of neighbors) {
          if (!visited.has(n)) next.add(n);
        }
      }
    }
    frontier = next;
  }

  for (const node of frontier) {
    visited.add(node);
  }

  return visited;
}

/**
 * The connected component containing `seed` — the "Domain" explore scope.
 * Breadth-first until the frontier is exhausted, O(V + E) over the component.
 */
export function getConnectedComponent(adjacency: AdjacencyMap, seed: string): Set<string> {
  const visited = new Set<string>();
  const stack = [seed];

  while (stack.length > 0) {
    const node = stack.pop()!;
    if (visited.has(node)) continue;
    visited.add(node);
    const neighbors = adjacency.get(node);
    if (neighbors) {
      for (const n of neighbors) {
        if (!visited.has(n)) stack.push(n);
      }
    }
  }

  return visited;
}

/**
 * Hub tables for the "Suggested starting points" list — the `count` tables in
 * `tablesInSchema` with the highest FK degree (most relations), ties broken
 * alphabetically. Degree is computed from the bidirectional adjacency map, so
 * both incoming and outgoing relations count. O(T log T) over schema tables.
 */
export function suggestStartingPoints(
  adjacency: AdjacencyMap,
  tablesInSchema: { name: string; schema: string }[],
  count = 5,
): string[] {
  return tablesInSchema
    .map((t) => {
      const key = `${t.schema}.${t.name}`;
      return { key, degree: adjacency.get(key)?.size ?? 0 };
    })
    .sort((a, b) => b.degree - a.degree || a.key.localeCompare(b.key))
    .slice(0, count)
    .map((s) => s.key);
}
