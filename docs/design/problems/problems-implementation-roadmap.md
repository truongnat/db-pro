# Problems — Implementation Roadmap

**Source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (2026-09-24)

**Inputs:** [Problems baseline](problems-baseline.md); [feature research](problems-feature-research.md); [research evidence handoff](AGENT_EVIDENCE.md).

## Decision

**Recommendation, pending owner approval:** remove Problems as a top-level rail/sidebar activity and retain its workspace-wide diagnostic list as an on-demand lower workspace tool panel. Keep inline diagnostics, a clearly scoped status/count affordance, and the command-palette entry. Preserve the distinction between Problems, query execution Messages, and Settings → Diagnostics. This placement recommendation is not a source fact or approved implementation commitment. The lower panel must support workspace scope rather than merely relabeling per-query `OutputTab::Messages`.

## Current state

### Implemented (source-observed, not runtime-verified)

- The rail/sidebar dispatches `Activity::Problems` to a Problems view; that view provides severity/source filters, empty state, diagnostic rows, selection, and optional deterministic Quick Fix (`crates/ui/src/activity_bar_view.rs:85-102`; `crates/ui/src/sidebar_view.rs:64-75`; `crates/ui/src/sidebar_activities_view.rs:96-138`; `crates/ui/src/sidebar_problems_view.rs:83-172`).
- Aggregation covers diagnostics from all open query documents plus scanned workspace SQL files; query-document entries retain source/range and may support deterministic fixes (`crates/ui/src/problems_view.rs:4-51`; `crates/ui/src/ide_workspace.rs:307-339`; `crates/ui/src/app_types.rs:86-99`).
- Query diagnostic selection targets its source document/range; workspace-file selection opens the file. The Query status count is active-document-only and opens Messages; Settings support Diagnostics is a separate surface (`crates/ui/src/problems_view.rs:71-98`; `crates/ui/src/query_view.rs:263-357`; `crates/ui/src/settings_diagnostics_view.rs`).
- Workspace sessions serialize the activity label `"Problems"` (`crates/ui/src/workspace_session.rs:14-36,104-139`).

### Not implemented / absent in source evidence

- No workspace-scoped Problems lower-panel surface exists; existing OUTPUT tabs are query-output-owned and do not represent the workspace aggregate (`crates/ui/src/shell_output_panel_view.rs:4-53`; `crates/ui/src/app_types.rs:346-353`).
- No source-visible route from workspace-file selection to the diagnostic line; the row handler supplies only the path (`crates/ui/src/sidebar_activities_view.rs:121-128`; `crates/ui/src/workspace_actions.rs:118-160`).
- No source evidence of native UI traversal, runtime diagnostics, session restart verification, or provider execution.

### Needs fix (inherited findings; retain severity)

