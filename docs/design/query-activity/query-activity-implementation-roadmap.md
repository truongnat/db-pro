# Queries Activity — Implementation Roadmap

**Source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (2026-09-24)

**Inputs:** [Query Activity baseline](query-activity-baseline.md); [feature research](query-activity-feature-research.md). No per-feature `AGENT_EVIDENCE.md` was present.

## Decision

**Recommendation, pending product-owner decisions:** preserve the Queries activity as the workspace navigator/library, but treat opening saved/history SQL as document-lifecycle operations rather than assigning strings into the active editor. Fix the no-op Open in Editor path and rename flow before broadening the library. Keep execution, result-grid, and central editor behavior in the separately documented Query Editor/Data Panel scope. Decide history persistence/privacy and Query/Scratch lifecycle before adding storage or document types.

## Current state

### Implemented (source-observed, not runtime-verified)

- Queries sidebar includes open query documents, saved-query folders, a short SQL-string history list, six built-in snippets, and New Scratch (`crates/ui/src/sidebar_queries_surface_view.rs:26-32,83-118`; `crates/ui/src/sidebar_queries_view.rs:5-68`; `crates/ui/src/sidebar_query_library_view.rs:26-85`; `crates/ui/src/sidebar_query_shortcuts_view.rs`; `crates/ui/src/query_snippets.rs`).
- Open query documents support select, duplicate, rename, close requests, and dirty marking. Saved-query library has folder grouping, copy, delete confirmation, and runtime command paths for load/save/rename/delete (`crates/ui/src/sidebar_activities_view.rs:32-44,175-202`; `crates/ui/src/query_save_commands.rs`; `crates/ui/src/query_save_actions.rs`).
- Saved-query/history selection currently assigns SQL to active document; built-in snippet insertion routes into active query editor (`crates/ui/src/sidebar_activities_view.rs:46-59,160-202`; `crates/ui/src/query_documents.rs:78-83`).
- Structured execution history exists separately from the sidebar `Vec<String>`; neither provider-runtime success nor scratch restart persistence is established (`crates/ui/src/query_history_events.rs:6-29`; `crates/ui/src/sidebar_activities_view.rs:11-20`; `crates/ui/src/query_documents.rs:23-34`).

### Not implemented / absent in source evidence

- “Open in Editor” context-menu action emits no open action (`crates/ui/src/sidebar_query_library_view.rs:167-180`).
- Rename does not collect a replacement title; reducer synthesizes one from folder draft or appends a suffix (`crates/ui/src/sidebar_activities_view.rs:190-202`).
- No source-visible dirty guard, separate saved-query document open, or source connection/schema transfer on saved-query/history selection (`crates/ui/src/sidebar_activities_view.rs:160-166`; `crates/ui/src/query_documents.rs:78-83`; `crates/ui/src/query_state.rs:41-46`).
- No sidebar search/filter or move-between-folders workflow is visible; no user-defined snippet catalog/search is represented (`crates/ui/src/sidebar_query_library_view.rs:108-148,150-223`; `crates/ui/src/sidebar_query_shortcuts_view.rs`; `crates/ui/src/query_snippets.rs`).
- Provider runtime, UI behavior, and scratch persistence/restart behavior were not verified.

### Needs fix (inherited findings; retain severity)

