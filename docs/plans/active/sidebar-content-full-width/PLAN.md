# Sidebar content full width

## Problem

Navigator tree content did not fill the left sidebar the way VS Code / DBeaver do.
Screenshot evidence: tree labels / `Failed:` hints clipped around ~200px while
`New query` / filter chrome reached ~360px and the drag separator sat near ~445px.

## Cause

1. `ScrollArea` content width settled on intrinsic (short-label) size, so tree rows
   truncated mid-panel.
2. Resize handle used egui `SidePanel` frame `response.rect`, which could disagree
   with the `exact_width(sidebar_width)` we requested — separator floated past content.

## Fix

- Derive sidebar content rect from clamped `sidebar_width` (asymmetric pad: left
  `SPACE_SM`, right `SPACE_XXS`).
- Lock drag separator to `panel_left + sidebar_width`.
- Force explorer `ScrollArea` + tree rows to the padded content width every frame.

## Scope

`crates/ui` only: `sidebar_view.rs`, `explorer_view.rs`, `explorer_tree.rs`.
