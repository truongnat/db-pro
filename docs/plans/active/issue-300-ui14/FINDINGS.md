# Findings

## Initial assessment

- The issue is a cross-surface native UI hardening wave, not a single isolated defect.
- `crates/ui/src/query_view.rs` renders a file path and query header controls in one horizontal row;
  long paths/titles can consume width before the run/builder/overflow actions.
- `crates/ui/src/sidebar_view.rs` already has explicit width bounds and a custom resize handle; the
  workspace selector now has an explicit truncation/tooltip contract.
- `crates/ui/src/workspace_view.rs` uses a horizontal scroll area for tabs; tab title sizing now has a
  bounded measured truncation and tooltip contract.
- Shared `Dialog`/`Sheet` overlays previously accepted fixed widths larger than a narrow viewport; both
  now clamp to available screen width and reserve the close action before truncating titles.
- Shared `Select` popups previously used a minimum width and unbounded parent-relative x position; they
  now clamp to the viewport and truncate option labels.

## Priority

1. Query header overflow is the first implementation slice because it directly risks hiding primary
   actions and is covered by existing query UI test infrastructure.
2. Sidebar long-name behavior is next.
3. Runtime screenshot evidence remains mandatory; source inspection alone cannot close this issue.

## Provider impact

n/a — this is native UI layout work. Database providers are not changed by the first slice.

## Slice 2–4 continuation (this session)

- `crates/ui/src/query_view.rs`: the header connection/schema selectors used natural-width combo
  buttons, so a 60+ character connection name expanded the row and pushed Run/Builder/overflow off
  the right edge at 1280x800. Both combos now use fixed widths (`HEADER_CONN_COMBO_WIDTH` 170,
  `HEADER_SCHEMA_COMBO_WIDTH` 130) with char-elided labels (`elide_chars`) and full-name hover
  tooltips. Fixed widths give a deterministic row budget; tooltips preserve the full name.
- `crates/ui/src/components/dialog.rs`: dialog cards grew unbounded vertically, so long validation
  errors or large-font translated content pushed the footer actions below the viewport. The body is
  now rendered inside a vertical `ScrollArea` capped at `screen.height() - 120`, so the card hugs
  short content and scrolls long content while the footer actions stay inside the card.
- `crates/ui/src/sidebar_view.rs` (slice 2): no code change needed — the header actions already
  render in a reserved right-aligned slot, and the resize grip plus restored sessions both clamp to
  `SIDEBAR_MIN_WIDTH`/`SIDEBAR_MAX_WIDTH` (`crates/ui/src/app_types.rs:236-237`).
- Grid/output scroll ownership (slice 4): verified by inspection — output panes use bounded
  `ScrollArea::max_height` regions (`crates/ui/src/query_output_view.rs:387,427,436`) and the result
  grid virtualizes its own scrolling, so there is no nested scroll trap. Cell-content wrap/truncate
  policy and ER dense labels remain for the next slice.
