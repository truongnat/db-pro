---
schema_version: 1
id: "MEM-PLAN-bsn-real-data-native-audit-601af4e34d4d"
title: "Plan archive: Bsn Real Data Native Audit"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, bsn-real-data-native-audit]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "601af4e34d4d22f6b3564af951ab02863688616af95dce49c0a772cf49748f12"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/bsn-real-data-native-audit/PLAN.md"
  - "docs/plans/active/bsn-real-data-native-audit/FINDINGS.md"
  - "docs/plans/active/bsn-real-data-native-audit/VERIFICATION.md"
  - "docs/plans/active/bsn-real-data-native-audit/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/bsn-real-data-native-audit/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# BSN Real-Data Native Product Audit

State: `IMPLEMENTING` — prior source/test remediation is recorded below. This continuation is a read-only QA audit; safe BSN-native captures now cover additional real feature surfaces, but coverage remains partial and no new product source change is authorized by this request.

Baseline source SHA: `cb6e00d633772de538498384a8b14d32d25f7915`.

## Goal

Audit and repair concrete native DB Pro defects found while reviewing the product against the owner's BSN PostgreSQL use case. Prioritize UI layout and localization, and follow confirmed logic defects through the app/provider boundary. Safe BSN reads are allowed; all data mutations and provider failure tests must use isolated fixtures.

## Scope

- Existing 25-row native feature audit in `CHECKLIST.md`; verify each recorded finding against current source and available runtime evidence.
- Fix confirmed correctness and user-visible defects, including cross-provider copy transaction failure atomicity, catalog defects, and incomplete localization of audited high-impact surfaces.
- Revalidate historical screenshot-only clipping candidates against current source/captures; fix only those reproduced on the current implementation.
- Audit supported `en`, `vi`, `zh`, `ja`, and `ko` catalogs for key structure, interpolation placeholders, and native view coverage.
- Preserve DB Pro UI components, theme tokens, actions, provider capability boundaries, and BSN data safety.
- Keep all claims separated into current native/runtime evidence, isolated-provider tests, source-only evidence, and historical synthetic captures.

## Safety and privacy boundary

- The saved profile is named `BSN`, uses PostgreSQL, and has the DB Pro `readonly` flag **false**. The app-side flag is not a safety boundary for this audit.
- Audit actions against BSN are read-only: metadata inspection and bounded `SELECT`/non-`ANALYZE` `EXPLAIN` only. No `INSERT`, `UPDATE`, `DELETE`, DDL, role/privilege changes, session termination, settings changes, scheduled-task execution, restore, or transfer apply.
- Do not export, backup, or persist raw BSN rows. Screenshots and logs must not contain credentials, customer identifiers, private addresses, or sensitive query text; use aggregate/sanitized results or redact before storing evidence.
- Any mutation behavior that cannot be proven without a write remains `NOT VERIFIED`; do not infer it from a fixture or a unit test.

## Evidence rules

- Reuse prior audits only for the exact feature/path/state they prove. The existing `ux-friendliness-audit`, `native-feature-ui-cleanup`, `native-ui-focus-consistency`, and RC1 QA are references, not fresh BSN runtime proof.
- The native capture driver's `DB_PRO_CAPTURE_*` openers seed presentation fixtures. Such screenshots are explicitly synthetic and never count as BSN data evidence.
- Capture each activity in Light and Dark at 1280×800 where the runtime permits. For the data/query/schema/ER/monitoring/security critical paths, include wide-layout checks at requested 1440×900 and 1920×1080 and report actual framebuffer sizes. Capture relevant loading/empty/error/focus states where safely reproducible.
- Static screenshots prove only visible layout. Keyboard focus, state transitions, query behavior, and command-to-database round trips require runtime interaction/log/test evidence.

## Out of scope

- Writes, DDL, administrative actions, exports, backups, transfers, restores, or preference/profile changes against BSN.
- Treating historical fixtures as current BSN UI proof; completing unobservable runtime checks by inference.
- Archived React/Tauri frontend changes, external UI-framework dependencies, or wholesale view/theme rewrites.

