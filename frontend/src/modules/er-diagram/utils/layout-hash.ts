import type { LayoutInput, LayoutOptions } from "./layout";

/**
 * Deterministic content hash of a layout request — the `schemaHash → positions`
 * cache key from the locked P1 architecture.
 *
 * The hash covers everything dagre's output depends on: node ids + sizes
 * (height drives rank separation), edge topology, and layout options. It is
 * order-independent (nodes/edges sorted) so equal graphs always hash equal.
 * Two 32-bit FNV-1a passes → 64-bit hex; collisions are possible in theory but
 * the cache additionally verifies the exact node-id set on read.
 */
export function computeLayoutHash(input: LayoutInput, options: LayoutOptions): string {
  const nodes = [...input.nodes].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  const edges = [...input.edges].sort((a, b) => {
    const ka = a.source + "\u0000" + a.target;
    const kb = b.source + "\u0000" + b.target;
    return ka < kb ? -1 : ka > kb ? 1 : 0;
  });

  const parts: string[] = [];
  for (const n of nodes) parts.push(n.id + ":" + n.height + ":" + (n.width ?? 0));
  for (const e of edges) parts.push(e.source + ">" + e.target);
  // P1-2: the profile id is part of the key, so overview (compact) and React
  // Flow (column-aware) layouts for the same graph never share cache entries.
  const opts = `profile=${options.profile ?? "react-flow"};dir=${options.direction ?? "LR"};ns=${options.nodeSep ?? 60};rs=${options.rankSep ?? 100}`;

  return fnv1aHex(opts + "|" + parts.join(","));
}

function fnv1a(str: string, seed: number): number {
  let h = seed >>> 0;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

function fnv1aHex(body: string): string {
  const h1 = fnv1a(body, 0x811c9dc5);
  const h2 = fnv1a(body, 0x01000193);
  return h1.toString(16).padStart(8, "0") + h2.toString(16).padStart(8, "0");
}
