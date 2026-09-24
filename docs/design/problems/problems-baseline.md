# Problems Activity Baseline

> Status: source analysis; no implementation or runtime verification performed.
>
> Source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (HEAD, 2026-09-24).

## Scope

This document traces the native **Problems** activity, SQL diagnostics aggregation, navigation/actions, and adjacent diagnostics affordances. It distinguishes editor/workspace Problems from the separate Settings support-diagnostics surface. This is source analysis, not runtime evidence or an implementation commitment.

Line anchors refer to the exact source SHA above. Source-visible behavior does not prove the UI was exercised.

## 1. Surface and dispatch

- The activity rail currently includes `Activity::Problems` under Tools & Management (`crates/ui/src/activity_bar_view.rs:85-102`). `Activity::Problems` is an enum member, and the sidebar dispatch maps it to `draw_problems` (`crates/ui/src/app_types.rs:36-52`; `crates/ui/src/sidebar_view.rs:64-75`).
- `draw_problems` collects entries, renders the `SidebarProblemsContext`, and dispatches severity/source filter changes, row selection, and Quick Fix actions (`crates/ui/src/sidebar_activities_view.rs:96-138`). The rendered list has severity/source filters, an empty state, location/message/source labels, and an optional Quick fix button (`crates/ui/src/sidebar_problems_view.rs:83-172`).
- The full-product goal’s final rail lists Explorer, Search, Queries, Data, ER, Agent, Monitoring, Transfer, and Settings; Problems is not included. It also says the rail stays compact and context belongs in the sidebar (`docs/goals/goal-full-product.md:221-265`).

## 2. Diagnostics collected into Problems

- `collect_problem_entries()` walks diagnostics on every open query document and appends `ide_workspace.workspace_diagnostics` (`crates/ui/src/problems_view.rs:4-51`; workspace SQL diagnostics are built in `crates/ui/src/ide_workspace.rs:307-339`). Thus the list spans open SQL/query documents and scanned workspace SQL files; it is not limited to the active document.
- Query entries retain document ID/title, diagnostic index, severity, source, message, line/column, range, and whether a deterministic fix exists (`crates/ui/src/problems_view.rs:7-22`; `crates/ui/src/app_types.rs:86-99`). Workspace-file diagnostics use a sentinel document index, relative path, Lint source, line, and no fix (`problems_view.rs:25-49`).
- Available source filters are Parser, Lint, Delimiter, and Database; severity filters are All, Errors, and Warnings (`crates/ui/src/app_types.rs:68-84`; `problems_view.rs:54-69`). The underlying diagnostic model includes Parser, Delimiter, Database, and Lint, and may carry a code or deterministic replacement (`crates/ui/src/editor/diagnostics.rs:3-20,45-103`).
- Clicking a query-document row selects its source document/range and sets `WorkspaceTab::Query` while the Problems activity remains selected (`crates/ui/src/problems_view.rs:71-98`). A workspace-file row passes the relative path to `open_workspace_sql_file()` and then sets the Files panel sub-tab to Search (`crates/ui/src/sidebar_activities_view.rs:121-128`). `open_workspace_sql_file()` switches to an already-open file document or creates a Query document and resets the cursor for a new one (`crates/ui/src/workspace_actions.rs:118-160`). The Problems row’s line is not passed to that action, so the source does not show a jump to the workspace-file diagnostic line. Quick Fix applies a deterministic replacement, marks the document dirty, and refreshes diagnostics (`problems_view.rs:100-132`).

## 3. Adjacent diagnostics surfaces

- The Query status bar shows the active query document’s diagnostic count. Clicking it opens the bottom output panel and selects `OutputTab::Messages` (`crates/ui/src/query_status_bar_surface_view.rs:140-150`; `crates/ui/src/query_view.rs:263-280,326-357`). This is a separate entry point from the all-open-documents/workspace Problems list.
- The shell’s lower panel is titled OUTPUT and offers Results, Chart, Messages, Explain, and History; it has no Problems tab (`crates/ui/src/shell_output_panel_view.rs:13-53`; `crates/ui/src/app_types.rs:346-353`). Its content context is bound to query session/output/editor state (`shell_output_panel_view.rs:4-10`), while Problems also aggregates workspace diagnostics. Reusing `OutputTab` directly would therefore mix a workspace-wide list with a per-document query-output model.
- Command Palette distinguishes “Problems” (“Open SQL diagnostics across documents”) from “Diagnostics” (“App version, drivers, and redacted support summary”). Problems currently opens `Activity::Problems`; Diagnostics opens Settings → Diagnostics (`crates/ui/src/palette_catalog.rs:112-125`; `crates/ui/src/palette_actions.rs:19-27`). The Settings support summary is a separate diagnostics bundle path (`crates/ui/src/problems_view.rs:135-139`; `crates/ui/src/settings_diagnostics_view.rs`).

## 4. Persisted navigation implications

Named workspace sessions persist the activity as a string. `activity_label()` serializes `Activity::Problems` as `"Problems"`, and `parse_activity()` restores that value (`crates/ui/src/workspace_session.rs:14-36,104-139`). If the activity is removed or re-routed, legacy named sessions need an intentional compatibility mapping; the current parser’s unknown-value fallback is Explorer.

## 5. Source-observed risks and limits

- **P2 — Problems is modeled as a top-level navigation destination despite being document/workspace diagnostics.** Its useful action is to return the user to a source document/range or workspace file, while the authoritative full-product rail omits Problems (`problems_view.rs:71-98`; `goal-full-product.md:221-265`).
- **P2 — Diagnostic entry points have different scopes and destinations.** Problems aggregates all open query-document diagnostics plus workspace diagnostics; the Query status-bar count is active-document-only and routes to Messages, not to the Problems list (`problems_view.rs:4-51`; `query_view.rs:263-357`).
- **P2 — Existing lower output tabs are not a drop-in Problems destination.** They are query-output tabs/state, with no Problems tab and no workspace-diagnostics context (`shell_output_panel_view.rs:4-10,28-53`; `app_types.rs:346-353`).
- **P2 — Workspace-file diagnostic selection does not navigate to its reported line.** The entry carries a line number, but the selection handler passes only the file path; opening a new file resets the query cursor rather than positioning it at the diagnostic (`problems_view.rs:25-49`; `sidebar_activities_view.rs:121-128`; `workspace_actions.rs:118-160`).
- **Compatibility consideration — persisted `"Problems"` activity values** need a deliberate restore destination if the activity is removed (`workspace_session.rs:104-139`). This is a migration concern for future implementation, not a defect reproduced here.

No P0/P1 issue is established by this source-only placement review.

## 6. Evidence limits

This report is source-only at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. No build/test command, native UI traversal, screenshot, persistence/restart check, or provider runtime was performed. Existing source tests were inspected but not executed; they are not runtime evidence.
