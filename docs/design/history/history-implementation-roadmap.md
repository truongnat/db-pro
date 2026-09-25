# History — Implementation Roadmap

**Source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (2026-09-24)

**Inputs:** [History baseline](history-baseline.md); [feature research](history-feature-research.md); [research evidence handoff](AGENT_EVIDENCE.md).

## Decision

**Recommendation, pending product-owner decisions:** make query execution history one coherent, safe-to-use feature and place Query History under Queries, consistent with product goals. First fix the source-derived P1 UTF-8 preview panic and P2 unsafe/open, outcome-display, and recency issues. Then decide canonical persistence, record semantics, privacy, and retention before schema or durable-storage changes. Meta-store canonical ownership is recommended by research, not an accepted implementation decision; Phase G and Full Product Goal currently describe storage differently.

## Current state

### Implemented (source-observed, not runtime-verified)

- The History rail activity reuses saved-query and local-history library sections; it is not a structured execution log (`crates/ui/src/sidebar_view.rs:64-75`; `crates/ui/src/sidebar_activities_view.rs:140-157`).
- Local sidebar history is distinct SQL strings in a `Vec<String>`, recorded before results, capped at 20, shown up to 15, and not persisted by the cited lifecycle hook (`crates/ui/src/query_execution_actions.rs:128-150`; `crates/ui/src/sidebar_query_library_view.rs:48-67`; `crates/ui/src/app_lifecycle.rs:53-65`).
- Structured UI execution entries record SQL, context, timing, outcome, row counts, and errors, cap at 500, persist through eframe storage, and appear in output Recent Executions; Replay creates and executes a context-bearing document (`crates/ui/src/runtime_query_types.rs:55-75`; `crates/ui/src/query_history_events.rs:6-29`; `crates/ui/src/app_lifecycle.rs:53-59`; `crates/ui/src/query_output_actions_view.rs:117-154,173-230`).
- Quick Open consumes the structured UI list; a separate SQLite meta-store repository and `QueryApi::history` read path exist. No UI caller for that API was found in source research (`crates/ui/src/palette_search_view.rs:266-299`; `crates/infrastructure/src/meta/query_history_repo.rs:11-65`; `crates/runtime/src/api.rs:352-358`).

### Not implemented / absent in source evidence

- A single history source used by rail/sidebar, output pane, Quick Open, and meta-store API is not established; the meta-store read path has no UI caller (`history-baseline.md:41-45`).
- No source-visible history clear/export controls, meta-store retention/pruning contract, or privacy policy exists (`history-baseline.md:44-45,63-69`).
- No runtime evidence for persistence/restart, replay, PostgreSQL/SQLite execution, or native UI behavior was collected.

### Needs fix (inherited findings; retain severity)

