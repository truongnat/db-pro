# Runtime UX Audit v2 — PATCH 5 Baseline

**Date:** 2026-08-08  
**Viewport:** 1440×900 (target)  
**Commit baseline:** `6f4db1b`  
**Commit after:** `035a0ec` (D8 complete)

## Screenshot Inventory

| # | Screen | File | Status |
|---|--------|------|--------|
| 01 | Query idle | `before/01-query-idle.png` | Captured (full-screen) |
| 02 | Query result | `before/02-query-result.png` | Pending manual capture |
| 03 | Explain | `before/03-query-explain.png` | Pending manual capture |
| 04 | Query error | `before/04-query-error.png` | Pending manual capture |
| 05 | DB Object Data | `before/05-db-object-data.png` | Pending manual capture |
| 06 | Explorer expanded | `before/06-explorer-expanded.png` | Pending manual capture |
| 07 | Agent open | `before/07-agent-open.png` | Pending manual capture |
| 08 | Empty workspace | `before/08-empty-workspace.png` | Pending manual capture |

**Note:** Automated GUI screenshot navigation was not feasible in the agent environment. The user-provided screenshot from pre-PATCH 5 review serves as the primary baseline evidence for Query + Explain screens.

## Implementation Summary

| Step | Commit | Description |
|------|--------|-------------|
| D1 | `88ba9d7` | Shell geometry: ActivityBar 46px, Explorer 288px, Topbar 32px, StatusBar 22px |
| D2 | `e1a3e56` | Tab system: kind icons, 110-220px width, surface active state |
| D3 | `ad516dc` | Query workbench: 36px toolbar, editor surface, 65:35 split, 32px result tabs |
| D4+D5 | `6fed33c` | Explain redesign: summary + tree + node details split view |
| D6 | `1041955` | Explorer: 28px search, row density, 14px indent, hierarchy typography |
| D7 | `3a62755` | DB Object: 40px header breadcrumb, 32px section tabs |
| D8 | `035a0ec` | Agent: context chips, starter suggestions, ⌘↵ composer |

## Verification

| Check | Result |
|-------|--------|
| cargo check | clean (0 errors, 0 warnings) |
| cargo test | 217 passed, 0 failed |
| tsc --noEmit | 0 errors |

## Baseline Findings (from user-provided screenshot + spec review)

### P0 — Critical Visual Issues

None. The architecture is sound; issues are visual hierarchy and polish.

### P1 — High-Impact Visual/UX Issues

| ID | Area | Finding | Target |
|----|------|---------|--------|
| V1 | **Visual weight** | All layers have equal visual weight; eye doesn't know where to focus | Editor must dominate; Explorer quiet; Output secondary |
| V2 | **Explorer width** | Object names truncated (`t_workflow_definit...`) | Default 288px, range 240–420px |
| V3 | **Explorer row density** | Rows too cramped (~18–20px) | Connection 30px, schema 28px, object 25–26px |
| V4 | **Tab system** | Too many tabs, titles unclear, active state weak, tabs too narrow | Min 110px, ideal 150px, max 220px; clear icon+title+state |
| V5 | **Query context bar** | Context and actions same visual weight; no hierarchy | Compact context left; Run primary right |
| V6 | **Editor surface** | Looks like "large dark empty box"; no editor feel | Distinct surface, 13.5–14px font, better padding/gutter |
| V7 | **Explain panel** | Raw JSON debug tree; 80% whitespace | Summary + Plan Tree + Node Details |
| V8 | **Result tabs** | 5 peer tabs (Results/Explain/History/Local History/Snippets) | Results/Explain/Messages primary; History/Snippets secondary |
| V9 | **Typography** | Text feels 9–10px throughout; looks "cheap" | Main UI 13px, editor 13.5–14px, secondary ≥11px |
| V10 | **Borders** | Border around every layer; visually noisy | Reduce 40–50%; prefer surface contrast + spacing |
| V11 | **Activity Bar** | Icons too small, inactive too dark, active unclear | 46px width, 18px icons, subtle active bg, 2px indicator |
| V12 | **Status bar** | Too small, all items same weight | 22–24px, left=context, right=diagnostics |
| V13 | **Error UI** | Thin red 18px strip; not structured | Structured panel with message, context, actions |
| V14 | **Color hierarchy** | Too monochrome; no deliberate surface contrast | Layered surfaces: app→nav→workbench→editor→output |

### P2 — Polish Issues

| ID | Area | Finding |
|----|------|---------|
| P2-1 | Hover actions | Always-visible icons; should show on hover only |
| P2-2 | Search input | Too small; should be 180–220px command surface |
| P2-3 | Tree indentation | Too shallow; hierarchy unclear |
| P2-4 | Close behavior | Close icon always visible; should be hover-only |
| P2-5 | Resize handles | Too prominent idle; should be subtle idle, obvious hover/drag |

## Implementation Priority

**D1 — Shell Geometry** (this commit)  
Addresses: V2, V3, V9, V10, V11, V12

Changes:
- ActivityBar: 46px width, 18px icons, subtle active state
- Explorer: default 288px, improved row heights
- Topbar: 32–36px, compact
- StatusBar: 22–24px, quiet
- Typography baseline: 13px main UI
- Reduce border noise

**Golden screen:** Query + Explain at 1440×900.

## Acceptance Criteria

| Check | Pass Condition |
|-------|----------------|
| Explorer | Long object names usable (not truncated at ~12 chars) |
| Typography | No text below 11px; main UI 13px |
| Borders | visibly fewer hard borders between panels |
| Activity Bar | Icons 18px, active state clear |
| Density | Compact but not microscopic |
| 1440×900 | Balanced layout, no clipping |