## Acceptance

1. Each confirmed in-scope defect has a regression test or direct reproduction and a minimal root-cause fix.
2. Transfer `Replace` and multi-statement copy writes execute atomically on the target using the provider transaction contract; failure leaves prior target data intact.
3. All five locale catalogs resolve the tested keys without duplicate-key loss or placeholder mismatch; confirmed high-impact native strings are localized and Korean export copy describes exporting correctly.
4. Historical layout candidates are either reproduced and fixed on the current implementation or retained as explicitly unverified; no screenshot-only inference is reported as a current defect.
5. Safe BSN runtime reads remain read-only; provider mutation tests use disposable local fixtures only.
6. Focused tests, package checks, native build/capture where available, source SHA, exact limitations, and Vietnamese handoff are recorded. This plan remains active until all required live screenshot, feature, focus and viewport gates are evidenced or explicitly dispositioned; safe captures are now available, but the complete matrix remains incomplete.


## Source: `FINDINGS.md`

# Findings

Source base/HEAD at implementation start: `cb6e00d633772de538498384a8b14d32d25f7915`. Product and evidence edits are uncommitted on `main`; changed-tree test results are recorded separately in `VERIFICATION.md`.

## Scope and evidence status

- The native shell exposes 15 `Activity` routes in `crates/app/src/app/app_types.rs:57-74`; the detailed product/UI checklist is `CHECKLIST.md`.
- `docs/plans/active/ux-friendliness-audit/` already contains a broad 13-area UX/source inventory. Its historical claims are not a substitute for this BSN live-data/native-UI audit.
- `crates/native-app/src/capture.rs:135-140,343-363` routes audit/capture scenarios; `crates/app/src/workspace/data_cleanup_capture.rs:1-7` calls the data cleanup capture content presentation-only fixtures. Synthetic captures must be labeled and cannot establish live BSN behavior.
- The saved profile `BSN` was inspected in DB Pro's local metadata store without reading secrets. It is PostgreSQL and has `readonly=false`. The endpoint was private-network data and is deliberately omitted from this plan.
- The saved BSN profile connected successfully through `DbProRuntime`. Safe `SELECT 1`, plain `EXPLAIN (FORMAT TEXT) SELECT 1`, live schema introspection, and a `LIMIT 1` table-data API fetch from a populated table all succeeded; exactly one row was fetched and its values were discarded without logging or display.

## Confirmed findings and remediation

- **P1 — table-copy Replace could lose existing target rows on later statement failure.** Reproduced before the fix by `replace_failure_preserves_the_existing_target_row`; the replacement INSERT failure left the prior target row deleted. Core now submits the full write batch through `execute_transaction`; SQLite failure-injection integration proves rollback preserves the row. The PostgreSQL regression was added and compiles, but runtime verification is BLOCKED: the documented disposable fixture URL reached a server that rejected role `dbpro`; a local socket check found no PostgreSQL server. No BSN mutation was attempted. Source anchors at HEAD `cb6e00d633772de538498384a8b14d32d25f7915` plus uncommitted changes: `crates/core/src/application/table_copy.rs:57-63,103-119`, `crates/native-app/src/table_copy_port.rs:59-79`, `crates/infrastructure/src/sqlserver/connector.rs:159-177,407-457`.
- **P2 — several high-impact app/table/query strings were hard-coded or lost from locale catalogs.** At the uncommitted source tree based on `cb6e00d633772de538498384a8b14d32d25f7915`, removed duplicate root `diagram:` maps that shadowed diagram labels; corrected Korean export wording; translated previously English Explain/Explain ANALYZE and lint labels, plus Vietnamese access-token/filter terms; localized Settings language (`crates/app/src/settings/settings_general_view.rs:33`), destructive/unsaved Query dialogs (`crates/app/src/query_workbench/query_dialog_surface_view.rs:24`; `query_save_dialog_surface_view.rs:86`), Table Data controls/filters/presentation, row-conflict (`crates/app/src/table/table_conflict_dialog_surface.rs:29`) and insert-row (`table_insert_row_surface_view.rs:30`) in all five supported locales. Catalog tests enforce key parity, duplicate absence, placeholder parity, and no-English regressions for the audited terms. This remains scoped to the audited Settings/Query/Table paths; it does not assert that every native surface in the full product is translated.
- **Test isolation — global locale mutation made parallel tests flaky.** Replaced the locale-switch test that changed process-global state with a deterministic static catalog validator; Settings render test now uses default English and does not change the global locale. Full app tests pass after this change.

