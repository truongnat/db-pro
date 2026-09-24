# Agent evidence — Transfers feature research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · product research lane |
| Issue(s) | n/a — user requested “tiếp tới transfers” |
| Task state | Done — source baseline and feature research recorded |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | `docs/files-tab-design` / no PR |
| Scope interpretation | Source-grounded review of the native Transfers activity, transfer core/adapters, query export, backup/restore, user workflow, and IA. |
| Out of scope | Code changes, UI implementation, provider runtime qualification, phase-goal/status transitions, DB-to-DB implementation, and independent external approval. |

## 2. Progress checkpoint

- Current HEAD: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Completed acceptance rows: [x] traced native Transfer surface and call paths; [x] inventoried core transfer adapters, query export, and backup/restore; [x] compared official DBeaver and DataGrip workflows; [x] assessed IA, provider contract, safety boundaries, and stale documentation; [x] wrote source baseline, feature assessment, and evidence handoff.
- Remaining acceptance rows: none for this research task. Product implementation and source/document correction remain follow-up work; no issue was created.
- Findings / risks (all source claims below refer to the Baseline SHA):
  - **P1 product gap, inherited/unimplemented:** real file-to-table import is not wired to a live PostgreSQL or SQLite target (`crates/core/src/application/delimited_transfer.rs:32-93`; transfer implementations at the SHA are enumerated in `transfers-baseline.md`).
  - **P1 product gap, inherited/unimplemented:** native quick export covers loaded query-result rows; there is no integrated streaming live-table/full-query export in Transfers (`crates/ui/src/query_dialogs_view.rs:100-145`; `crates/core/src/application/transfer_service.rs:8-33`).
  - **P2 integration safety risk, source-derived and not runtime-reproduced:** transfer file-writer failure/cancellation cleanup unconditionally removes the final destination and can remove a pre-existing file (`crates/core/src/application/delimited_transfer.rs:104-164`; `crates/core/src/application/transfer_service.rs:115-160`). Current UI usage is the synthetic harness; do not wire these adapters to user destinations before fixing ownership/overwrite semantics.
  - **P2:** transfer jobs are transient harness state; no durable transfer history or integrated row/byte progress/cancel flow (`crates/ui/src/transfer_state.rs:1-6`; `crates/ui/src/transfer_activity_surface_view.rs:262-297`).
  - **P2:** the generic loop has a row ceiling but no byte budget; XLSX buffers all rows (`crates/core/src/application/transfer_service.rs:8-33`; `crates/core/src/application/json_excel_transfer.rs:152-219`).
  - **P2:** native Query export displays PostgreSQL `COPY` output without a visible provider gate (`crates/ui/src/query_dialog_surface_view.rs:95-107`; `crates/ui/src/result_grid_export.rs:97-110`).
  - **P2:** Phase C goal and capability documents contain stale source assertions; exact drift is catalogued in `transfers-baseline.md`.
