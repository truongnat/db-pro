---
schema_version: 1
id: "MEM-SRC-018b220bdedf38ae"
title: "Source mirror: docs/plans/active/qa-keyboard-correctness-wave/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/qa-keyboard-correctness-wave/CHECKLIST.md"
source_sha256: "018b220bdedf38ae13568db9ae8b4486774e3af2c264391624ee023ff23d1b9d"
mirror_body_sha256: "018b220bdedf38ae13568db9ae8b4486774e3af2c264391624ee023ff23d1b9d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist

## Implementation

- [x] Topbar window-drag hit target no longer focusable (`Sense::CLICK | Sense::DRAG`).
- [x] Global chords fire while the custom SQL editor holds focus.
- [x] Focus rings on files tab strip and tools hub cards.
- [x] `on_schema_loaded` gated by matching `schema_request`; matching failure cancels pending diff.
- [x] Worker-unavailable schema dispatch cancels pending diff (`app/mod.rs`).
- [x] Migration planner validates SQLite declared type/default and ALTER source type; unsupported
      SQL stays empty.
- [x] `table.no_dependencies` added to en/vi/ja/ko/zh catalogs.
- [x] COPY export format offered only for PostgreSQL connections; stale selection coerced to CSV.
- [x] Table switch clears client-side filter/sort/manual row identity.
- [x] SQLite EXPLAIN no longer treats plan keywords (`CONSTANT`, `TEMP`, `SUBQUERY`, …) as relations.
- [x] Quick Open lists each palette action at most once.

## Regression tests (focused runs recorded in VERIFICATION.md)

- [x] `app_lifecycle_tests::tab_focus_traverses_topbar_into_the_workspace`
- [x] `shortcut_catalog_tests::global_chords_fire_while_the_sql_editor_is_focused`
- [x] `shortcut_catalog_tests::chord_matches_when_key_release_lands_in_the_same_frame`
- [x] `runtime_event_handlers_tests` (4 tests)
- [x] `migration_planner` safety tests (4 tests)
- [x] `overlay_state_tests::copy_format_is_only_offered_for_postgres`
- [x] `table_data_state_tests::table_switch_clears_client_side_view_state`
- [x] `explain_plan_tests::sqlite_explain_does_not_treat_plan_keywords_as_relations`
- [x] `palette_catalog_tests::quick_open_lists_each_action_once`

## Gates (execute, do not assume)

- [x] `cargo +nightly fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` — 2,153 passed / 0 failed / 56 ignored
- [x] `cargo build --release --locked -p db-pro-native`

## Runtime evidence (hidden Xvfb)

- [x] Light + dark captures at 1280×800, 1440×900, 1920×1080 (33 PNGs, exact sizes verified).
- [x] Keyboard: Tab reaches shell controls with visible focus rings; Ctrl+P opens/filters/activates
      and Escape closes Quick Open; Ctrl+P/Ctrl+T fire with the SQL editor focused (15 keyboard PNGs).
- [x] Quick Open shows one row per action.
- [x] SQLite export dialog does not offer COPY script.
- [x] Dependency empty state localizes (`table.no_dependencies`).
- [x] Files tab / tools card focus ring visible when focused.

## Documentation

- [x] Plan directory created with PLAN/CHECKLIST/FINDINGS/VERIFICATION.
- [x] `docs/plans/STATUS.md` row added/updated.
- [x] Final Vietnamese summary delivered.
