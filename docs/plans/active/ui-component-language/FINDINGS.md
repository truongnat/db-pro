# Findings

## P2 — Primary native controls still bypassed canonical components

At baseline `8da3c108ce237811eef6931d6c081cef9bf85447`, the value inspector
used raw `ui.selectable_label` and `ui.button` controls, while the Explorer
toolbar hand-built a search frame/TextEdit and a raw refresh context button.
The repository already provides `SegmentedTabs`, `Button`, `SearchInput`, and
`ctx_menu_item` for these patterns.

This is a contained visual-consistency gap, not a data or provider defect. The
minimal fix is to reuse those primitives on the two named surfaces and leave
the broader #288 inventory for later focused slices.

## P2 — Runtime screenshot provider is unavailable in this environment

At follow-up implementation SHA `78eb8c34506f82c0e8e9221f6a028771e15c2fd7`,
the release native binary launched and introspected a 977-table local fixture,
but the desktop provider reported `screenshot=false`; Orca could not identify
the native window for accessibility capture. This prevents the required
1280×800, 1440×900 and 1920×1080 visual evidence.

The clean-code scan also reports inherited long methods in
`schema_workbench.rs`/`schema_workbench_form.rs`; the follow-up only changes
button composition and does not claim those methods were refactored.

## Follow-up slice — Tasks, Settings Sessions, and Query Output

- `tasks_view.rs`: Migrated manual action buttons (Run, Disable schedule, Schedule 60s, Delete, Save task, Cancel) to canonical `Button` with appropriate size `ButtonSize::Sm` and variants (`Default`, `Secondary`, `Destructive`, `Ghost`).
- `settings_view.rs`: Migrated workspace session management buttons (Save workspace, Restore, Duplicate, Delete) to canonical `Button`.
- `query_output_view.rs`: Migrated EXPLAIN ANALYZE button to canonical `Button` with `enabled` binding.
- `components/mod.rs`: Expanded `primary_native_surfaces_do_not_reintroduce_raw_buttons` test to prevent raw `ui.button` regressions in these files.
