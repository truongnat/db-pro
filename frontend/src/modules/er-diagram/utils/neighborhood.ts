import type { IntrospectResult } from "@/modules/schema/types/schema.types";

export type AdjacencyMap = Map<string, Set<string>>;

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
