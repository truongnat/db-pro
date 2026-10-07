# Data Grid Record View

State: RUNTIME_VERIFY · implementation authorized 2026-10-06; source and automated verification complete, exact-height/live-provider evidence pending.

User scope: research delivered, then implementation authorized; click a Record field label/value to copy its corresponding text. Reference: supplied Data Grid screenshot; “db-eavar” interpreted as DBeaver.

Implemented behavior: a narrow vertical rail immediately left of the `#` gutter, with Record anchored at its bottom. Selecting one row and pressing Record replaces the grid with that row's fields. Grid returns to the same result and selection.

Implemented scope: Table → Data, viewing/copying one loaded record in a shared Field/Value Table. No new SQL, provider adapter, Query tab, editing workflow, multi-record layout, or automatic database Save. Existing Save/Discard semantics remain authoritative.

Architecture: native egui in `crates/ui`; reuse TableDataState selection, UiQueryResult, staged_cell_value. Keep presentation mode distinct from the existing right-side inspector. Mount the rail outside grid scroll areas. See FINDINGS.md for source evidence and acceptance details.

Acceptance: exactly one selected row, complete values, correct NULL/empty distinction, same staged values as Grid, stable selection on Grid/Record switches, safe result replacement, visible rail at all required sizes/states. Runtime evidence is required before feature completion.