- **P2:** Top-level rail placement conflicts with the target compact product rail, which omits Problems (`crates/ui/src/activity_bar_view.rs:85-102`; `docs/goals/goal-full-product.md:221-265`).
- **P2:** Query status-bar count and workspace aggregate have different scopes and destinations (`crates/ui/src/query_view.rs:263-357`; `crates/ui/src/problems_view.rs:4-51`).
- **P2:** Existing lower OUTPUT state is not a workspace Problems owner; adding a label alone does not implement aggregation (`crates/ui/src/shell_output_panel_view.rs:4-53`).
- **P2:** Workspace-file diagnostic selection does not jump to its reported line (`crates/ui/src/problems_view.rs:25-49`; `crates/ui/src/sidebar_activities_view.rs:121-128`).
- **Compatibility decision required:** legacy session activity `"Problems"` needs an intentional restore destination; this is a migration concern, not a reproduced defect (`crates/ui/src/workspace_session.rs:104-139`).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P2 (inherited) | fix | `problems-baseline.md:38-42`; `problems-feature-research.md:43-51`; product rail `docs/goals/goal-full-product.md:221-265` | Approve the IA cutover: remove Problems rail dispatch and place aggregate diagnostics in a workspace-owned lower tool panel; route the Problems palette action there. | Decide whether lower panel is generalized tool-panel state or a dedicated dock. | Problems no longer occupies the rail; palette opens/focuses the aggregate list. Inline diagnostics remain available. Messages and Settings → Diagnostics retain distinct content and destinations. |
| P2 (proposed) | missing | `problems-baseline.md:21-29`; `problems-feature-research.md:45-50` | Implement workspace-scoped panel state/view over all open query documents and scanned workspace SQL diagnostics; retain severity/source filters, row details, and deterministic Quick Fix. Do not reuse per-query Messages state as the data owner. | IA/dock ownership decision; preserve existing `ProblemEntry` source actions. | With multiple query documents and a workspace SQL file, one panel lists each applicable diagnostic; filters narrow the list; Quick Fix changes the intended document and refreshes diagnostics. |
| P2 (inherited) | fix | `problems-baseline.md:24,41`; `problems-feature-research.md:48,60` | Carry workspace diagnostic line through file activation and position the caret on that line after opening/switching to the file. | Workspace file activation accepts a target location. | Selecting a workspace-file row opens the correct file and places the caret at the diagnostic line; query-document rows still select their exact range. |
| P2 (proposed) | fix | `problems-baseline.md:28-30,38-42`; `problems-feature-research.md:49-51` | Make status/count semantics explicit: active-document count remains an active-document affordance; any aggregate count names or opens the workspace aggregate. Define whether counts include scanned workspace SQL. | Panel scope and count-scope decision. | Each displayed count has a stated scope; activating it opens the list with matching scope, never only Messages for a workspace aggregate. |
| P2 (proposed) | fix | `problems-baseline.md:32-34`; `problems-feature-research.md:51` | Map persisted `"Problems"` activity deliberately to the chosen Queries/workspace-panel restore state instead of relying on unknown-activity fallback. | IA decision; session compatibility policy. | A legacy saved session restores to the documented destination without losing unrelated workspace state; new sessions persist the chosen activity state. |
| Proposed P2 — lower than correctness/placement work | upgrade | `problems-feature-research.md:66-69` | Decide whether future diagnostic sources beyond SQL belong in the aggregate; preserve current SQL-only coverage until a named source and lifecycle are approved. | Product scope decision; stable diagnostic identity for mutable lists if required. | No non-SQL source is implied by the current panel; any added source is separately named, filterable, and navigable to its origin. |
| Proposed P2 — release gate | verification | `problems-baseline.md:46-48`; `problems-feature-research.md:61,70-74` | Collect native UI and session evidence for the accepted panel/cutover; separately verify the existing database-diagnostic path. | Implement preceding rows. | Record observed behavior for multiple documents, workspace SQL selection/line jump, Quick Fix, panel filters, legacy-session restore, and separation from Messages/Settings Diagnostics. |

## Rollout / dependency order

1. Decide panel ownership, scope/count semantics, and legacy `"Problems"` restore mapping.
2. Build the workspace-scoped panel using the existing aggregate and actions; retain editor inline diagnostics and avoid treating Messages as the backing store.
3. Complete workspace-file line navigation and explicit count routing.
4. Cut over rail and palette access together; migrate saved-session restore intentionally.
5. Run the verification gates below before calling the placement complete. Broader diagnostic-source expansion is deferred until explicitly scoped.

## Provider/support matrix

This is a UI diagnostics feature, not a claim of provider feature parity. Inputs name a Database diagnostic source but provide no independent PG/SQLite runtime result.

| Provider | Source/research statement | Runtime support evidence |
|---|---|---|
| PostgreSQL | Database diagnostics are one available diagnostic source category (`crates/ui/src/app_types.rs:68-84`; `crates/ui/src/editor/diagnostics.rs:3-20,45-103`). | Not collected; do not infer provider coverage. |
| SQLite | Same source-level category; no separate SQLite behavior is established by these inputs. | Not collected; do not infer provider coverage. |

## Verification gates still needed (not run)

- Native UI traversal: filters, selection, row source navigation, Quick Fix, empty state, and palette/status entry points.
- Mixed-scope scenario with several open query documents and a workspace SQL diagnostic; workspace file caret lands on the reported line.
- Saved-session restore for legacy `"Problems"` and the selected new panel/activity state.
- Confirm Problems aggregate, query execution Messages, and Settings → Diagnostics remain distinct.
- Separate PG and SQLite runtime checks for any database-origin diagnostics claimed as supported.

No build, tests, lint, provider runtime, restart check, or native UI traversal was run for this roadmap.

## Out of scope / unresolved decisions

- No source changes, test additions, or edits to baseline/research/evidence documents.
- Pending: generalized lower tool-panel tab versus dedicated dock; exact aggregate count scope; legacy-session destination; whether future diagnostics include migrations/tasks or remain SQL-only.
- No claim that a source-visible route or recommendation has been implemented or runtime-verified.
