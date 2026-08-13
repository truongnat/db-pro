/**
 * Large-schema state machine for Gate 4 (IT0-104).
 *
 * Invariant table (locked):
 *
 * | State        | Graph nodes | Worker layout | Cytoscape |
 * |--------------|-------------|---------------|-----------|
 * | search       | 0           | 0             | No        |
 * | neighborhood | ≤100        | ≤100          | No        |
 * | overview     | Full        | Full          | Yes       |
 *
 * The phase transitions are explicit user actions — never automatic tier-based.
 */

import type { AdjacencyMap } from "./neighborhood";

/** Hard cap for neighborhood rendering (React Flow safe zone). */
export const NEIGHBORHOOD_NODE_CAP = 100;

/** Phase of the large-schema exploration flow. */
export type LargeSchemaPhase = "search" | "neighborhood" | "overview";

/** State of the large-schema exploration. */
export interface LargeSchemaState {
  phase: LargeSchemaPhase;
  /** Table key (schema.name) selected as neighborhood seed. */
  seedTable: string | null;
  /** Node key (schema.name) with focused LOD detail. */
  focusedNodeId: string | null;
}

/** Actions that transition the large-schema state. */
export type LargeSchemaAction =
  | { type: "SELECT_TABLE"; tableKey: string }
  | { type: "SHOW_ALL" }
  | { type: "BACK_TO_NEIGHBORHOOD" }
  | { type: "BACK_TO_SEARCH" }
  | { type: "FOCUS_NODE"; nodeKey: string }
  | { type: "CLEAR_FOCUS" };

/** Initial state: search phase with no selection. */
export const initialLargeSchemaState: LargeSchemaState = {
  phase: "search",
  seedTable: null,
  focusedNodeId: null,
};

/**
 * Pure reducer for large-schema state transitions.
 *
 * Invariants:
 * - `search` phase: seedTable=null, focusedNodeId=null
 * - `neighborhood` phase: seedTable!=null, focusedNodeId optional
 * - `overview` phase: seedTable!=null (preserved for back-navigation)
 */
export function largeSchemaReducer(
  state: LargeSchemaState,
  action: LargeSchemaAction,
): LargeSchemaState {
  switch (action.type) {
    case "SELECT_TABLE":
      return {
        phase: "neighborhood",
        seedTable: action.tableKey,
        focusedNodeId: action.tableKey,
      };

    case "SHOW_ALL":
      if (state.phase !== "neighborhood") return state;
      return { ...state, phase: "overview" };

    case "BACK_TO_NEIGHBORHOOD":
      if (state.phase !== "overview") return state;
      return { ...state, phase: "neighborhood", focusedNodeId: state.seedTable };

    case "BACK_TO_SEARCH":
      return initialLargeSchemaState;

    case "FOCUS_NODE":
      if (state.phase !== "neighborhood") return state;
      return { ...state, focusedNodeId: action.nodeKey };

    case "CLEAR_FOCUS":
      return { ...state, focusedNodeId: state.seedTable };

    default:
      return state;
  }
}

/**
 * BFS neighborhood with a hard node cap.
 *
 * Returns the set of table keys within `hops` of `seed`, bounded to `maxNodes`.
 * If the cap is reached before exhausting the hop radius, `truncated` is true.
 *
 * The algorithm is deterministic: nodes are visited in BFS order, with
 * alphabetical tiebreaking within each frontier level.
 */
export function getBoundedNeighborhood(
  adjacency: AdjacencyMap,
  seed: string,
  hops: number,
  maxNodes: number = NEIGHBORHOOD_NODE_CAP,
): { nodes: Set<string>; truncated: boolean } {
  if (maxNodes <= 0) {
    return { nodes: new Set(), truncated: true };
  }

  const visited = new Set<string>();
  let frontier = [seed];
  let truncated = false;

  for (let hop = 0; hop < hops && frontier.length > 0; hop++) {
    const next: string[] = [];

    // Sort frontier for deterministic ordering within each level
    const sortedFrontier = [...frontier].sort();

    for (const node of sortedFrontier) {
      if (visited.has(node)) continue;

      if (visited.size >= maxNodes) {
        truncated = true;
        break;
      }

      visited.add(node);
      const neighbors = adjacency.get(node);
      if (neighbors) {
        for (const n of neighbors) {
          if (!visited.has(n)) next.push(n);
        }
      }
    }

    if (truncated) break;

    // Deduplicate and sort next frontier for determinism
    frontier = [...new Set(next)].sort();
  }

  // Add remaining frontier nodes up to cap
  if (!truncated) {
    for (const node of frontier) {
      if (visited.size >= maxNodes) {
        truncated = true;
        break;
      }
      if (!visited.has(node)) {
        visited.add(node);
      }
    }
  }

  return { nodes: visited, truncated };
}

/**
 * Determine whether a schema should enter the large-schema flow.
 *
 * Entry condition (locked): table count > 200 OR tier is L/XL.
 * This ensures sparse 300-table schemas don't bypass the gate if they're
 * somehow classified below L tier.
 */
export function shouldEnterLargeSchemaFlow(
  tableCount: number,
  tier: "XS" | "S" | "M" | "L" | "XL",
): boolean {
  return tableCount > 200 || tier === "L" || tier === "XL";
}