- **P1 (research):** opening saved query/history can overwrite unsaved active SQL and retain the wrong connection/schema context (`query-activity-baseline.md:111-117`; `query-activity-feature-research.md:51-92`).
- **P1 (research):** Open in Editor is a no-op and saved-query rename has no user-supplied title (`query-activity-baseline.md:119-125`; `query-activity-feature-research.md:45-46,64-91`).
- Product research labels safe open/context P1; this roadmap retains that priority rather than downgrading it.

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 (inherited) | fix | `query-activity-baseline.md:111-117`; `query-activity-feature-research.md:51-92` | Make saved-query and history Open create a separate document carrying source SQL and source connection/schema. Separate explicit Insert/Replace/Copy actions; dirty buffers are not overwritten without an explicitly named replacement action and guard. Preserve saved-query identity if required by chosen Save/Update semantics. | Define source identity and missing/deleted connection behavior; document lifecycle/dirty policy. | With active dirty SQL, Open leaves it unchanged and creates the intended document with original context; Insert and Copy have distinct effects; missing context is surfaced, never silently replaced by active connection. |
| P1 (inherited) | fix | `query-activity-baseline.md:119-125`; `query-activity-feature-research.md:64-91,180-184` | Wire Open in Editor to the safe-open path. Replace synthesized rename behavior with a user-entered name, cancel, validation, and command-result feedback. | Safe document-open contract; existing rename command result path. | Open in Editor opens the selected saved query; rename uses exactly the submitted title; cancel preserves old title; runtime failure is not presented as success. |
| P2 (research) | fix | `query-activity-baseline.md:99-107`; `query-activity-feature-research.md:94-121` | Decide whether Queries history consumes structured execution records or remains a documented short session list; if unified, search records by SQL and metadata and keep unexecuted-text recovery separate. Do not claim restart persistence without proof. | Product decision on persistence/retention/privacy; canonical history record decision (coordinate with History roadmap). | Search/filter obey defined scope; history entry context is explicit; restart behavior matches approved policy; unexecuted SQL recovery is not claimed from execution records. |
| P2 (research) | missing | `query-activity-baseline.md:102-103`; `query-activity-feature-research.md:123-142,192-198` | Complete saved-query library operations: user-named rename first; then decide create/rename/move folder and move query, plus search/filter by name or SQL. Keep saved-query records distinct from filesystem SQL files. | Safe open/identity lifecycle; folder ownership/connection scope decision. | Each library action acts on the chosen saved-query identity/folder; search results identify title/source context; file-backed scripts are not silently treated as saved-query records. |
| Proposed P2 — after lifecycle decision | verification | `query-activity-baseline.md:66-68,97-107`; `query-activity-feature-research.md:144-161,200-207` | Verify current Query/Scratch restore and close behavior before changing document kinds or describing scratch as disposable. Choose explicit ownership, persistence, connection binding, and close/reopen contract. | Inspect/approve lifecycle and persistence policy; no new document type presumed. | Scratch/Query behavior across close/reopen/restart matches documented policy; dirty close does not lose content contrary to that policy. |
| Proposed P2 | upgrade | `query-activity-baseline.md:106-107`; `query-activity-feature-research.md:163-176` | Add snippet search/category/favorites or user-defined snippets only if authoring use case warrants it; make provider/dialect tags and insert/selection semantics explicit. | Snippet format/variable and provider applicability decisions. | Search returns matching snippets; unsupported dialect snippets are not presented as compatible; insertion follows documented selection/cursor behavior and does not unexpectedly replace content. |
| Proposed P2 — release gate | verification | `query-activity-feature-research.md:85-92,202-209`; `query-activity-baseline.md:87-90,134-138` | Exercise safe open, rename, save/load/delete, dirty lifecycle, and provider-specific context behavior independently on PostgreSQL and SQLite. | Implemented P1 workflows; test environments and provider workflows. | Each provider’s selected context and saved-query operations are observed independently; no behavior is inferred across providers; failures and stale contexts are visible. |

## Rollout / dependency order

1. Define safe-open document identity, source connection/schema handling, and dirty-buffer semantics.
2. Implement separate Open/Insert/Replace/Copy actions; wire Open in Editor; replace fake rename with user input and truthful command feedback.
3. Coordinate history model/storage choices with History scope; decide query-text privacy, retention, and unexecuted-text recovery separately.
4. Complete saved-query folder/search/move workflows only after identity and ownership semantics are settled.
5. Verify Query/Scratch close, reopen, and restart contract before expanding document lifecycle.
6. Consider snippet catalog expansion after authoring/provider insertion contract is defined.
7. Run provider-specific and native UI gates before describing the feature as complete.

## Provider/support matrix

The baseline identifies query library commands loaded per connection, but explicitly does not establish successful PostgreSQL or SQLite calls. Context behavior must be checked independently.

| Provider | Source/research statement | Runtime support evidence |
|---|---|---|
| PostgreSQL | Saved query list/operations route through runtime commands and current-connection events (`query-activity-baseline.md:87-90,102-103`). Research requires independent validation of connection/schema semantics (`query-activity-feature-research.md:85-92,202-209`). | Not collected; no successful PG saved-query, safe-open, rename, or context run established. |
| SQLite | Same source-level command-path statement only; no independent SQLite behavior can be inferred. | Not collected; no successful SQLite saved-query, safe-open, rename, or context run established. |

## Verification gates still needed (not run)

- Native UI dirty-buffer scenario for saved-query and history open, explicit insert/replace/copy, context preservation, Open in Editor, rename cancel/success/failure.
- Provider-specific PostgreSQL and SQLite runs for query load/save/rename/delete and opening with original connection/schema; test absent or disconnected context handling.
- Query/Scratch close/reopen/restart behavior against an explicit persistence contract.
- History search/persistence/retention only after coordination with the History feature’s source-of-truth/privacy decision.
- Snippet search, dialect compatibility, selection insertion, and cursor behavior if that upgrade is approved.

No build, tests, lint, provider runtime, restart check, or native UI traversal was run for this roadmap.

## Out of scope / unresolved decisions

- No source changes or edits to existing baseline/research documents.
- Excludes central SQL editing/execution/result-grid/output implementation covered by `docs/design/query-editor-data-panel/`.
- Pending: saved-query ownership/scope and Save/Update identity; behavior for deleted connections; history persistence/retention/privacy and query recovery; Query/Scratch lifecycle and storage; folder semantics; snippet format/provider applicability.
- No inference of PG behavior from SQLite or vice versa; no claims of runtime support from source paths alone.
