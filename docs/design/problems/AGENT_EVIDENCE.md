# Agent evidence — Problems placement research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · research lane |
| Issue(s) | n/a (no issue supplied for the research request) |
| Task state | Done |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | n/a (documentation research; branch/PR status not asserted) |
| Scope interpretation | Trace the native Problems activity and diagnostics scope, then assess a non-sidebar placement consistent with the user’s direction and the product IA. |
| Out of scope | Source implementation, lifecycle-plan/status changes, design implementation, tests, runtime/provider verification, and vendor parity claims. |

## 2. Progress checkpoint

- Current HEAD: `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (baseline established earlier in this session; no source changes made).
- Completed acceptance rows: [x] trace rail/sidebar, diagnostics sources, filters, navigation, and Quick Fix; [x] distinguish Problems from Messages and Settings support Diagnostics; [x] compare the product rail goal and official IDE workflows; [x] record placement recommendation, risks, and evidence limits.
- Remaining acceptance rows: none for this research handoff.
- Findings / risks: P2 — rail placement conflicts with the target IA (`problems-baseline.md:15-17`); P2 — active-document diagnostics and aggregate Problems have mismatched scope/destination (`problems-baseline.md:28-30`); P2 — lower output state is not a workspace Problems container (`problems-baseline.md:28-30`); P2 — workspace SQL row selection does not jump to its diagnostic line (`problems-baseline.md:24,41`). No P0/P1 issue established.
- Tests already run: none. No build/test command, native UI traversal, session restore, screenshot, PostgreSQL runtime, or SQLite runtime was run.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | No commit created by this research task. |
| File / surface inventory | `docs/design/problems/problems-baseline.md` — source dispatch, aggregation, actions, adjacent diagnostics surfaces, and limits; `docs/design/problems/problems-feature-research.md` — IA assessment, vendor references, proposed interaction contract, and future verification scenarios; `docs/design/problems/AGENT_EVIDENCE.md` — canonical research handoff. |
| Acceptance mapping | Current state and risks → SHA-qualified anchors in baseline; product placement decision and candidate interaction contract → feature research §§1–5; execution status/limits → this handoff. |
| Commands and counts | No commands run in this research task. Baseline SHA was established earlier in the session. No test/build counts to report. |
| CI run IDs / status | Not run (documentation research). |
| Known limitations | Source-only; no visual runtime evidence or independent review. Placement is a recommendation informed by the user’s stated intent and product-goal documents, not an accepted feature plan. |
| Migrations / config implications | None executed. Future cutover must deliberately restore persisted workspace sessions whose activity string is `"Problems"`. |
| Out-of-scope changes | No source code, product goal, status table, lifecycle plan, unrelated docs, or tests changed. |

## 4. Review outcome

n/a — research handoff, not an independent code review or PR verdict. P2 classifications describe product/UX placement and architecture risks inferred from the cited source at the stated SHA, not runtime failures.

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Source URLs / references:
  - Product/source docs: `docs/goals/goal-full-product.md:221-265`; `docs/design/query-editor-data-panel/query-editor-data-panel-baseline.md:227-260`.
  - Native source references:
    - `crates/ui/src/activity_bar_view.rs:85-102`; `crates/ui/src/sidebar_view.rs:64-75`; `crates/ui/src/sidebar_activities_view.rs:96-138`; `crates/ui/src/sidebar_problems_view.rs:83-172`; `crates/ui/src/problems_view.rs:4-139`.
    - `crates/ui/src/app_types.rs:36-52,68-99,346-353`; `crates/ui/src/query_status_bar_surface_view.rs:140-150`; `crates/ui/src/query_view.rs:263-357`; `crates/ui/src/shell_output_panel_view.rs:4-53`.
    - `crates/ui/src/ide_workspace.rs:307-339`; `crates/ui/src/workspace_actions.rs:118-160`; `crates/ui/src/palette_catalog.rs:112-125`; `crates/ui/src/palette_actions.rs:19-27`; `crates/ui/src/workspace_session.rs:14-36,104-139`.
  - Vendor workflow references: [VS Code editing and diagnostics](https://code.visualstudio.com/docs/editing/editingevolved); [JetBrains Problems tool window](https://www.jetbrains.com/help/idea/problems-tool-window.html).
- Factual findings: Source-visible Problems collection/action/scope, output-panel scope, activity persistence, and product-rail omission are anchored in `problems-baseline.md` at the baseline SHA above. Vendor references describe their own workflows only.
- Inference: P2 classification and the recommendation for a workspace-level lower Problems tool panel are product/IA judgments, not source facts or accepted implementation decisions.
- Decision / recommendation: Accept the user’s direction that Problems should not be in the sidebar. Recommend removing it as a top-level rail destination and rehoming the aggregate list to an on-demand workspace-level lower tool panel; preserve inline diagnostics and palette/status access, and keep Settings support Diagnostics separate.
- Unresolved questions: Whether the lower Problems surface is a generalized tool-panel tab or a dedicated dock; whether aggregate counts cover open query documents only or all scanned workspace SQL files; future scope beyond SQL diagnostics; legacy session restoration destination.
- Downstream tasks activated: none; no issue or implementation plan was created.

## 6. Tổng kết (Vietnamese summary)

Đã hoàn tất source/product research cho Problems tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. Problems hiện tổng hợp lỗi trên query documents và SQL files nhưng nằm trong activity/sidebar; product goal không liệt kê nó trên rail. Khuyến nghị chuyển thành workspace-level lower Problems panel, giữ inline/status/command access và tách khỏi Messages lẫn Settings → Diagnostics. Chưa sửa code, chưa có runtime/test evidence; khi triển khai cần tổng quát hóa lower-panel state và xử lý session `"Problems"` cũ.