- **P1 edge-case risk:** preview truncation slices at byte 117 and can panic if that index is not a UTF-8 character boundary (`crates/ui/src/query_output_actions_view.rs:224-227`). Source-derived, not reproduced.
- **P2:** sidebar history selection replaces active SQL without dirty confirmation or origin context (`crates/ui/src/sidebar_activities_view.rs:160-166`; `crates/ui/src/query_documents.rs:78-83,239-246`).
- **P2:** Quick Open takes the first 30 records of an oldest-to-newest collection, so it omits newer retained records (`crates/ui/src/palette_search_view.rs:266-299`; `crates/ui/src/query_history_events.rs:8-29`).
- **P2:** Cancelled entries render as OK (`crates/ui/src/query_output_actions_view.rs:181-191`).
- **P2:** retention and clear semantics differ across local strings, eframe records, and SQLite meta-store (`crates/ui/src/query_history_events.rs:6-29`; `crates/ui/src/query_execution_actions.rs:143-150`; `crates/core/src/ports/query_history_repository.rs:10-21`).
- **P2:** History rail duplicates query-library content and conflicts with product placement under Queries (`crates/ui/src/sidebar_queries_surface_view.rs:34-68,83-108`; `docs/goals/goal-full-product.md:221-258`).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 edge-case risk (inherited) | fix | `history-baseline.md:33,67`; `history-feature-research.md:84-99` | Replace byte-index string slicing with character-boundary-safe truncation while retaining a bounded preview. | None. | A preview containing multibyte UTF-8 across the former cutoff renders without panic and does not split a character. |
| P2 (inherited) | fix | `history-baseline.md:23-26,63-65`; `history-feature-research.md:67-82` | Give history actions explicit Open/Insert/Copy/Replay semantics. Open a history record into a new document with original context; do not overwrite dirty SQL. Keep Replay an explicit execution action and preserve existing safety/confirmation pipeline. | Stable history record identity/context; define behavior when connection/schema is unavailable. | Selecting Open leaves active SQL unchanged and creates a context-bearing document; Insert changes only by explicit action; Replay executes only when selected and does not silently substitute a different connection/schema. |
| P2 (inherited) | fix | `history-baseline.md:38-39,65`; `history-feature-research.md:84-99` | Make Quick Open select the newest retained records under an explicit chronological ordering; use stable entry identity rather than a fragile display-list index. | Ordering contract and stable ID, if source order can change during selection. | With more than 30 records, results include the latest 30 in documented newest-first order; selecting one opens the intended entry without executing it. |
| P2 (inherited) | fix | `history-baseline.md:32,66`; `history-feature-research.md:88-99` | Render Success, Failed, and Cancelled as distinct outcomes; expose useful recorded context consistently. | None. | A cancelled execution is visibly Cancelled and cannot be mistaken for success; failure remains visibly Failed. |
| Proposed P1 — correctness invariant | verification | `history-baseline.md:33,48`; `history-feature-research.md:94-99` | Add a focused regression scenario for UTF-8 cutoff and ordering/outcome boundaries after fixes. | Corresponding fixes. | Test proves multibyte boundary safety, newest-record selection beyond 30, and cancelled-versus-success presentation through consumer-visible behavior. |
| Proposed P2 — owner decision before durable change | verification | `history-baseline.md:41-45,68`; `history-feature-research.md:49-65,101-115,150-159` | Resolve canonical record schema/source, success/failure/cancel/partial multi-statement semantics, SQL-text privacy, retention, clear/export, and migration policy before modifying persistence. | Product/storage owner decisions; none inferred from existing storage paths. | Written contract identifies authoritative records, outcome behavior, retention/clear/privacy scope, and migration/restart expectations; no storage migration precedes approval. |
| Proposed P2 — after policy approval | missing | `history-baseline.md:41-45,68`; `history-feature-research.md:60-65` | If approved, implement one canonical history source behind existing repository/application boundaries; connect Queries, output history, and Quick Open to it, with eframe limited to view state. Preserve failure/cancel/context fields per the approved contract. | Approved canonical-store/outcome/privacy/retention contract; schema migration if required. | All history surfaces show the same identifiable records and outcomes after refresh/restart as policy specifies; no silent provider/context reassignment. |
| Proposed P2 — after canonical source | upgrade | `history-feature-research.md:101-115`; `history-baseline.md:68` | Add SQL search and metadata filters, explicit retention and scoped clear/export, and user-facing storage/privacy description only as approved. | Canonical source and owner-approved retention/privacy contract. | Search/filter matches defined fields; clear removes records within the stated scope; records outside scope remain; restart follows chosen policy. |
| P2 (inherited) | fix | `history-baseline.md:15-17,69`; `history-feature-research.md:116-128` | Consolidate Query History under Queries and remove duplicate History rail entry after all access paths are migrated. Keep top-level History only if a distinct cross-feature timeline is explicitly designed. | Information-architecture decision; entry-point migration. | Queries has the supported query-history entry point; no duplicate rail surface remains, or an explicitly scoped cross-feature timeline is documented and distinct. |

## Rollout / dependency order

1. Fix the UTF-8 boundary risk; establish focused consumer-visible checks.
2. Make Open/Insert/Copy/Replay semantics safe and distinct; correct Quick Open recency and outcome labels.
3. Decide record identity, canonical store, success/failure/cancel/partial semantics, SQL privacy, retention, clear/export, migration, and restart policy.
4. Only after approval, migrate storage/read paths and connect all UI history surfaces to the canonical source.
5. Add search/filters/retention controls and change rail placement after dependent entry points are complete.
6. Verify eframe/meta-store restart behavior and PostgreSQL and SQLite separately.

## Provider/support matrix

The meta-store repository identified by the baseline is SQLite-backed. This does not establish PostgreSQL or SQLite query-execution history parity. UI records and meta-store records are different paths; provider runtime was not exercised.

| Provider / store | Source/research statement | Runtime support evidence |
|---|---|---|
| PostgreSQL query target | UI history can carry connection/schema metadata; no PostgreSQL runtime result or provider-specific history behavior is established (`history-baseline.md:30-45`). | Not collected. |
| SQLite query target | Same UI-path evidence only; no SQLite query-history runtime result is established. | Not collected. |
| SQLite meta-store | Repository implementation is in `crates/infrastructure/src/meta/query_history_repo.rs:11-65`; it saves/lists records for the meta store. | Repository source only; no runtime/restart verification. |

## Verification gates still needed (not run)

- Focused UTF-8 cutoff regression and visible cancelled/success/failed distinction.
- Quick Open over more than 30 records, proving newest ordering and intended selected identity.
- Dirty active-document scenario: Open preserves current SQL; explicit Insert/Copy/Replay semantics and unavailable-context behavior.
- Native UI traversal and restart check for all history surfaces under the approved persistence policy.
- Separate PostgreSQL and SQLite execution scenarios, plus SQLite meta-store save/list/migration/restart checks; report each independently.
- Confirm approved retention, clear/export, privacy, and partial multi-statement outcomes.

No build, tests, lint, UI traversal, restart, or provider runtime was run for this roadmap.

## Out of scope / unresolved decisions

- No code changes or edits to existing baseline/research/evidence documents.
- Unresolved: meta store versus current eframe ownership; record/outcome contract; SQL literal privacy; retention scope and clear/export; replay when context is missing; recovery for never-executed SQL; History under Queries versus a separately modeled cross-feature timeline.
- No claim that recommendations or source-visible behavior have been runtime-verified.
