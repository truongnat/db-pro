# Findings — UI09 Files/IDE Workspace Polish

1. `files_activity_view.rs` used numerous legacy button helpers (`compact_button`, `compact_icon_button`, `secondary_button_with_icon`, `ghost_button_with_icon`, `danger_button`).
2. Standardizing all of these to `Button` with consistent variants (`Default`, `Secondary`, `Ghost`, `Destructive`), sizes (`Sm`, `IconSm`), and tooltips creates visual harmony across the entire IDE lane.
