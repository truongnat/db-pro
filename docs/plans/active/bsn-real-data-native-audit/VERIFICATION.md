# Verification

Baseline source SHA: `cb6e00d633772de538498384a8b14d32d25f7915`.

## Remediation verification — uncommitted working tree based on `cb6e00d633772de538498384a8b14d32d25f7915`

- `cargo +nightly fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS, with inherited unused import warning at `crates/infrastructure/src/sqlserver/introspect.rs:9`.
- `cargo test --workspace` — PASS: 2,133 passed / 0 failed / 56 ignored.
- `cargo test -p db-pro-app` — PASS: 609 passed / 0 failed / 0 ignored. This was rerun after replacing the test that mutated `rust_i18n` process-global locale and caused parallel test interference.
- `cargo test -p db-pro-core table_copy -- --nocapture` — PASS: 4 passed / 0 failed.
- `cargo test -p db-pro-infrastructure --test schema_columns_atomicity_regression parameterized_transaction_failure_rolls_back_replace_delete -- --nocapture` — PASS: isolated SQLite rollback preserved the existing target row.
- `cargo test -p db-pro-app chosen_copy_reads_tables_from_empty_form -- --nocapture` — PASS: 1 passed.
- `cargo test -p db-pro-infrastructure --test pg_integration pg_parameterized_transaction_failure_rolls_back_replace_delete --no-run` — PASS: PostgreSQL failure-injection test compiles.
- `DATABASE_URL=postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture cargo test -p db-pro-infrastructure --test pg_integration pg_parameterized_transaction_failure_rolls_back_replace_delete -- --ignored --nocapture` — FAIL before test setup: PostgreSQL rejected role `dbpro`; `psql -d postgres` also found no local socket. PostgreSQL runtime rollback is NOT VERIFIED. No test DDL was issued.
- `cargo clippy --workspace --all-targets -- -D warnings` — FAIL on pre-existing diagnostics: unused import `DbConnector` in SQL Server introspection, two `manual_div_ceil` warnings in diagram spatial code, and a needless borrow in an editor test.
- `cargo build --release --locked -p db-pro-native --features capture` — PASS (warnings: inherited unused SQL Server import and dead-code warnings in `capture.rs`). Attempted isolated synthetic table capture from `target-build/release/deps/db_pro_native-8e824f9f72284a33` with `DB_PRO_DATA_DIR=/tmp/db-pro-ui-audit-capture`, `DB_PRO_CAPTURE_TO=/tmp/db-pro-table-light.png`, and `DB_PRO_WINDOW_SIZE=1280x800`; process timed out after 120 seconds and `/tmp/db-pro-table-light.png` was absent. Runtime logs show only the capture fixture's seeded 11-table connection. This is NOT BSN UI evidence; no screenshot was fabricated or saved.
- Static catalog sweep across `en`, `vi`, `zh`, `ja`, `ko` — 521 keys each; 0 missing/extra keys, 0 duplicate keys, 0 placeholder mismatches. App regression `locale_catalogs_have_matching_keys_and_placeholders` also passes.

Known pre-fix test failure: the table-copy failure-injection test observed the target row missing after replacement INSERT failure. The fixed implementation and SQLite rollback test now pass.

## Planning/safety evidence — 2026-10-20

- Read `docs/plans/FEATURE_LIFECYCLE.md`, `docs/plans/STATUS.md`, `docs/architecture/system-overview.md`, `docs/architecture/provider-contract.md`, `docs/architecture/security-boundaries.md`, `docs/10-egui-native-migration-plan.md`, and `RULES.md`.
- Read relevant active plans: `ux-friendliness-audit`, `rc1-full-product-qa`, `native-feature-ui-cleanup`, and `native-ui-focus-consistency`.
- Read skills: `ui-design-review`, `better-layout`, `better-accessibility`, `a11y-audit`, `clean-code-review`, and `screenshot`.
- Inspected native `Activity` enum (`crates/app/src/app/app_types.rs:57-74`): 15 route variants.
- Inspected only non-secret profile metadata for `BSN`: PostgreSQL, DB Pro `readonly=false`. The private endpoint, user, and credentials are intentionally omitted.
- All BSN operations used `DbProRuntime` with `DB_PRO_DISABLE_KEYRING=1` and the existing DB Pro data directory. No profile edit, config write, or secret read was performed. A temporary probe resolved the profile by name, connected, and disconnected.
- `DB_PRO_DISABLE_KEYRING=1 cargo run --quiet --manifest-path /tmp/db-pro-bsn-audit-probe/Cargo.toml` — PASS after correcting a temporary probe syntax error. Successful path: `SELECT 1` (validated one returned row), plain `EXPLAIN (FORMAT TEXT) SELECT 1`, schema introspection (7 schemas, 1,302 tables, 184 views, 39,095 columns), and one bounded row fetch from a table with positive catalog row estimate. The fetched row values were not printed, logged, stored in evidence, or displayed; disconnect succeeded.
- One earlier probe used a numeric-only-table selection heuristic and exited 9 because no such table matched. This was a probe-selection miss, not a provider/query failure; the later generic bounded-row selection succeeded.
- Cargo emitted an existing warning in `crates/infrastructure/src/sqlserver/introspect.rs:9` about an unused `DbConnector` import. No product tests, build gates, UI architecture scan, or native UI launch were performed for this audit run.
- Inspected prior synthetic screenshots: `native-feature-ui-cleanup/evidence/{postgres-demo/welcome-light-1280x800.png,query-agent-dark-1280x800.png,schema-definition-dark-1280x800.png,diagram-design-dark-1280x800.png,security-dark-1280x800.png,monitoring-dark-1280x800.png,transfers-light-1280x800.png,settings-general-dark-1280x800.png}`. Their logs/provenance and exact visual limits are recorded in `FINDINGS.md`; none is BSN-native UI evidence.
- Reviewed `../../_audit-2026-10-13/CAPTURE-EVIDENCE.md`: it documents 102 historical visual captures across 34 surfaces and 3 requested widths; source SHA is absent from that manifest, and it explicitly excludes keyboard focus/hover/drag/IME. These are historical fixture/presentation captures, not BSN UI evidence; one separate native ER screenshot is live SQLite and is labeled as such in `FINDINGS.md`.
- Earlier same-environment evidence in `native-feature-ui-cleanup/VERIFICATION.md` records that the virtual display had no discoverable/capturable native app window and the capture driver opened SQLite/presentation states. This pass did not rerun that known-unavailable visual path.
- Source review of `crates/app/src/table/table_editor_context.rs:10-70` and `crates/app/src/table/table_data_state.rs:208-260` at baseline SHA confirms table editability gates; this does not identify the specific BSN table's read-only cause without its selected-object metadata/UI state.
- This statement reflects the planning/API-only phase; a native Xvfb UI run and focused source/test fix were performed in the subsequent continued audit recorded below.

## Earlier pending gates — updated by continued run below

- The earlier planning/API-only phase had not captured BSN UI or tested the shipped app. Those gates now have initial live evidence; the broad per-feature matrix remains incomplete.
- The earlier source-only transfer `Replace` candidate is superseded by the verified table-copy failure injection and SQLite rollback test recorded above; independent PostgreSQL runtime remains blocked. Never test transfer or rollback on BSN.

## Continued native BSN run

- Shipped native window launched in isolated Xvfb display `:102` using the saved `BSN` connection. Connection succeeded; app showed live counts for 7 schemas, 1,302 tables, 184 views, and 454 functions. The broader catalog probe separately reports 39,095 columns.
- Captured and visually inspected eleven privacy-safe live BSN screenshots at 1280×800, Light mode during the initial run: Welcome/connected, expanded master tree, empty `m_all_bank` data, stale-filter error, one-row `SELECT 1` result, one Tab-focus ring state, `m_all_bank` Structure/type tooltip/indexes/empty foreign keys, and ER search. See `evidence/runtime/MANIFEST.md` for binary identities, capture provenance and PNG SHA-256. No customer rows were retained.
- Reproduced P1 stale table query state: opening `m_article` after `m_all_bank`, then `m_accounting`, rendered the selected table but the data read still used the prior `all_bank_cd` column, producing PostgreSQL missing-column errors. No production mutation was issued.
- A temporary regression test failed before a temporary source reset and passed with that reset. Both temporary source/test edits were removed to keep the final audit scope read-only; no regression test or fix remains in the working tree.
- `cargo test -p db-pro-app --lib` — PASS: 610 passed while the temporary reset experiment was present; this is not a verification run of the final source tree.
- Live layout findings: in `master.m_all_bank` Structure at 1280×800, long PostgreSQL data types truncate; hover revealed one complete type. On ER Diagram, searching `m_all_bank` reported five matches but the canvas remained blank after 1-hop selection, 100% zoom, and a five-second wait. The latter is a confirmed UI symptom with root cause unisolated. Index and empty-FK views were also inspected. Dark/wide dimensions were not tested.
- The experiment-only `cargo build --release --locked --features capture -p db-pro-native` passed while the temporary reset was present. After removing temporary source/test edits, `cargo run --release --locked --features capture -p db-pro-native` rebuilt and launched the final-source app; its binary SHA-256 is `45314f7aebddae65e2c01e8a3f227c7e9d79faa41378722a1103798c6241d2e8`. Warnings: inherited unused `DbConnector` import and capture-only dead code. The final-source app connected to BSN and ran safe `SELECT 1`.
- No product source/test fix is retained. The current code path remains unfixed; no post-fix runtime was performed.

## Continued read-only BSN UI audit — Xvfb :2

- Final-source native binary: `/data/cargo-target/release/db-pro-native`, SHA-256 `45314f7aebddae65e2c01e8a3f227c7e9d79faa41378722a1103798c6241d2e8`; runtime uses the saved BSN profile, `DB_PRO_DISABLE_KEYRING=1`, `WINIT_UNIX_BACKEND=x11`, `DB_PRO_WINDOW_SIZE=1280x800`, `DISPLAY=:2`. Actual full-window captures are 1280×800. Cropped evidence artifacts record their actual crop dimensions in the manifest.
- Used only UI navigation plus a read-only role-metadata refresh. Inspected the Tools hub at two scroll positions, Security role list, Monitor health advisor (Auto-refresh unchecked), default Schema Workbench, empty Execution History, General Settings and Agent offline-draft state. No Agent provider call, query, SQL write, DDL, export, backup, transfer, maintenance, role mutation, setting save or other database mutation was issued in this continuation. No customer row values were captured.
- Reproduced persistent success feedback: after role refresh, the status bar continued to show `Security · 18 role(s)` on Tools/Welcome. The live full-screen capture is `evidence/runtime/bsn-tools-hub-live-light-1280x800.png`. Three Tab presses reached a visible focus treatment on the shell forward button; this is not a complete focus-order test.
- Inspected a privacy-redacted role-row crop: PostgreSQL predefined roles are included in the live catalog and the UI shows the destructive Drop affordance. No Drop was clicked; confirmation and actual server behavior were not exercised. Monitoring evidence masks relation identifiers and metric detail.
- A separate native presentation-fixture results capture shows status text `0 functions` touching the `SQL Editor` context at 1280×800. This is SYNTH SQLite fixture evidence, not BSN evidence. No dark-mode or 1440×900/1920×1080 BSN screenshots were collected.
- No tests, build, lint or source edits were run in this continuation. Prior command outcomes remain explicitly limited to the earlier working tree and are not retested here.

## Remaining blocking evidence

- Proposed source remediation remains unimplemented; the audit’s focus is the verified failure and scope-safe fix recommendation.
- Full feature coverage, dark mode, wide sizes, focus/keyboard traversal and i18n visual pass: NOT VERIFIED; see the row-level matrix in `CHECKLIST.md`.
- No product source/test edits were retained. No BSN `INSERT`, `UPDATE`, `DELETE`, DDL, export, backup, transfer, maintenance or admin action was attempted.

## Continued read-only feature walkthrough — final-source BSN session

- Reused `/data/cargo-target/release/db-pro-native` SHA-256 `45314f7aebddae65e2c01e8a3f227c7e9d79faa41378722a1103798c6241d2e8`, Xvfb `:2`, Light 1280×800 and the saved BSN profile. No profile or preference changes were made.
- Inspected live Schema Compare empty state, Schema Workbench Views, Transfer seed/masking surface, Saved Tasks empty state and unsaved SQL/backup-task forms, Files/Data/Problems/History activities, and the in-session Query History after executing only `SELECT 1`. New captures are linked and hashed in `evidence/runtime/MANIFEST.md`; no sensitive row, saved-query text, local path or endpoint was retained.
- Reproduced a clipped Schema Workbench action row. The synthetic-seed `Apply INSERT (run)` affordance was visible without a Production confirmation on the connected profile; the environment mapping was not independently validated. No table was selected, no INSERT/DDL/export/backup/transfer/admin operation was activated, and no task was saved or run.
- Screenshots and metadata prove only the named light-mode viewport/surface; they do not establish dark mode, requested wide viewports, full keyboard traversal, accessibility-tree behavior, or the unvisited feature subpaths. These remain NOT VERIFIED in `CHECKLIST.md`.
- No tests, build, formatter, linter or CI command was run in this visual continuation. No product source/test code was changed; screenshots and audit documentation were updated.
