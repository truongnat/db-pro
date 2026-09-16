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