- Tests already run: none. No build, unit/integration test, UI launch, or provider runtime scenario was run. Source tests were inspected but not executed.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | none — documentation work is uncommitted |
| File / surface inventory | `docs/design/transfers/transfers-baseline.md` — source inventory, provider state, documentation drift, and safety gate; `docs/design/transfers/transfers-feature-research.md` — product model, IA recommendation, workflow/provider contract, reference-product comparison, prioritized gaps; `docs/design/transfers/AGENT_EVIDENCE.md` — this handoff. |
| Acceptance mapping | “Continue with Transfers” → all three research artifacts; top-level placement → `transfers-feature-research.md` decision and activity structure; exact source evidence → baseline SHA and anchored inventory; provider separation → independent PostgreSQL/SQLite matrix; competitor patterns → official docs linked in feature research. |
| Commands and counts | No tests/build commands executed. Source read/search only. |
| CI run IDs / status | not run |
| Known limitations | No runtime/provider evidence; P2 file destination cleanup risk is source-derived, not reproduced; Phase C implementation details still need feature-owner review. |
| Migrations / config implications | none — no source or persistent data changed |
| Out-of-scope changes | No code, capability matrix, release limitation, product goal, plan status, or UI changes. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (source baseline; docs are uncommitted) |
| Verdict | n/a — research handoff, not independent code review or implementation approval |
| P0 / P1 / P2 counts | introduced by this SHA: 0 / 0 / 0; inherited or unimplemented at baseline: 0 / 2 / 5. The adapter cleanup risk is source-derived and remains unverified at runtime. |
| Findings | See §2 and `transfers-feature-research.md` “Prioritized open work”; no code verdict is asserted. |
| CI disposition | not run — documentation-only research |
| Next task(s) unblocked | Update the Phase C “current implementation” and capability-matrix rows against this baseline before using them as implementation evidence; then plan the import/export/job outcomes. No issue activated in this task. |

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Repository references, all at exact SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`:
  - `crates/ui/src/activity_bar_view.rs:85-96`, `palette_actions.rs:19-20`, `transfer_activity_surface_view.rs:27-36,50-94,162-297`, `transfer_activity_view.rs:21-93`, `transfer_harness_view.rs:56-464`, `transfer_state.rs:1-6`.
  - `crates/core/src/domain/transfer.rs:114-143`, `application/transfer_service.rs:8-223`, `application/delimited_transfer.rs:32-164`, `application/json_excel_transfer.rs:12-219`, `application/db_transfer.rs:24-212`.
  - `crates/ui/src/query_dialog_surface_view.rs:95-107`, `result_grid_export.rs:43-111`, `query_dialogs_view.rs:100-145`; `crates/core/src/application/export_service.rs:46-108`; `crates/runtime/src/api.rs:848-865`.
  - `crates/ui/src/settings_backup_view.rs:20-111`, `settings_view.rs:123-184`, `management_events.rs:12-18`; `crates/runtime/src/worker.rs:1644-1745`; `crates/core/src/application/backup_service.rs:39-109`; `crates/infrastructure/src/backup/pg_dump.rs:77-137`, `sqlite_backup.rs:87-160`.
  - Product docs: `docs/goals/goal-phase-c-transfer.md:10,101-117,138-156,221-265,648-683`; `docs/goals/goal-full-product.md:125-138,221-258`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:257-279`; `docs/release/known-limitations.md:203-215,245-257,317-329`.
- External references read 2026-09-24:
  - https://dbeaver.com/docs/dbeaver/Data-transfer/
  - https://dbeaver.com/docs/dbeaver/Backup-Restore/
  - https://www.jetbrains.com/help/datagrip/import-data.html
- Factual findings: the native rail contains a Transfers activity; its current controls exercise synthetic fixtures/mocks; the core has generic row-batch/file adapters but no inspected live database transfer adapter; native query export is a separate loaded-result quick action; backup/restore has a Settings/runtime path; its PostgreSQL and SQLite mechanisms and constraints differ. Details and anchors are in `transfers-baseline.md`.
- Inference: retaining a first-class Transfer rail and moving to task-oriented Import/Export/Backup/Restore/Jobs is the recommended IA, grounded in the master product goal and the current cross-cutting user intents. Vendor workflows are reference patterns only. The destination cleanup risk is source-derived; runtime impact is not established.
- Decision / recommendation: retain the rail; replace the synthetic harness as the primary content; keep query export as a quick action; reuse the existing backup service; implement provider-specific streaming imports/exports, safe destination behavior, honest background progress/cancellation, and durable redacted jobs. Keep Phase F compare/migration separate from Phase C delivery.
- Unresolved questions: final activity navigation (internal routes vs dedicated views); product priority/format commitments for CSV, JSON/JSONL, and XLSX; whether the existing developer harness remains in a developer-only surface. The Phase C goal already provides the initial workflow contract.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã khảo sát Transfers tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, lập baseline và đề xuất giữ Transfers trên activity rail nhưng thay harness giả lập bằng luồng Import/Export/Backup/Restore/Jobs. Phát hiện tài liệu Phase C và capability matrix đã lỗi thời; core có nền tảng transfer nhưng chưa nối với cơ sở dữ liệu thật. Cần xử lý an toàn ghi file trước khi dùng adapter với đường dẫn người dùng, và vẫn thiếu bằng chứng runtime riêng cho PostgreSQL/SQLite. Không sửa code, không chạy test hoặc UI/runtime trong lượt này.
