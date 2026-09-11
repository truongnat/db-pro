/**
 * P1.3 — Level-of-detail resolution for ER diagram nodes.
 *
 * The locked architecture mandates 4 render-tree levels:
 *   dot     → a single dot (schema overview)
 *   compact → table name only
 *   summary → table name + "N cols · M FK"
 *   detail  → full column list
 *
 * Each level is a separate leaf component (ErDotNode / ErCompactNode /
 * ErSummaryNode / ErDetailedNode); the dispatcher `ErTableNode` switches the
 * render tree so hidden DOM is truly unmounted, never CSS-hidden.
 */

export type LodLevel = "dot" | "compact" | "summary" | "detail";

/**
 * Zoom thresholds separating the LOD bands. Detail keeps the legacy >0.7
 * boundary so there is no visual regression at high zoom; dot/compact/summary
 * are hypotheses to be tuned with real schemas (see PLAN P1.3).
 */
export const LOD_THRESHOLDS = {
  dot: 0.2,
  compact: 0.45,
  summary: 0.7,
} as const;

export const LOD_ORDER: LodLevel[] = ["dot", "compact", "summary", "detail"];

/**
 * Resolve the LOD level for a given zoom. The manual `compact` toggle caps the
 * level at `summary` (name + counts, no columns) — the legacy "compact mode"
 * behavior, mapped onto the 4-level taxonomy.
 */
export function resolveLod(zoom: number, compact = false): LodLevel {
  let lod: LodLevel;
  if (zoom < LOD_THRESHOLDS.dot) lod = "dot";
  else if (zoom < LOD_THRESHOLDS.compact) lod = "compact";
  else if (zoom < LOD_THRESHOLDS.summary) lod = "summary";
  else lod = "detail";

  if (compact && lod === "detail") return "summary";
  return lod;
}

/** Numeric tier for the `data-tier` DOM attribute (instrumentation counts). */
export function lodTier(lod: LodLevel): 0 | 1 | 2 | 3 {
  switch (lod) {
    case "dot":
      return 0;
    case "compact":
      return 1;
    case "summary":
      return 2;
    case "detail":
      return 3;
  }
}
