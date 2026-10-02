# Findings

At baseline `6c41cc4fc93eb3e12eb9cada2069cd3514e9b3f2`, the spacing audit records seven inherited P2 issues (see completed/ui-spacing-audit/REPORT.md). No P0/P1.

Decisions: keep a compact 32pt workspace tab, 12pt horizontal inset, 4pt track gap, 8pt shell gutter and 4pt vertical strip padding. Shared segmented tabs already have 12pt label padding; increase their target separation to 4pt. Underline tabs already have 12pt label padding and 4pt gap. Standard Input/Password/Select use the declared 38pt token; SearchInput and toolbar buttons retain their compact role.

Reusable lesson: egui scopes inherit item_spacing; row/grid gutters must be restored before rendering nested content. Width budgets must include actual visible accessory targets plus native sibling spacing. Explicit add_space adds to native spacing, so use a scoped spacing owner rather than compensating per view.

Native evidence exposed an additional source of inherited zero spacing: shell_frame_view.rs deliberately makes the workspace flush. Gallery must restore spacing from the context theme, rather than copying the already-zero workspace spacing. Query shell geometry keeps its explicit compact layout.