- **P1 — opening another table reuses the previous table’s filters/sort/result and can fail the new table’s read.** Reproduced in the shipped native UI against the saved BSN profile: after navigating from `master.m_all_bank` to `master.m_article`, the grid displayed `m_article` but reported `query syntax error: column "all_bank_cd" does not exist`; no write or DDL was issued. Source trace: `crates/app/src/workspace/workspace_actions.rs:80-121` changes `selected_table` and calls `request_table_data()` without resetting `TableDataQueryState`; `build_load_data_command` forwards the retained `filters`/`sorts` in `crates/app/src/table/table_data_view.rs:12-27`. A temporary test-first experiment failed on retained filters; a temporary reset cleared the test, but both source and test edits were removed to honor the current read-only audit scope. The finding remains open. Evidence screenshots: `evidence/runtime/bsn-table-switch-error-light-1280x800.png` and `evidence/runtime/bsn-master-empty-light-1280x800.png`. Only empty table results and the error state were kept; no BSN record values were captured.

- **P1 — targeted table retry clears unrelated staged edits on success (source-confirmed, open).** At HEAD SHA `cb6e00d633772de538498384a8b14d32d25f7915`, `crates/app/src/table/table_mutation_state.rs:134-146` narrows the submitted retry to one target; `crates/app/src/runtime/operation_events.rs:378-395` routes a successful staged operation to `staged_apply_completed`; `crates/app/src/table/table_mutation_actions.rs:524-537` clears the entire staged-change set. Failure scenario: stage rows A and B, fail and roll back, retry B only, then success clears A locally even though A was not submitted. Existing tests cover failure preservation, not this success case (`crates/app/src/app/tests/app_tests.rs:6332-6450`). No provider reproduction or source modification was performed in this audit.
- **P2 — stale activity feedback persists on unrelated surfaces (live BSN UI + source).** At HEAD SHA `cb6e00d633772de538498384a8b14d32d25f7915`, `FeedbackState.runtime_message` is an unscoped string without a timestamp (`crates/app/src/shell/feedback_state.rs:3-14`); `runtime_status` returns any non-empty message (`crates/app/src/connection/connection_status.rs:154-163`), and the shell always supplies it to the footer (`crates/app/src/shell/chrome/shell_chrome_view.rs:120-148`; `shell_statusbar_view.rs:113-121`). After refreshing BSN role metadata, the footer still read `Security · 18 role(s)` after switching to Tools/Welcome. Evidence: `evidence/runtime/bsn-tools-hub-live-light-1280x800.png`. Impact: a success from one activity is presented as current context in another. Recommend scoped/expiring status feedback or clearing activity-specific success on navigation; preserve actionable errors with explicit context.
- **P2 — predefined PostgreSQL roles receive destructive Drop affordances (live UI + source).** At HEAD SHA `cb6e00d633772de538498384a8b14d32d25f7915`, `DatabaseUser` has no system/predefined-role flag (`crates/core/src/domain/user.rs:3-10`); `security_roles_view.rs:43-86` renders a red Drop button for every role. The live BSN role catalog included PostgreSQL predefined roles; the privacy-redacted capture shows a Drop button beside a role without exposing its name (`evidence/runtime/bsn-security-role-drop-one-row-crop-live-light-1280x800.png`). The confirmation dialog is present (`security_confirmation_view.rs:19-29,56-70`); no role action was activated and no server rejection is claimed. Impact: UI offers an impossible destructive operation for provider-defined roles and creates avoidable error/unsafe-action pressure. Recommend provider-derived role kind/capability and disable/hide Drop for non-droppable roles with an explanation.
- **P2 — status-bar content collides at 1280×800 (synthetic presentation reproduction).** At HEAD SHA `cb6e00d633772de538498384a8b14d32d25f7915`, `shell_statusbar_view.rs:47-64,69-120,123-138` places unbounded connection/database/schema identity labels, a fixed-width runtime message and right-aligned context in one horizontal row. A native capture fixture shows `0 functions` running directly into `SQL Editor` (`evidence/runtime/statusbar-overflow-crop-synthetic-light-1280x800.png`; crop is 1280×28 from a 1280×800 synthetic SQLite results screen). This is a verified fixture layout collision, not a BSN reproduction. Recommend flexible width budgeting and ellipsis/prioritization for lower-priority identity labels; verify actual BSN at narrow and wide sizes before remediation.
No P0. Two P1 defects remain open: targeted retry can clear unrelated staged edits after success (source-confirmed, no provider reproduction) and table switching reuses stale query state (live BSN reproduction). The earlier table-copy Replace defect is fixed; SQLite rollback is tested and PostgreSQL runtime remains blocked. Additional verified P2 UI findings are listed below. The user's separate "read-only" observation was not reproduced for `master.m_all_bank`: the saved BSN profile is `readonly=false`, its Structure view has a primary key, and Add Row is enabled; Save was disabled because no edit was staged. `m_article` is not a valid read-only diagnosis while its Data fetch is failing. Exact cause for the originally reported table/view remains NOT VERIFIED.

