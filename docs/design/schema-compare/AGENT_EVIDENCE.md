# Agent evidence — Schema Compare research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · research lane |
| Issue(s) | n/a (research task; #199 and #200 are related closed issues, not task assignments) |
| Task state | Done |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | n/a (documentation research; branch/PR status not asserted) |
| Scope interpretation | Source-grounded assessment of the current Schema Compare, migration preview/apply, and embedded Data Compare flows; compare product intent with official product documentation. |
| Out of scope | Code changes, implementation planning as an approved feature, UI/runtime/provider verification, and claiming product parity. |

## 2. Progress checkpoint

- Current HEAD: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Completed acceptance rows: [x] map source behavior and risks; [x] document feature gaps and recommendations; [x] include official workflow references; [x] identify provider/runtime verification gaps.
- Remaining acceptance rows: none for this research handoff.
- Findings / risks: P1 — CREATE TABLE/ADD COLUMN placeholder DDL can reach Apply; apply can target a different active connection/provider than the plan. P1/P2 — structural “identical” covers only a subset of schema metadata. P2 — stale snapshot/plan and out-of-order Data Compare results.
- Tests already run: none; documentation-only research. No build, test suite, UI runtime, PostgreSQL runtime, or SQLite runtime was run.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | No commit created by this research task. |
| File / surface inventory | `docs/design/schema-compare/schema-compare-baseline.md` — source behavior and risk evidence; `docs/design/schema-compare/schema-compare-feature-research.md` — product comparison, gaps, feature proposals and roadmap; `docs/design/schema-compare/AGENT_EVIDENCE.md` — canonical research handoff. |
| Acceptance mapping | Source behavior and risk claims → baseline report with SHA-qualified line anchors; official workflow references and recommendations → feature research report §2–§6; limitations and handoff state → this file. |
| Commands and counts | `git rev-parse HEAD` → `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, exit 0. No tests/build commands run. |
| CI run IDs / status | Not run (documentation research). |
| Known limitations | Source inspection only; no independent reviewer or runtime/provider evidence. Severity describes risks observed in source, not runtime-confirmed failures. |
| Migrations / config implications | None. |
| Out-of-scope changes | No code, active feature plan, provider behavior, archived frontend, or unrelated documentation changed. |

## 4. Review outcome

n/a — research handoff, not an independent code review or PR verdict. Reported P1/P2 findings are source-observed risks at the stated SHA; no claim is made that they are runtime-confirmed.

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Source URLs / references: `docs/goals/goal-phase-f-compare-migration.md`; `docs/plans/STATUS.md`; `docs/plans/FEATURE_LIFECYCLE.md`; `crates/ui/src/activity_bar_view.rs`; `crates/ui/src/palette_actions.rs`; `crates/ui/src/app_lifecycle.rs`; `crates/ui/src/state/schema_workspace_state.rs`; `crates/ui/src/state/schema_compare_state.rs`; `crates/ui/src/state/schema_compare.rs`; `crates/ui/src/views/schema_compare_view.rs`; `crates/ui/src/views/sidebar_view.rs`; `crates/ui/src/views/workspace_view.rs`; `crates/ui/src/state/explorer_navigation.rs`; `crates/ui/src/state/query_session.rs`; `crates/ui/src/state/ddl_events.rs`; `crates/ui/src/state/operation_events.rs`; `crates/ui/src/state/management_events.rs`; `crates/ui/src/runtime_protocol.rs`; `crates/ui/src/event_router.rs`; `crates/core/src/application/migration_planner.rs`; `crates/core/src/application/data_diff.rs`; [DBeaver Structure and Data Compare](https://dbeaver.com/docs/team-edition/desktop/Structure-and-Data-Compare/); [DBeaver Data Compare](https://dbeaver.com/docs/dbeaver/Data-compare/); [DataGrip Schema Comparison and Migration](https://www.jetbrains.com/help/datagrip/schema-comparison-and-migration.html); [DataGrip Compare Data](https://www.jetbrains.com/help/datagrip/compare-data.html); related closed issues [#199](issue://199) and [#200](issue://200).
- Factual findings: Current snapshot/diff and migration behavior, UI command path, Data Compare sampling/results, and observed tests are detailed with source line anchors in `schema-compare-baseline.md`; all source behavior claims refer to the baseline SHA above.
- Inference: Severity and proposed sequencing are analysis recommendations, not runtime findings or accepted product decisions. DBeaver/DataGrip are workflow references, not proof of DB Pro behavior or required parity.
- Decision / recommendation: Prioritize F1 safety: prevent incomplete placeholder DDL from reaching ExecuteDdl and bind/validate the intended target and provider. Then make diff coverage explicit and invalidate stale state. Keep Data Compare scope separate from migration safety.
- Unresolved questions: Snapshot semantics (historical baseline vs live Origin), target selection, snapshot persistence, authoritative source for full object definitions, metadata coverage required to say “identical,” Data Compare sync/export scope, and PostgreSQL/SQLite-specific capability and atomicity strategies; see feature research §6.
- Downstream tasks activated: none; no issue or implementation plan was created.

## 6. Tổng kết (Vietnamese summary)

Đã hoàn tất nghiên cứu Schema Compare tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, gồm baseline hành vi/rủi ro, đối chiếu workflow chính thức và đề xuất ưu tiên. P1 chính là placeholder CREATE TABLE/ADD COLUMN có thể được apply và migration plan có thể chạy trên connection/provider khác. Chưa có runtime evidence cho PostgreSQL, SQLite hoặc UI; các bước tiếp theo cần xử lý F1 rồi xác minh từng provider độc lập.
