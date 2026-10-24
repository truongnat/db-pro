---
schema_version: 1
id: "MEM-SRC-1387eafc517060f3"
title: "Source mirror: docs/plans/_audit-2026-10-13/INDEX.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/_audit-2026-10-13/INDEX.md"
source_sha256: "1387eafc517060f321492a9c6b9eda0d8deac677ad8e47b832974b232df076a2"
mirror_body_sha256: "1387eafc517060f321492a9c6b9eda0d8deac677ad8e47b832974b232df076a2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Feature & UI Audit — 2026-10-13

Audited HEAD of `main` by 6 parallel slices against `docs/plans/active/` (65 plans) +
`docs/plans/completed/`. Method: plan-doc claims verified against code/tests at HEAD;
not a full runtime re-verification.

| # | Slice | Report | Plans | P0 | P1 | P2 |
|---|---|---|---:|---:|---:|---:|
| 01 | Schema / safety / providers | [01-schema-safety-providers.md](01-schema-safety-providers.md) | 10 | 0 | ~3 | — |
| 02 | Query editor / workspace | [02-query-editor-workspace.md](02-query-editor-workspace.md) | 8 | 0 | 1 | 5 |
| 03 | Data grid / table editor | [03-data-grid-table-editor.md](03-data-grid-table-editor.md) | 7 | 0 | 1 | 7 |
| 04 | ER / explorer / sidebar | [04-er-explorer-sidebar.md](04-er-explorer-sidebar.md) | 7 | 0 | 0 | 7 |
| 05 | Design system / components | [05-design-system-components.md](05-design-system-components.md) | 14 | 0 | 2 | 8 |
| 06 | Shell / cross-cutting | [06-shell-crosscutting.md](06-shell-crosscutting.md) | 22 | 0 | 1 | — |

## Headline findings

**P1 remediation status (working tree based on HEAD `d73148cd3126deb97f3153ca52e9147c4b5666f4`):**
- Fixed: architecture allowlist + state-module boundary violation; guard now passes.
- Fixed: UTF-8 word-prefix panic, with multi-byte/emoji regression tests.
- Fixed: chart series colors now flow through `DbProTheme` tokens; production alert/hover/toast shadows use semantic tokens.
- Implemented: provider-aware column ALTER for PostgreSQL/MySQL/SQL Server, SQLite explicitly gated; native Structure → workbench → existing confirmation path.
- Implemented: PostgreSQL trigger Enable/Disable + `tgenabled` state introspection; SQLite/MySQL/SQL Server explicitly gated. Live PG SQL toggle verified; native UI→runtime→DB interaction remains pending.
- Fixed/verified: old grid compact helpers and unused `set_target_table` are absent; explorer folder iterators already avoid pre-expansion Vec clones; compare badge/title spacing repaired and recaptured.
- Final gates: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo build --release --locked -p db-pro-native`, and `bash scripts/check-ui-architecture.sh` all PASS.
- `DATABASE_URL=postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture cargo test --workspace --locked` → 1819 passed / 0 failed / 41 ignored (includes live PG 28); `DATABASE_URL=mysql://dbpro:dbpro_test@localhost:3306/dbpro_fixture cargo test -p db-pro-infrastructure --test mysql_integration -- --ignored` → 8 passed / 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings` FAILS with 65 diagnostics (many `async_trait` `double_must_use`; also `result_large_err` in `table_mutation_execution.rs`, untouched by this fix round).

**Remaining evidence / explicitly open work:**
- P2 query-editor large-schema work: avoid cloning active result in the dock and move `SchemaSymbolIndex::build` off the UI event path only after profiling; current audit provides source evidence but no frame-time measurements.
- Native interactive verification: trigger context action / confirmation / live UI round-trip, hover/drag/scroll/keyboard/IME; requires a human-operated app pass.
- Display-limited captures: 1440×900 and 1920×1080 requested sizes still clamp to 838 logical px.
- Provider fixtures unavailable here: SQL Server and SSH backup suite remain BLOCKED.
- MySQL `TEXT DEFAULT` is rejected by the local MariaDB; supported column type/default combinations remain engine/version-specific and may surface a database error.

**Stale docs / claims corrected:**
- `explorer-tree-perf`: earlier audit report is stale on the clone finding; current
  Views/Functions/Triggers use lazy iterator traversal without materializing a `Vec`.
- STATUS.md S6 "no position persistence" is superseded — `er-diagram-canvas-ux` ships real
  per-connection layout persistence (`diagram/persistence.rs`, key `dbpro.native.er-layouts-v1`).
- FEATURE_AUDIT §5 blank results-grid defect: **FIXED** in `294ab089` / PR #308.
- Two baseline connection-form failures cited across plans **now PASS at HEAD** (slice 03).
- `responsive-layout-primitives` fully landed; docs now annotate the corrected state. `ui-core-audit`
  has 3 of 5 formerly open P1s implemented; its remaining plan audit needs refresh.
- STATUS.md's egui 0.36.2 note corrected; `native-visual-redesign` plan's own Wave-17 note may remain stale.
- `gallery-form-controls/VERIFICATION.md` empty stub was replaced with evidence + explicit open gaps.

**OBSOLETE candidates:**
- `ux-friendliness-audit` + frontend-targeted `rc1-full-product-qa` rows (React app deleted).
- `rs-ui-runtime-integration`: zero rs-ui remnants in code/lockfile; only stale
  `docs/rs-ui-integration-plan.md` (P2). `restore-egui-ui` verified clean.

**Runtime evidence pass (2026-10-13, this run):** rebuilt with `--features capture`;
**102 PNGs, 34 surfaces × 3 sizes**, 0 failures — see [CAPTURE-EVIDENCE.md](CAPTURE-EVIDENCE.md)
and `captures/`. 1440/1920 effective height is 838 (host display clamp).
**Live providers also exercised:** `pg_integration` 28/28 PASS on PostgreSQL 18.6 and
`mysql_integration` 8/8 PASS on live MySQL — the `#[ignore]`d-suite gap for PG/MySQL is
closed on this host. SQL Server (:1433 absent) and SSH fixtures (`DB_PRO_SSH_*` unset)
remain BLOCKED.

**v0.1 gate list** — see slice 06 §6 (10 items incl. fix the red architecture guard,
P0/P1=0 + explicit P2 disposition, native runtime matrix, provider suites, packaging smoke).
