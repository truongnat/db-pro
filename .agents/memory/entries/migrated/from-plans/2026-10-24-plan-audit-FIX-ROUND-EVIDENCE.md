---
schema_version: 1
id: "MEM-AUDIT-FIX-ROUND-EVIDENCE-d3ba7443909a"
title: "Plan archive: Audit Fix Round Evidence"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, audit-FIX-ROUND-EVIDENCE]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d3ba7443909a62a8582fb1deaf033de3c230ff6741d5e38eec88281e8f5b977b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/_audit-2026-10-13/FIX-ROUND-EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ audit slice `docs/plans/_audit-2026-10-13/FIX-ROUND-EVIDENCE.md`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

# Audit fix round — evidence handoff

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main integration + 7 fix slices |
| Task state | Review |
| Baseline SHA | `d73148cd3126deb97f3153ca52e9147c4b5666f4` |
| Branch / PR | `main` / no PR; local changes are uncommitted |
| Scope | Resolve code-actionable P1s and selected P2s from the 2026-10-13 six-slice audit; reconcile plan evidence without changing lifecycle states |
| Out of scope | SQL Server server setup, SSH fixtures, manual mouse/keyboard/IME verification, full-height captures beyond host display capability |

The baseline SHA is the HEAD that the fix round started from. Implemented changes remain uncommitted, so this SHA does not identify the final working tree; do not treat this as a clean-SHA handoff.

## 2. Implemented findings

- **Architecture guard:** allowlist now includes legitimate `DbProApp.dev_tools` and `capture_evidence_seeded`; a full-app UI test was moved out of a state-only module. `bash scripts/check-ui-architecture.sh` passes.
- **Query editor:** `extract_word_prefix` uses UTF-8 character boundaries and has Vietnamese/emoji regressions. Confirmed-hover occurrence highlights use the theme accent and only scan the visible/overscan window.
- **Design tokens:** chart series colors now come from `DbProTheme`; alert, hover-card, and toast shadows/backdrops use semantic theme values. No hardcoded chart palette remains.
- **Column ALTER:** PostgreSQL type/nullability/default ALTER; MySQL full-definition MODIFY; SQL Server type/nullability (default changes rejected); SQLite gated because ALTER requires table rebuild. Structure action enters the existing workbench + confirmation path. MySQL data-type/default combinations may be provider/version-specific; local MariaDB rejected `TEXT DEFAULT`.
- **Trigger lifecycle:** PostgreSQL Enable/Disable routed through the typed mutation/workbench path; `tgenabled` state is introspected. SQLite, MySQL, and SQL Server are explicitly unsupported and produce no SQL.
- **Cleanup:** dead `compact_*` helpers and `set_target_table` are absent; `remove_insert` has a real caller; explorer folders use iterators without materializing a pre-open `Vec`. Some of these removals were present after a cleanup worker hit a rate limit; no cleanup handoff was produced by that worker.
- **Compare presentation:** explicit `SPACE_XS` spacing separates diff badges/headings and bullets/object names; captured after the change.
- **Docs:** `STATUS.md`, `FEATURE_AUDIT_2026-09-23.md`, and 13 plan verification/checklist files were reconciled. S1/S4 remain `RUNTIME_VERIFY`; no plan was promoted to `COMPLETED`.

Per-slice handoffs: `fix-arch-guard.md`, `fix-query-editor.md`, `fix-chart-tokens.md`, `fix-column-alter.md`, `fix-trigger-lifecycle.md`, `fix-docs-reconcile.md`.

## 3. Commands and results

- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo build -p db-pro-native --features capture` — PASS.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `bash scripts/check-ui-architecture.sh` — PASS.
- `DATABASE_URL=postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture cargo test --workspace --locked` — **1819 passed / 0 failed / 41 ignored**, exit 0; 28 ignored PG integration tests were enabled and passed against PostgreSQL 18.6.
- `DATABASE_URL=mysql://dbpro:dbpro_test@localhost:3306/dbpro_fixture cargo test -p db-pro-infrastructure --test mysql_integration -- --ignored` — **8 passed / 0 failed**, exit 0.
- `cargo test -p db-pro-core trigger` — **9 passed / 0 failed**.
- `cargo test -p db-pro-core column_alter` — **5 passed / 0 failed**.
- UI focused tests: trigger toggle **2 passed**, column workbench plan **1 passed**, chart contrast **1 passed**, UTF-8 regression **1 passed**, hover occurrences **3 passed**.
- `cargo clippy --workspace --all-targets -- -D warnings` — **FAILED, 65 diagnostics**. The emitted diagnostics include `double_must_use` on `async_trait`-expanded repository traits and `result_large_err` in untouched `crates/core/src/application/table_mutation_execution.rs:36`. No changed-file match appeared in the inspected clippy output. No lint was suppressed.
- One provider-suite invocation initially omitted `DATABASE_URL` and failed fast (28 test setup failures); rerun with the fixture URL above passed 28/28. The later MySQL invocation passed 8/8.

## 4. Runtime evidence

- Capture matrix: `CAPTURE-EVIDENCE.md` + `captures/`: 102 original PNGs (34 surfaces × 3 requested sizes), plus refreshed `compare-spacing_1280x800.png` and `table-structure-alter_1280x800.png`.
- Effective framebuffer sizes: 2560×1600 for 1280×800; 2880×1676 and 3840×1676 for requests 1440×900 / 1920×1080 (macOS clamps height to 838 logical px).
- Live PG trigger SQL smoke: within a transaction, disabling `products_updated_at` changed `tgenabled` O→D; enabling restored O; transaction rolled back.
- Live PG column ALTER smoke: temporary table combined TYPE + DROP NOT NULL + SET DEFAULT succeeded and verified through `information_schema`; transaction rolled back.
- Live MySQL column ALTER smoke: temporary VARCHAR column MODIFY (type/nullability/default) succeeded. MySQL TEXT with a constant DEFAULT failed on local MariaDB 26.7.0, documenting provider/version constraints.
- **Not verified:** clicking trigger context action through native UI→runtime→DB→refreshed UI; hover/keyboard/drag/IME interaction; full requested 900/1080 logical heights; SQL Server and SSH runtime.

## 5. Remaining work / blockers

- Two query large-schema P2s remain: result clone in the output dock and synchronous `SchemaSymbolIndex::build`. Profile before redesign/async offload; no frame-time evidence was collected.
- Manual native interaction pass remains necessary for trigger actions, confirmation, keyboard/focus, hover, scroll/drag, and IME.
- SQL Server fixture is unavailable (port 1433 closed); SSH backup fixtures are unavailable (`DB_PRO_SSH_*` not set).
- Full-workspace Clippy gate remains red with the 65 diagnostics above. It must be fixed or dispositioned separately; this audit fix round did not suppress or hide them.
- No independent external review was performed; no commit or PR was created.

## 6. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent reviewer and changes are uncommitted |
| Verdict | n/a |
| P0 / P1 / P2 | Introduced: no new P0/P1 observed in focused/full test evidence; existing P2 performance and runtime verification items remain as listed |
| CI disposition | not run |
| Next tasks | resolve workspace clippy diagnostics; perform the native interactive/provider runtime matrix; measure query large-schema P2s before changing architecture |

## 7. Tổng kết bằng tiếng Việt

- Đã xử lý các P1 về architecture guard, UTF-8 panic, màu chart, column ALTER và trigger Enable/Disable; có regression tests và evidence PG/MySQL.
- `cargo fmt`, workspace check/test, native release build và architecture guard PASS; live PG 28/28, MySQL 8/8 PASS. Clippy workspace vẫn FAIL với 65 diagnostics, không bị suppress.
- Còn manual native interaction, SQL Server/SSH fixture, full-height screenshot và hai P2 query large-schema cần profiling. Không đổi trạng thái plan sang `COMPLETED`; thay đổi còn uncommitted và chưa có external review.