## UI evidence inspected

- `../native-feature-ui-cleanup/evidence/postgres-demo/welcome-light-1280x800.png`: shell and Welcome content fit in the frame; SQLite is active and the BSN/PostgreSQL profile is only collapsed in the sidebar. Not a BSN view.
- `../native-feature-ui-cleanup/evidence/query-agent-dark-1280x800.png`: query, sidebar and Agent share the viewport; the center editor is relatively narrow, but no overlap/clipping is visible. Footer shows an old missing-demo-database error. Log: `../native-feature-ui-cleanup/evidence/query-agent-light-1280x800.log` confirms fixture database `fullstack_starter` was absent when captured. Neither issue is a BSN defect.
- `../native-feature-ui-cleanup/evidence/schema-definition-dark-1280x800.png`: the code editor occupies the content area without nested outer card; SQL shown is fixture text. Footer has the same stale fixture error.
- `../native-feature-ui-cleanup/evidence/diagram-design-dark-1280x800.png`: synthetic schema-map design draft reports 215 catalog tables and 12 relationships, with five table nodes drawn; it does not prove full-schema rendering. `security-dark-1280x800.png` also shows a separate 10-table fixture ER canvas beside disconnected security guidance.
- `../er-diagram-canvas-ux/screenshots/er-1280x800-dark.png` is an actual native SQLite runtime capture, not BSN. It renders 73 tables / 94 relationships at 21% auto-fit; table names are truncated in the overview but zoom/search affordances remain visible. This does not validate the 1,302-table BSN view.
- `../data-grid-record-view/evidence/grid-1280x800.png` and `../table-profile-quality/evidence/profile-dynamic-numeric-1280x800.png` are deterministic native presentation fixtures, not provider evidence; the grid shows synthetic email-like values and a wide timestamp column near the right viewport edge. Horizontal-scroll behavior and whether the edge clipping reproduces on current BSN remain unverified.
- `../native-feature-ui-cleanup/evidence/security-dark-1280x800.png` and `monitoring-dark-1280x800.png`: these show a disconnected/fixture state, not BSN role or monitoring data. The monitoring fixture contains SQL/session text and must not be confused with production content.
- `../native-feature-ui-cleanup/evidence/transfers-light-1280x800.png`: synthetic transfer controls in the sidebar beside an ER fixture. Existing prior work found width clipping at this surface and has separate before/after evidence; this capture is not a fresh BSN transfer workflow.
- `../native-feature-ui-cleanup/evidence/settings-general-dark-1280x800.png`: one General heading and a compact control group; no visual defect is confirmed from this synthetic screenshot.

