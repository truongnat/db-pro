# Findings — UI04 Explorer Polish

## Findings

1. `crates/ui/src/explorer_folders.rs`:
   - Embedded hardcoded RGB colors (`Color32::from_rgb(5, 150, 105)` for views, `Color32::from_rgb(124, 58, 237)` for functions, `Color32::from_rgb(234, 88, 12)` for triggers).
   - Replaced with theme-aware semantic token references (`theme.success`, `theme.accent`, `theme.warning`) ensuring proper contrast across light and dark themes.

2. `crates/ui/src/explorer_view.rs`:
   - Connection context menu Delete action uses destructive styling.
