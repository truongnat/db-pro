/**
 * Overview node paint geometry — SINGLE SOURCE OF TRUTH (P1-2).
 *
 * Deliberately dependency-free: the canvas renderer (CytoscapeErRenderer)
 * imports these without pulling in `layout.ts` (and with it dagre) into the
 * lazy overview chunk. The layout profile (layout-profile.ts) re-exports them,
 * so the renderer and the dagre layout engine can never drift apart — the
 * P1-2 failure class (renderer paints 160×28 while dagre lays out 220×640)
 * cannot silently recur.
 */
export const OVERVIEW_NODE_WIDTH = 160;
export const OVERVIEW_NODE_HEIGHT = 28;