### Historical screenshot observations — P2 candidates, not verified on BSN/current build

- `../../_audit-2026-10-13/captures/table_1280x800.png`: the rightmost `created_at` grid column is cut at the viewport 

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

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
- No product source/test edits were retained. No BSN `INSERT`, `UPDATE`, `DEL

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — DB Pro read-only QA audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · read-only QA audit |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | `cb6e00d633772de538498384a8b14d32d25f7915` — repository also had pre-existing uncommitted changes |
| Branch / PR | `main` / no PR |
| Scope interpretation | Prioritized read-only QA across native UI/UX, code quality, architecture, functional behavior, security, performance, tests and real BSN-facing surfaces. Findings distinguish live reproduction, source/test/configuration evidence, synthetic fixture evidence and NOT VERIFIED. |
| Out of scope | Product-source fixes, tests, database writes/DDL, admin or destructive actions, exports/backups/transfers, settings writes, full dark/wide capture matrix, complete focus/accessibility traversal, and unsafe BSN operations. |

## 2. Progress checkpoint

- Current HEAD: `cb6e00d633772de538498384a8b14d32d25f7915`.
- Completed acceptance rows: [x] prioritized report with file/line, evidence, impact and recommended fix; [x] 25-feature checklist with explicit LIVE/SYNTH/SRC/NOT VERIFIED status; [x] read-only BSN visual walkthrough across available safe surfaces; [x] screenshot provenance/privacy and finding evidence; [x] source/runtime/test/coverage findings separated; [x] safety exclusions recorded.
- Remaining acceptance rows: [ ] full feature-state matrix; [ ] Light/Dark and 1440×900/1920×1080 live capture; [ ] full keyboard/focus and accessibility-tree/screen-reader verification; [ ] remaining provider-specific functional/error cases; [ ] independent PostgreSQL rollback test on a disposable fixture.
- Findings / risks: P0=0; P1=2 open (`QA_AUDIT_REPORT.md`, targeted retry clears unrelated staged edits on success and live stale table filter on table switch); P2=22 (16 verified source/live/configuration/coverage findings and 6 suspected source-traced runtime/performance risks). Six live BSN UI/safety-affordance findings include the conditional synthetic INSERT gate risk; report states limits and no mutation.
- Tests already run: no tests/build/lint run in this visual continuation. Prior results are recorded in `QA_AUDIT_REPORT.md` and `VERIFICATION.md` and are explicitly qualified; the 610-app-test run used a temporary patch and is not a final-tree verification. Screenshot hashes for 33 retained PNGs are recorded in `evidence/runtime/MANIFEST.md` and must match the final `sha256sum` check below.
- Dependency / blocker changes: added live evidence for additional BSN surfaces and two P2 UI/safety findings; the full matrix, dark/wide rendering, and independent disposable PostgreSQL runtime remain unverified/blocked. BSN remains writable, so no mutation was attempted.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Commit list | No commits created |
| File / surface inventory | `QA_AUDIT_REPORT.md` — prioritized findings and evidence; `docs/plans/active/bsn-real-data-native-audit/PLAN.md` — read-only continuation status; `CHECKLIST.md` — per-feature dispositions; `FINDINGS.md` — current source/runtime findings; `VERIFICATION.md` — exact runtime and test limitations; `evidence/runtime/MANIFEST.md` — capture provenance, dimensions and SHA-256; this handoff file — evidence contract. No product source/test file modified in this continuation. |
| Acceptance mapping | Audit dimensions and severity → `QA_AUDIT_REPORT.md`; feature scope/status → `CHECKLIST.md` F01–F25; live screenshot evidence → `evidence/runtime/MANIFEST.md`; safe-runtime constraints and unavailable gates → `VERIFICATION.md`; status/remaining acceptance → `PLAN.md`. |
| Commands and counts | `sha256sum docs/plans/active/bsn-real-data-native-audit/evidence/runtime/*.png` — 33 hashes verified, exit 0. No tests, build, lint or CI run in this continuation. Historical command results and their limitations are reported in the QA report. |
| CI run IDs / status | Not run in this continuation |
| Known limitations | Only Light mode at 1280×800 was captured live; remaining activity/error/loading/large-data states, dark and wide layouts, full accessibility/focus traversal, visual i18n, provider-specific cases and PostgreSQL rollback remain NOT VERIFIED. Source findings are tied to the recorded HEAD while the worktree includes earlier uncommitted changes; report explicitly identifies that state. |
| Migrations / config implications | None. No preferences, profiles, credentials or database state changed. |
| Out-of-scope changes | Product source, production data, external framework/dependency changes, and remediation. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `cb6e00d633772de538498384a8b14d32d25f7915` |
| Verdict | n/a — research/audit handoff; no independent code-review approval claimed |
| P0 / P1 / P2 counts | Current findings: P0=0, P1=2 open, P2=22 (16 verified source/live/configuration/coverage; 6 suspected). No product-code regression introduced by this documentation/evidence-only continuation is known. |
| Findings | Two P1 defects remain open, including one live BSN reproduction. Six live BSN UI/safety-affordance findings include clipped Structure types, ER search with blank canvas, stale Security status, predefined-role Drop, clipped Workbench actions and the conditional synthetic INSERT gate risk; one separate statusbar collision is synthetic-only. Other P2 source, test and suspected-risk findings are separated in `QA_AUDIT_REPORT.md`. |
| CI disposition | Not run in this continuation; prior CI/test states are separately qualified in the report. |
| Next task(s) unblocked | Separate remediation task may address the confirmed defects after this read-only audit is reviewed. Keep the audit plan active while screenshot/focus/provider gates remain open. |

## 5. Research / audit handoff

- Source date: 2026-10-20.
- Source URLs / references: `AGENTS.md`, `RULES.md`, `docs/plans/FEATURE_LIFECYCLE.md`, active audit plan/checklist/findings/verification, `docs/plans/_template/AGENT_EVIDENCE.md`, inspected source/test/configuration paths cited by exact line in `QA_AUDIT_REPORT.md`, and `evidence/runtime/MANIFEST.md`.
- Factual findings: live BSN screenshots reproduce stale table-filter failure and stale activity-status feedback; live Structure types and Workbench view actions clip; ER search reports matches with a blank canvas; PostgreSQL role UI exposes Drop; Transfer shows a synthetic INSERT action without Production confirmation on the connected profile (conditional risk; not exercised). Safe query History shows `SELECT 1`. A synthetic fixture reproduces statusbar label collision. Source-confirmed targeted retry clears the entire staged set. Full evidence and limitations are in the report and manifest.
- Inference: suspected concurrency and performance impacts are explicitly marked suspected; no customer data mutation, SQL write, or destructive operation was used to substantiate a finding.
- Decision / recommendation: keep the audit plan active and hand off the prioritized defects for remediation; do not treat any NOT VERIFIED matrix item as a pass.
- Unresolved questions: exact cause of the user-reported read-only screen; full activity/feature-state coverage; focus restoration/order and accessibility semantics; dark/wide layout; provider-specific permission and timeout behavior; whether the BSN profile’s environment mapping reflects the intended production policy; independent disposable PostgreSQL transaction-failure verification.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

- Đã tiếp tục kiểm tra UI thực tế BSN ở các màn hình an toàn, bổ sung checklist 25 feature, hai finding P2 về clipping ở Schema Workbench và affordance INSERT có điều kiện, cùng ảnh đã kiểm tra/redact/hash trong manifest. Không sửa source/test và không ghi dữ liệu BSN. Còn 2 lỗi P1 cần remediation; focus/accessibility, dark/wide, nhiều feature và PostgreSQL rollback vẫn chưa xác minh. Giữ plan ở trạng thái active.

