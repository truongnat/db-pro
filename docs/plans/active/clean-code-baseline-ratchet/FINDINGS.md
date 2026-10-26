# Findings — 23 cc-scan errors

Source of truth: `python3 tools/cc-scan.py . --json --no-baseline` at HEAD
`52507402` (23 error-severity findings: 15 `HUGE_FUNCTION`, 7 `HARD_COMPLEXITY`,
1 `HARD_PARAMS`; 126 warnings; score 0.0/grade E over 1083 files, 188 363 source
lines, 9 112 functions).

Each row was read at the reported line before being classified — the "what it is"
column is the reason for the decision, not a guess from the rule name.

## Group A — declarative tables; the rule does not apply (accept, no refactor)

| # | Site | Metric | What it is |
|---|---|---|---|
| 1 | `crates/ui/src/theme/tokens/semantic.rs:191` `light()` | 72 lines | struct literal assigning every semantic token role for the light theme |
| 2 | `crates/ui/src/theme/tokens/semantic.rs:266` `dark()` | 71 lines | same for the dark theme |
| 3 | `crates/app/src/schema/schema_workbench.rs:146` `default()` | 73 lines | struct literal of initial workbench state |

Splitting a token table into sub-builders would scatter one visual vocabulary
across functions and make the theme harder to audit side by side; the file is the
readable form. Both `semantic.rs` sites already carry
`// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass`, i.e. the
shape was judged deliberate before. The allow is rule-scoped, so the
`HUGE_FUNCTION` tier still fired. Decision: accept through the baseline; **do not**
extend the inline allow.

## Group B — egui draw pipelines (accept, watch list)

