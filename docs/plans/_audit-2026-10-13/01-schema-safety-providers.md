# Audit — Schema Safety & Providers slice

Date: 2026-10-13. Method: plan-doc claims vs HEAD code/tests; package-scoped `cargo test -p db-pro-infrastructure` runs executed where noted; no workspace-wide builds.

## 1. Slice scope

Plans covered: `schema-columns-runtime` (S1), `schema-indexes-runtime` (S2), `schema-relations-runtime` (S3), `schema-triggers-runtime` (S4), `schema-regression` (S7), `sqlite-view-column-introspection`, `ddl-normalization` (S5), `core-safety-hardening`, `mysql-transaction-read-statement-correctness`, `sqlserver-provider`.

Code areas: `crates/core` (schema model, safety, DDL builders), `crates/infrastructure` (postgres/sqlite/mysql/sqlserver adapters + introspection + integration tests), `crates/ui` schema workbench surfaces.

## 2. Per-plan verdict table

| Plan | Claimed state | Supported state | Verdict | Key evidence |
|---|---|---|---|---|
| schema-columns-runtime | RUNTIME_VERIFY | RUNTIME_VERIFY — SQLite rollback regression test runs live; PG lane unproven | EVIDENCE_GAP | `tests/schema_columns_atomicity_regression.rs:35` PASS today (`cargo test -p db-pro-infrastructure --test schema_columns_atomicity_regression`); `execute_batch` on `DbConnector` `ports/db_connector.rs:78`; PG batch awaits rollback `postgres/connector.rs:316-360`; SQLite batch `sqlite/actor.rs:226-258`. All PG evidence rests on `#[ignore]`d `pg_integration.rs` (29 ignored). |
| schema-indexes-runtime | RUNTIME_VERIFY | RUNTIME_VERIFY — SQLite index lifecycle proven; PG + UI round-trip open | EVIDENCE_GAP | `tests/schema_indexes_runtime_verification.rs:34` PASS today; SQLite `PRAGMA index_list/index_info` `sqlite/introspect.rs:155-183`; autoindex filter `sqlite/introspect.rs:808`. UI surface `table_indexes_surface_view.rs` exists (token-clean). |
| schema-relations-runtime | RUNTIME_VERIFY | RUNTIME_VERIFY — composite FK identity/order proven on SQLite live | EVIDENCE_GAP | `tests/schema_relations_runtime_verification.rs` PASS today; PG grouping `postgres/introspect.rs:656 group_foreign_key_rows`; SQLite `PRAGMA foreign_key_list` `id`-grouping `sqlite/introspect.rs:261-340`; DDL grouping `schema_ddl.rs:66-82`; `ForeignKey{from_columns,to_columns}` `domain/schema.rs:80-98`. PG live lane unproven. |
| schema-triggers-runtime | Previously COMPLETE (VERIFICATION header) | Implemented on source: PG lifecycle + UI; focused/live verification pending | EVIDENCE_GAP | Trigger lifecycle now has PG `ALTER TABLE … ENABLE/DISABLE TRIGGER`, capability gating for non-PG providers, `tgenabled` mapping and internal-trigger filtering; native trigger row uses `enabled`. Evidence remains pending in `schema-triggers-runtime/VERIFICATION.md`. |
| schema-regression | REVIEW + self-declared EVIDENCE_GAP (2026-09-14 correction) | Accurate | SUPPORTS | Correction banner `VERIFICATION.md:5-9` honestly downgrades PASS claims; ignored-test inventory matches HEAD counts (pg_integration 28 tests/29 ignores, ssh_backup 1/1 ignored). |
| sqlite-view-column-introspection | Verified via unit + integration test | Supported; tests run live today | SUPPORTS | `sqlite/introspect.rs:16-19` extends object_names with views before `introspect_columns`; unit test `introspection_populates_columns_for_sqlite_views` `sqlite/introspect.rs:881`; integration `introspect_views` `tests/integration.rs:694` PASS today. Minor: VERIFICATION cites stale test name `introspect_view_columns_populated` (renamed). |
| ddl-normalization | REVIEW | REVIEW — view DDL + dialect quoting + trigger DDL builders exist; React DDL-editor claims obsolete | EVIDENCE_GAP | View DDL `schema_service.rs:150-152` + test `get_table_ddl_view` `schema_service_tests.rs:335`; dialect-qualified naming `schema_ddl.rs:222-228`; `build_create_trigger`/`build_drop_trigger` `ddl_builder.rs:174,194`; trigger DDL tests `schema_service_tests.rs:533-589`. `DdlOperation::enableTrigger/disableTrigger` + en/ja keys = archived React; native DDL surface is read-only preview, Apply disabled `table_ddl_surface_view.rs:174`. |
| core-safety-hardening | RUNTIME_VERIFY (corrected; live PG/SSH lanes retracted) | Source + unit level supported; spot-checks pass | SUPPORTS | Splitter dollar/comment/quote-aware `domain/safety.rs:133,246`; `DO/CALL/EXECUTE`→Destructive `safety.rs:75`; explain wrapped-classification policy `query_service.rs:280-287`; PG+SQLite batch rollback awaited `postgres/connector.rs:432,495,575`, `sqlite/actor.rs:293-312`; SQLite interrupt+ack `sqlite/actor.rs:106-124`; `SshTunnelConfig.password` `#[serde(skip_serializing)]` `domain/connection.rs:66` + regression test `connection.rs:635`; `StrictHostKeyChecking=no` absent repo-wide; `expect_affected_rows` `table_mutation_execution.rs:132`; backup hydrates SSH `backup_service.rs:119-128`. |
| mysql-transaction-read-statement-correctness | Fixed + unit-tested | Supported — fix present verbatim, unit tests PASS today | SUPPORTS | `mysql/connector.rs:224-238` `execute_batch` delegates to `execute_transaction` and sums `Affected`; `execute_transaction` honors `read_statements` via `fetch_all` `connector.rs:276-283`; 13 mysql lib tests PASS today. Live MySQL path only via `mysql_fixture_matrix.rs` (skips w/o `mysql://` DATABASE_URL — not `#[ignore]`d). |
| sqlserver-provider | RUNTIME_VERIFY; explicitly gated on live fixture | RUNTIME_VERIFY — adapter complete at source level; zero live evidence | EVIDENCE_GAP | `DriverType::SqlServer` `domain/connection.rs:45`; factory registered `infrastructure/connector.rs:78-94,170-171`; connector 477 LOC + dialect `sqlserver/connector.rs`; full catalog loaders `sqlserver/introspect.rs:75-368`; tiberius dep `infrastructure/Cargo.toml:18`. All 4 `sqlserver_integration.rs` tests `#[ignore]`d (need `SQLSERVER_URL`); conformance skips SqlServer `provider_contract_conformance.rs:285`. Doc honestly states the block — code matches, evidence missing. |

