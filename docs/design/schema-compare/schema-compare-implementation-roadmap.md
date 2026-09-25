# Schema Compare Implementation Roadmap

- **Source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- **Input documents:** [Schema Compare Baseline](schema-compare-baseline.md); [Feature Research](schema-compare-feature-research.md)
- Source-level claims below are observations at that SHA, not proof of native UI behavior, provider runtime behavior, or test success. Priorities and proposed work are recommendations from the research unless identified as inherited severity.

## Decision

Treat structural comparison, migration planning/apply, and keyed row comparison as distinct workflows with independent evidence and safety gates. First block unsafe or misleading schema migration. A plan must not execute incomplete placeholder DDL, silently retarget a connection/provider, or claim full equivalence from partial metadata. Keep Data Compare explicitly sample-based and preview-only; do not make it a migration acceptance proxy. This ordering follows F1–F4 and Goal Phase F constraints in the research (§2–5).

## Current state

### Implemented (source-observed only)

- One in-memory schema snapshot can be diffed against the current Explorer schema; UI compares table presence and column name/type/nullability, and view/routine identities. Evidence: baseline §2, §4–5 (`schema_compare.rs:5-15,48-117`; `schema_compare_state.rs:15-59`).
- A migration-plan/SQL-preview/apply path exists, with warnings and destructive confirmation; apply dispatches DDL to whichever connection is active. Evidence: baseline §2 and §5 (`migration_planner.rs:112-217`; `schema_compare_view.rs:173-258`; `query_session.rs:59-82`).
- Keyed Data Compare reads two connections using a 1,000-row sample, reports total table counts and sample outcomes, and presents comment-only sync preview. Evidence: baseline §3 (`workspace_actions.rs:17-39`; `data_diff.rs:37-46,91-119,182-199`; `schema_compare_view.rs:262-345`).
- Source-visible tests exist for a basic diff, state validation, and some planner behavior; they were not run and do not establish provider runtime safety (baseline §7).

### Not implemented / incomplete

- Full structural coverage is absent: PK/FK definitions and other properties are not compared; adapter drops nullability/view/routine deltas and supplies no index deltas. “Schemas are identical” therefore means only equal on a subset. Evidence: baseline §4–5 (`schema_compare.rs:48-117,120-174`).
- Executable DDL is not derived from complete captured object metadata: reachable CREATE TABLE and ADD COLUMN use placeholders; core CREATE INDEX is also placeholder-based, though the UI adapter currently does not feed index deltas. Evidence: baseline §5 (`migration_planner.rs:112-151,167-217`; UI adapter `schema_compare.rs:120-174`).
- Plan is not bound to a stable target connection/provider or checked against live target schema at apply. Data Compare request IDs are not used to discard stale results; result panel omits cell-level column changes. Evidence: baseline §5–6 (`schema_compare_state.rs:75-88,91-140`; `query_session.rs:59-82`; `event_router.rs:66-67`; `management_events.rs:139-146`; `schema_compare_view.rs:285-343`).
- Persistent snapshot lifecycle, explicit source/target picker, selected-operation controls, and Data Compare independent of structural-diff empty state are not present in the analyzed source. Evidence: baseline §4–6; research §3–4.

### Needs fix (inherited findings; retain severity)

- **P1:** Apply can execute reachable placeholder DDL; block incomplete operations before dispatch (baseline §5, lines 78–80).
- **P1:** Plan can be applied to another active connection/provider after switching; bind and verify target identity (baseline §5, lines 82–84).
- **P1 / P2 conditional:** Partial structural diff can omit real drift while reporting identical or feeding a misleading migration plan; do not call partial coverage complete (baseline §5, lines 90–92).
- **P2:** Replacing a snapshot leaves an old plan applyable; successful DDL also leaves old plan/confirmation active (baseline §5, lines 86–88, 98–100).
- **P2:** Out-of-order keyed compare responses can overwrite newer results (baseline §5, lines 94–96).

## Ordered V3 backlog