| # | Site | Metric | What it is |
|---|---|---|---|
| 4 | `crates/app/src/explorer/explorer_toolbar_view.rs:67` `draw_filter_workbench` | 73 | popup: name-match mode + per-kind toggles, one section each |
| 5 | `crates/app/src/query_workbench/query_dialogs_view.rs:250` `export_result_to_disk` | 80 | validate path → atomic publish → overwrite guard (#244, #539) |
| 6 | `crates/app/src/security/security_rls_view.rs:160` `draw_policy_form` | 80 | policy form fields, returns actions |
| 7 | `crates/app/src/security/security_role_details_view.rs:37` `draw_attributes` | 76 | role attribute rows |
| 8 | `crates/app/src/table/table_insert_row_view.rs:68` `draw_insert_row_dialog` | 86 | insert-row dialog: columns, types, nulls, defaults |
| 9 | `crates/app/src/visual_query_builder/visual_query_builder_surface_view.rs:263` `draw_filters` | 74 | WHERE/HAVING/ORDER/LIMIT section |

Immediate-mode UI code reads linearly by nature: one widget per row. These are the
weakest candidates for a "split by business step" refactor because the steps *are*
the widget order. #5 and #8 are worth revisiting if a change touches them anyway —
#5 interleaves validation with rendering, which is a real seam.

## Group C — genuine branch complexity (named follow-up refactors, P2)

Ordered by value, not by size. These are functions where the branch count is a
testability problem, which is exactly what `HARD_COMPLEXITY` claims.

| # | Site | Metric | Why it matters |
|---|---|---|---|
| 10 | `crates/core/src/domain/saved_task.rs:106` `sql_embeds_credential_literal` | 27 branches | highest in repo; a hand-written SQL tokenizer guarding saved-task credentials — a missed case leaks a password, so it needs a table-driven token spec and a case matrix |
| 11 | `crates/app/src/app/app_types.rs:338` `is_user_visible_schema` | 20 branches | the system-schema boundary behind the destructive `DROP SCHEMA` guard (#348/#402); per-driver sets belong in a per-driver table, not one boolean chain |
| 12 | `crates/core/src/application/object_mutation_builders.rs:104` `alter_existing_table` | 18 branches, 105 lines | emits DDL; wrong output is a data-corruption class defect (P1 territory per severity policy) |
| 13 | `crates/core/src/domain/explain_plan.rs:124` `parse_sqlite_explain_json` | 18 branches, 102 lines | provider-specific parser (SQLite only) — the natural split is per node kind |
| 14 | `crates/infrastructure/src/postgres/connector.rs:537` `execute_batch` | 76 lines | transaction/commit path; a broken batch is a P1 by policy |
| 15 | `crates/infrastructure/src/backup/pg_dump.rs:237` `backup` | 78 lines | shells out and writes files; steps (tool check, temp file, run, verify) are already implicit |
| 16 | `crates/app/src/app/app_lifecycle.rs:173` `prepare_frame` | 18 branches, 68 lines | per-frame pump; the boot/warm-up branches are separable from steady-state |
| 17 | `crates/ui/src/diagram/design_mode.rs:163` `plan_design_draft` | 78 lines | draft → mutation preview; per-object-kind builders |

## Group D — developer/capture tooling (accept; cheapest real fix)

| # | Site | Metric | What it is |
|---|---|---|---|
| 18 | `crates/app/src/workspace/data_cleanup_capture.rs:7` `open_data_cleanup_for_capture` | 171 lines | largest function flagged in the repo; a `surface → view state` dispatcher used only by the capture harness, executing no provider operation |
| 19 | `crates/native-app/src/capture.rs:459` `open_workspace_capture` | 17 branches | capture dispatcher over env-var-selected surfaces (file already carries an allow at :425) |
| 20 | `crates/native-app/src/main.rs:353` `initialize_sample_sqlite` | 16 branches | first-run sample-database bootstrap |

Not runtime user-facing code, so a defect is a bad screenshot rather than bad data.
#18 is nonetheless the single cheapest structural win in the whole list: a 171-line
chain of `if … { open view X }` is a lookup table with no behaviour to lose.

## Reconciling 23 findings against 21 functions

| # | Site | Metric | Note |
|---|---|---|---|
| 21 | `crates/app/src/query_workbench/query_context_view.rs:254` `draw_context_chip_contents` | 6 params | the only `HARD_PARAMS`; already exempted for `TOO_MANY_PARAMS` at the call site — the 6th argument is `max_width` derived at the caller, so an options struct is the honest fix |

The scan reports **23 findings over 21 distinct functions**: #12
(`object_mutation_builders.rs:104`) and #13 (`explain_plan.rs:124`) each cross two
error thresholds, `HUGE_FUNCTION` and `HARD_COMPLEXITY`, and are counted once per
rule. `app_lifecycle.rs:173` and `query_context_view.rs:254` carry a single error
each; their other findings (`LONG_FUNCTION`) are warning-severity and stay
unfrozen.

## Observation on the exemption mechanism itself

Seven of the fifteen `HUGE_FUNCTION` sites sit directly under a
`// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass` comment with
identical wording. A blanket reason repeated across sites is the exemption system
being used as a mute button rather than an argument; and because allows are
rule-scoped while the rules are tiered (`LONG_FUNCTION` warns, `HUGE_FUNCTION`
errors), the earlier author could silence the warning tier and leave the error
tier firing. Any future allow must name the concrete constraint that makes the
shape correct, at one site.

## Known weaknesses of the frozen baseline (accepted, not hidden)

`tools/cc-scan.py:874-886` `apply_baseline` suppresses by **(rule, file) count**,
not by finding identity, iterating in severity→file→line order. Consequences:

1. A PR that fixes one error and introduces another **in the same file and rule**
   nets to the frozen count and passes silently.
2. Splitting a legacy function into two compliant ones lowers the count — correct
   and safe (ratchet tightens).
3. Moving a flagged function to a **new file path** exceeds that file's zero budget
   and fails CI, even though nothing got worse. Remedy: re-run
   `--update-baseline` in the same PR, which is legitimate only because total
   counts stay equal or fall.
4. `cc-scan.py:806` skips dot-directories and `skipRulesForTests` (config line 72)
   exempts test files from `LONG_FUNCTION`/`HUGE_FUNCTION`, so the gate covers
   production shape debt only.