## 3. UI findings

- **P1 — Table-trigger enable/disable was absent on audit HEAD; implementation now exists in source.** PostgreSQL typed mutation generation uses `ALTER TABLE ... ENABLE|DISABLE TRIGGER` with dialect quoting; SQLite/MySQL/SQL Server are gated with explicit unsupported reasons. Native Explorer exposes the capability-gated action and routes it through the schema mutation preview/apply flow. Focused checks and live PG evidence remain pending.
- **P2 — `UiTriggerSummary.enabled` was dead data on audit HEAD; now rendered.** Trigger rows display the introspected Enabled/Disabled state, and the available action is the inverse state.
- **P2 — No hardcoded colors on schema surfaces (PASS).** All `schema_workbench*.rs`, `table_relations*`, `table_indexes*`, `table_ddl*`, `schema_object*` files use `DbProTheme` tokens exclusively (`grep 'Color32|Rgba|hex'` = 0 hits); e.g. `schema_workbench_form.rs:15` carries `theme: DbProTheme`.
- No unwrap/expect on data paths in schema surfaces — the only `expect` (`table_indexes_surface_view.rs:200`) is inside a `#[test]` module.

## 4. Feature findings

- **P1 — No column type/nullable/default ALTER path anywhere.** S1 scope claims type change, nullable toggle, default-value mutation via one batch. On HEAD, `ObjectDefinition::Column` supports only `Create`/`Drop`/`Rename` (`object_mutation_builders.rs:65-93`; `_ => Err(unsupported)` at :94). No `ALTER COLUMN ... TYPE`/`SET/DROP NOT NULL`/`SET/DROP DEFAULT` builder exists in `ddl_builder.rs`. The workbench Column form offers only Create/Drop/Rename actions (`schema_workbench_form.rs:131-152`, `schema_workbench_mutation.rs:74-87`). The S1 "combined mutation" scope item has no implementation path on HEAD — it was React-era.
- **P1 — All live-PostgreSQL evidence is `#[ignore]`d.** `pg_integration.rs`: 28 tests, 29 `#[ignore]` attributes; `ssh_backup_runtime_verification.rs`: 1/1 ignored. Every "PG PASS" lane in S1–S7 evidence reduces to ignored tests; S7's own correction already records this.
- **P1 — S4 claimed COMPLETE rested on deleted React surface at audit HEAD; source implementation now exists, but verification is pending.** See the S4 source-level status and remaining evidence in `schema-triggers-runtime/VERIFICATION.md`.
- **P2 — `sqlite-view-column-introspection/VERIFICATION.md` cites test `introspect_view_columns_populated`, renamed to `introspection_populates_columns_for_sqlite_views`** (`sqlite/introspect.rs:881`). Stale name only; behavior verified.
- **P2 — `sqlserver` has no non-ignored test at all.** 3 connector unit tests + 2 mapper tests exist; the 4 integration tests are all `#[ignore]`d; `provider_contract_conformance.rs:285` returns `None` for SqlServer. Acceptable per the plan's own gate, but means zero automated behavioral evidence.
- **P2 — `schema_ddl.rs` keeps a private `quote_identifier` (`:251-254`) instead of the dialect trait used by `ddl_builder.rs`.** Functionally correct for PG/SQLite today (both double-quote), but a second convention; will silently emit `"…"` quoting if a non-ANSI dialect (e.g. MySQL backticks) ever routes through `get_table_ddl`.