| Order / priority | Type | Evidence | Concrete change and outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| 1 · **P1 inherited** | fix | Baseline §5 (`migration_planner.rs:112-151,167-217`; `schema_compare_state.rs:119-140`); research F1 | Make “supported” mean complete provider-valid SQL. Reject placeholder/incomplete SQL and unresolved warnings at the final dispatch boundary; retain a reason per blocked operation. Do not expose an apply action for a plan that cannot be executed faithfully. | None | No placeholder/comment-marker SQL reaches `ExecuteDdl`; incomplete operations are visibly blocked with reasons; complete operation SQL accounts for source metadata rather than placeholder columns/types. |
| 2 · **P1 inherited** | fix | Baseline §5 (`schema_compare_state.rs:75-88,119-140`; `query_session.rs:59-82`); research F1/F3 | Bind plan to target connection identity and driver, and verify both at apply. Record/compare target schema generation or fresh canonical fingerprint; require re-diff on target drift. Never silently retarget. | 1 for safe apply gate | Changing active connection/provider or target schema after preview prevents dispatch and identifies the mismatch; unchanged target can proceed only after explicit confirmation. |
| 3 · **P2 inherited** | fix | Baseline §5 (`schema_compare_state.rs:50-72,61-88`; `ddl_events.rs:13-34`; `operation_events.rs:187-205`) | Invalidate diff, plan, preview and destructive confirmation when snapshot/current schema/target changes and after successful DDL. Preserve failure evidence; do not leave a completed plan replayable. | 1–2 | Taking a replacement snapshot, switching target, refreshing changed schema, and successful apply each make the old plan non-applyable; failed apply retains diagnostics and does not claim completion. |
| 4 · **P1 inherited for migration evidence; P2 for explicitly partial inspection** | fix | Baseline §4–5 (`schema_compare.rs:48-117,120-174`); research F2 | Define metadata coverage per category and provider; expand canonical diff only where source and target metadata suffice. Until then show `not compared`/partial status; never claim “identical” or create migration operations for unavailable/incomplete deltas. | 1; metadata model and provider capabilities | A schema differing only in an unsupported/uncompared property is labeled partial/not compared, not identical; every planner operation traces to a represented delta with sufficient metadata. |
| 5 · **P2 inherited** | fix | Baseline §6 (`runtime_protocol.rs:550-553`; `event_router.rs:66-67`; `management_events.rs:139-146`); research F4 | Carry request/context identity through Data Compare completion and accept only the current request for the same source/target/table/key context. | None | Start request A then B and complete A last: displayed result remains B; changing compare inputs invalidates the prior result. |
| 6 · **Proposed P2** | missing | Baseline §4, §6 (`schema_compare_state.rs:30-45,91-117`); research F3/F4 | Make comparison direction and provenance explicit: identify snapshot/source and destination connection/schema in the preview; decide whether user-selected live source/target is in scope before adding persistence. | 2; owner decision on snapshot semantics | Preview names the source snapshot/context and exact destination connection/provider; no ambiguous “source/target” direction; unresolved provenance blocks apply. |
| 7 · **Proposed P2** | upgrade | Baseline §6 (`schema_compare_view.rs:89-101,260-284,285-343`); research F4 | Improve the read-only row-diff experience: allow opening Data Compare without first producing structural differences, render changed-column examples subject to an explicit data-display policy, and label sample counts versus full-table counts. Keep sync preview non-executable. | 5 for request correctness | User can run row comparison when structural diff is empty; sampled changed fields and truncation/sample semantics are visible; no UI action implies sync execution. |
| 8 · **Proposed P2** | verification | Baseline §7–8; research F1–F4 | Establish targeted behavioral and provider-runtime gates for migration safety, coverage, invalidation, and row compare; separate automated, UI and provider evidence. | 1–7 as relevant | Gate report records observed output and exact provider/version/config; no provider is marked supported from source wiring or unit tests alone. |

## Rollout / dependency order

1. **Safety stop:** backlog 1 prevents incomplete DDL; backlog 2 binds plan and checks target freshness before any migration execution.
2. **Lifecycle correctness:** backlog 3 invalidates plans at every provenance-changing transition.
3. **Honest structural coverage:** backlog 4 establishes `compared` versus `not compared`; expand planner support only with complete metadata.
4. **Independent Data Compare correctness:** backlog 5 then backlog 7; keep its sample/read-only semantics distinct from schema migration.
5. **Direction and workflow decision:** backlog 6 clarifies snapshot versus live-pair semantics; do not add persistence until ownership/retention is decided.
6. Run gates in §7 separately for PostgreSQL and SQLite before making provider-specific support claims.

## Provider/support matrix

Source evidence proves no PostgreSQL or SQLite runtime behavior (baseline §8; research §7). Existing planner source tests mention SQLite type-alter capability only; this is not runtime support evidence.

| Provider | Source-level observation | Required V3 disposition / runtime gate |
|---|---|---|
| PostgreSQL | Planner records a driver and emits SQL, but target driver/identity is not checked at apply; placeholder paths are marked supported (baseline §5, lines 60–65, 78–84). | Independently verify complete preview SQL, blocked unsupported/incomplete operations, target binding/freshness, execution result, post-apply introspection and stale-plan invalidation against a PostgreSQL runtime. Do not claim migration support before this gate. |
| SQLite | Source-visible planner test covers a type-alter capability path; it does not establish runtime correctness. Research explicitly calls for SQLite-specific unsupported alter/rebuild handling (baseline §7; research F1). | Independently verify each operation and transaction/rebuild strategy on SQLite; otherwise block unsupported alter with a precise explanation. Verify same target/freshness and post-apply behavior. Do not infer PostgreSQL behavior or vice versa. |

## Verification gates still needed (not run)

- **Automated behavior:** placeholder-operation rejection; connection/provider mismatch; target schema change after preview; snapshot replacement and post-success plan invalidation; omitted-category partial status; out-of-order DataDiff completion. Source-visible tests were reported in baseline §7, but none were run for this roadmap.
- **Native UI:** traverse snapshot → diff → preview → blocked/apply states, destructive confirmation, target mismatch, stale-plan invalidation, partial-coverage copy, and Data Compare independently of structural diff. No native UI traversal was performed in either input.
- **PostgreSQL runtime:** execute only complete supported plans on disposable schemas; compare expected resulting metadata; exercise stale target and failure paths. Not run.
- **SQLite runtime:** independently verify supported DDL, unsupported type alteration/rebuild behavior, failure/recovery and refreshed metadata. Not run.
- **Data Compare runtime:** validate key uniqueness and sample/truncation claims with known fixtures across both providers where supported. Not run; sync preview remains non-mutating.

## Out of scope / unresolved decisions

- Whether the snapshot is a local historical restore baseline or one side of a live Origin/Target comparison; target selection and snapshot persistence/retention remain owner decisions (research §6).
- Data synchronization/export/apply; the current displayed sync SQL is comment-only preview (baseline §3).
- Full commercial-tool parity, every schema category, and automatic claim of schema equivalence; categories must be explicitly scoped (research §2, F2).
- Per-operation selection, data conflict policy, transaction/rollback guarantees, and provider rebuild strategy remain to be designed before they are promised (research §4, §6).
