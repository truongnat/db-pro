# Agent evidence — History research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · research lane |
| Issue(s) | n/a (no issue supplied for the research request) |
| Task state | Done |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | n/a (documentation research; branch/PR status not asserted) |
| Scope interpretation | Source-grounded assessment of the History activity and adjacent query-execution history paths: local sidebar strings, structured UI records, Quick Open, and SQLite meta-store history. |
| Out of scope | Code changes, feature-plan creation, UI/runtime/provider validation, and claiming DataGrip/DBeaver parity. |

## 2. Progress checkpoint

- Current HEAD: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Completed acceptance rows: [x] trace History rail/sidebar; [x] distinguish all source-visible history stores and replay/open paths; [x] compare current product goals and official history workflows; [x] record severity, limits, and recommended next work.
- Remaining acceptance rows: none for this research handoff.
- Findings / risks: P1 — UTF-8 preview truncation can panic at a non-character boundary. P2 — sidebar history replaces active SQL without prompt/context restoration; Quick Open selects oldest rather than newest entries; cancelled executions display as success; history sources, retention and product placement diverge.
- Tests already run: none. No build/test command, native UI traversal, restart persistence check, PostgreSQL runtime, or SQLite runtime was run.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | No commit created by this research task. |
| File / surface inventory | `docs/design/history/history-baseline.md` — source paths, state stores, and risks; `docs/design/history/history-feature-research.md` — product intent, external workflow references, proposed features and roadmap; `docs/design/history/AGENT_EVIDENCE.md` — canonical research handoff. |
| Acceptance mapping | Current behavior/risk claims → SHA-qualified anchors in baseline; product goals/workflow comparison/recommendations → feature research §§2–6; evidence limitations and next decisions → this file. |
| Commands and counts | `git rev-parse HEAD` → `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, exit 0. No tests/build commands run. |
| CI run IDs / status | Not run (documentation research). |
| Known limitations | Source-only; no independent reviewer or runtime confirmation. Unicode panic and severity are source-derived, not runtime reproduced. Product docs differ on history storage ownership. |
| Migrations / config implications | None executed. Recommendations may require a meta-store schema migration after storage ownership/retention decisions. |
| Out-of-scope changes | No source code, product goal, status table, lifecycle plan, or unrelated documentation changed. |

## 4. Review outcome

n/a — research handoff, not an independent code review or PR verdict. The P1/P2 classifications describe source-observed risks at the stated SHA, not provider/runtime test results.

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Source URLs / references: `docs/goals/goal-phase-g-productivity.md`; `docs/goals/goal-full-product.md`; `docs/design/query-activity/query-activity-baseline.md`; `crates/ui/src/activity_bar_view.rs`; `crates/ui/src/sidebar_view.rs`; `crates/ui/src/sidebar_activities_view.rs`; `crates/ui/src/sidebar_queries_surface_view.rs`; `crates/ui/src/sidebar_query_library_view.rs`; `crates/ui/src/query_editor_state.rs`; `crates/ui/src/query_execution_actions.rs`; `crates/ui/src/query_execution_events.rs`; `crates/ui/src/query_result_events.rs`; `crates/ui/src/query_failure_events.rs`; `crates/ui/src/query_multi_result_events.rs`; `crates/ui/src/query_history_events.rs`; `crates/ui/src/runtime_query_types.rs`; `crates/ui/src/query_output_actions_view.rs`; `crates/ui/src/query_documents.rs`; `crates/ui/src/query_state.rs`; `crates/ui/src/editor/buffer.rs`; `crates/ui/src/palette_search_view.rs`; `crates/ui/src/palette_actions.rs`; `crates/ui/src/app_lifecycle.rs`; `crates/ui/src/app_storage.rs`; `crates/runtime/src/worker.rs`; `crates/runtime/src/api.rs`; `crates/core/src/application/query_service.rs`; `crates/core/src/ports/query_history_repository.rs`; `crates/core/src/domain/history.rs`; `crates/infrastructure/src/meta/query_history_repo.rs`; `crates/infrastructure/src/meta/schema.rs`; [DBeaver Query Manager](https://dbeaver.com/docs/dbeaver/Query-Manager/); [DataGrip Find recent queries and files](https://www.jetbrains.com/help/datagrip/find-recent-queries-and-files.html).
- Factual findings: Source-observed surfaces, record fields, retention limits, action routing, query-service saves and API reads are anchored in `history-baseline.md` at the baseline SHA above.
- Inference: Severity, canonical-store recommendation and milestone order are recommendations, not accepted feature decisions. Official docs are workflow references, not requirements for full parity.
- Decision / recommendation: First make opening/replaying SQL safe and fix the UTF-8 preview panic, status rendering and Quick Open ordering. Then resolve the Phase G vs Full Product Goal persistence mismatch; recommended direction is one meta-store-backed execution-history record with eframe limited to UI-local state.
- Unresolved questions: Canonical record migration from current local dual state; failure/cancellation/partial-script semantics; SQL literal privacy; retention, clear/export; replay behavior when the source connection/schema no longer exists; whether History moves under Queries or becomes a separately scoped cross-feature timeline; independent PostgreSQL/SQLite and restart evidence.
- Downstream tasks activated: none; no issue or implementation plan was created.

## 6. Tổng kết (Vietnamese summary)

Đã hoàn tất source/product research cho History tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. Có ba đường lưu history và các surface không dùng chung records; sidebar thay SQL đang soạn, Quick Open lấy nhầm entries cũ, Cancelled hiển thị như OK, và preview UTF-8 có thể panic. Chưa có runtime/provider evidence. Bước tiếp theo là sửa an toàn Open/Replay và lỗi preview, sau đó chốt canonical store, retention/privacy và placement theo Queries goal.