## 5. Captures taken

**None — capture mechanism failed twice, not retried further.** Two attempts at 1280x800 (`DB_PRO_CAPTURE_TABLE_INDEXES`, `DB_PRO_CAPTURE_TABLE_FOREIGN_KEYS`, settle=30 on second try) against `./target/debug/db-pro-native` both ran 100+s with the process alive but never produced a PNG — the `egui::Event::Screenshot` reply apparently never arrives in this session (two other `db-pro-native` release instances from sibling agents were running concurrently on the same host). Evidence note, per protocol: captures could not be produced, so UI claims remain source-level only.

## 6. Missing evidence to reach COMPLETED (per plan)

- schema-columns-runtime: live PG combined/failing-mutation proof; live SQLite capability-gating proof; native column-alter path exists first (currently absent); native captures at 3 sizes.
- schema-indexes-runtime: live PG create/unique/composite/drop; UI create→refresh round-trip on native.
- schema-relations-runtime: live PG composite-FK introspection + reconstructed DDL proof; native relations-surface captures.
- schema-triggers-runtime: focused checks/live PG trigger lifecycle including disabled state and native captures. The source implementation is now present.
- schema-regression: the live PG half (un-ignore or fixture-run `pg_integration`), CHECK/unique-constraint dedicated evidence, native schema-traversal artifact — all already listed in its own correction.
- sqlite-view-column-introspection: none for backend; a native capture showing view columns in the explorer would close runtime lane.
- ddl-normalization: native-UI evidence for view DDL display + trigger DDL preview; decide whether React DDL-editor ops need a native equivalent (enable/disable trigger op).
- core-safety-hardening: live PG + SSH fixture runs (`pg_integration`, `ssh_backup_runtime_verification`) with retrievable output; already self-gated.
- mysql-transaction-read-statement-correctness: a `DATABASE_URL=mysql://` run of `mysql_fixture_matrix.rs` proving rollback on mid-batch failure.
- sqlserver-provider: live SQL Server fixture (`SQLSERVER_URL`) run of the 4 ignored tests + native captures; exactly what its PLAN states.

## 7. P0/P1/P2 totals

- P0: 0
- P1: 4 (missing column-alter path; trigger enable/disable absent; all-PG-evidence-ignored [counted once across plans]; S4 COMPLETE claim unsupported)
- P2: 4 (dead `enabled` field; stale test name in VERIFICATION; zero non-ignored sqlserver tests; private `quote_identifier` second convention)

## 8. Tổng kết bằng tiếng Việt

- Phần lõi backend của S1–S5 đúng như tài liệu: `execute_batch` có transaction trên PG/SQLite/MySQL/SQL Server, composite FK được group đúng (PG `conname`, SQLite `PRAGMA id`), introspection trigger đầy đủ; các test SQLite chạy thật hôm nay đều PASS.
- Mọi bằng chứng "PostgreSQL PASS" đều nằm trong test `#[ignore]` — chưa có bằng chứng runtime PG nào có thể truy xuất, đúng như S7 tự công nhận (EVIDENCE_GAP).
- S4 (`schema-triggers-runtime`) tuyên bố COMPLETE là STALE: tính năng enable/disable trigger chỉ tồn tại trong React đã archive; trên HEAD không có bất kỳ dòng SQL `ENABLE/DISABLE TRIGGER` nào, UI native chỉ có View/Drop, field `enabled` là dữ liệu chết.
- S1 thiếu hẳn đường ALTER column (đổi kiểu/nullable/default) — builder chỉ có Create/Drop/Rename; phạm vi "combined mutation" là di sản React.
- `sqlserver-provider` hoàn chỉnh ở mức code nhưng không có một test nào không-ignore — đúng như plan tự ghi, vẫn EVIDENCE_GAP chờ live fixture.
- Capture UI thất bại 2 lần (không sinh PNG) nên mọi khẳng định về UI chỉ dừng ở mức source; UI schema surfaces sạch token, không hardcode màu.
