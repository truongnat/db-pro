# Findings — UI07 Table/Object/Schema Workbench Quality

1. Schema workbench and object workbench had remaining usages of legacy button helpers (`secondary_button`, `compact_button_with_icon`, `compact_icon_button`, `danger_button`, `ghost_button_with_icon`).
2. Standardizing to canonical `Button` struct with explicit `variant`, `size`, `icon`, and `tooltip` makes the visual layout consistent with the rest of the application design system.
