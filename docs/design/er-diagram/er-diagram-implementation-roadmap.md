# ER Diagram Implementation Roadmap

- **Source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- **Input documents:** [ER Diagram Baseline](er-diagram-baseline.md); [Feature Research](er-diagram-feature-research.md)
- Source facts below refer to the named SHA. They describe code paths only; neither input establishes rendered UI behavior, provider runtime success, or test execution.

## Decision

Keep the diagram read-only for inspection by default and treat schema identity/invalidation as the first release gate: a graph must never display or navigate to objects from a prior connection/schema. Next, close Design Mode's stale-plan and dispatch acknowledgment gaps before broadening its mutation surface. Diagram scope/export and sidebar/tab polish are secondary, optional improvements rather than parity requirements.

## Current state

### Implemented (source-observed only)

- Central ER graph from loaded table details, with PK/FK indicators, FK edges when referenced table is present, pan/zoom, spatial culling and table navigation. Composite FK uses one edge anchored on first source/target column. Evidence: baseline §3 (`diagram_view.rs:107-138`; `diagram/model.rs:82-153`; `diagram_canvas_view.rs:60-149`).
- For schemas above 200 tables, search mode can show a one/two-hop FK neighborhood capped at 100 nodes; graph construction still begins from all loaded table details. Evidence: baseline §3 (`diagram/model.rs:10-13`; `diagram_view.rs:17-21,69-97,107-138,232-269`).
- Background layout results are checked against request id and graph version; old graph remains renderable while rebuilding (baseline §3, `diagram_view.rs:23-30,107-153`).
- Opt-in Design Mode has draft table/column/FK creation, undo/redo/discard and SQL preview/apply routed through Query runtime. This is a source path, not proof of successful dispatch, transaction semantics or provider execution (baseline §4; `diagram/design_mode.rs:155-233`; `diagram_design_actions.rs:28-83`).
- A separate Diagram sidebar activity is a schema navigation panel; canvas lives in central Diagram workspace (baseline §2, `sidebar_view.rs:75-79`; `diagram_view.rs:4-17`).

### Not implemented / incomplete

- Graph cache invalidation is not tied to real datasource/schema identity or introspection generation. A same-table-count schema transition can leave old graph nodes/edges; pending layout requests can also be reissued as versions advance before worker completion. Evidence: baseline §6 (`diagram_view.rs:107-153`; `schema_events.rs:37-60`; `schema_explorer_state.rs:43-55`; `explorer_navigation.rs:177-180,214-217`).
- Design Mode source is a panel-based create-only draft, not a full diagram editor. It does not expose persisted-object edits, rename/remove column, indexes, composite keys, or canvas repositioning; `is_unique` is not propagated from the UI/model path. Evidence: baseline §4 (`diagram_design_panel_view.rs:34-125`; `diagram/design_mode.rs:13-58,155-233`).
- Design Mode fingerprint covers ordered `schema.table` names only, not columns/FKs; apply clears draft before dispatch and the handler ignores dispatch's boolean result. Evidence: baseline §4 (`diagram_design_actions.rs:76-83`; `workspace_actions.rs:42-50`; `events_query_dispatch.rs:110-143`).
- No evidence of custom diagram scope, arbitrary selected-table neighborhoods, persisted layout, or export/print in the analyzed source (baseline §3; research F2).
- Explorer connection-menu entry opens central Diagram tab without necessarily selecting the Diagram sidebar activity; semantics are not normalized among all entry points (baseline §2, §8; research F4).

### Needs fix (inherited findings; retain severity)

- **P1:** Same-size connection/schema transition can show and open stale tables/FKs; empty-schema transition also does not guarantee cached graph is cleared (baseline §6, lines 70–76).
- **P1:** Design Mode fingerprint misses metadata drift and apply can discard the draft/report success when query dispatch rejects the operation (baseline §4, lines 62–66; research F3).
- **Source scheduling risk, priority proposed:** repeated layout dispatch/version churn while a replacement is pending may invalidate earlier results; runtime frequency/impact is unmeasured (baseline §6, lines 78–80).

## Ordered V3 backlog

| Order / priority | Type | Evidence | Concrete change and outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| 1 · **P1 inherited** | fix | Baseline §6 (`diagram_view.rs:107-153`; `schema_events.rs:37-60`; `explorer_navigation.rs:177-180,214-217`); research F1 | Key graph, spatial index and layout work to datasource identity plus schema/introspection generation. Invalidate graph and pending work on connection/schema switch, refresh replacement, empty result and relevant errors. Keep generation stable during one in-flight layout so a render does not make its valid result stale. | None | Switching schema A to different schema B with the same number of tables changes cards, edges, search results and click targets to B; A→empty→same-size B works; an old layout result cannot replace B. Loading/error states do not masquerade as current or empty schema. |
| 2 · **P1 inherited** | fix | Baseline §4 (`diagram/design_mode.rs:138-153`); research F3 | Replace table-name-only stale check with schema generation or canonical metadata fingerprint covering objects referenced by the draft/plan. Reject apply when relevant schema context changed after preview. | 1 or shared canonical schema-generation contract | Change a column or FK without changing table names after preview; apply is blocked and draft remains recoverable with a re-preview explanation. |
| 3 · **P1 inherited** | fix | Baseline §4 (`diagram_design_actions.rs:76-83`; `workspace_actions.rs:42-50`; `events_query_dispatch.rs:110-143`); research F3 | Preserve the draft until the query runtime accepts dispatch; report success only after accepted/observable execution result. Define failure/partial-failure and post-success refresh behavior without claiming atomicity unless runtime guarantees it. | 2; Query runtime result contract | When dispatch is refused (query already running or destructive gate), draft and preview remain, user sees refusal, and no success feedback appears. On accepted dispatch, outcome is tied to actual runtime result and schema refresh is observable. |
| 4 · **Proposed P1/P2 boundary** | verification | Baseline §6 and §9; research F1/F3 | Add regression coverage for same-size and empty-schema transitions plus layout worker identity; exercise Design Mode stale/apply-rejection semantics. Severity remains P1 for stale-context correctness; priority of worker churn remediation should follow measured reproduction. | 1–3 | Deterministic scenario proves graph and navigation context after each transition; no stale worker result is accepted. UI feedback matches dispatch acceptance/failure. |
| 5 · **Proposed P2** | upgrade | Baseline §3 (`diagram_view.rs:69-97`); research F2 | Add explicit context/scope labels (connection, database/schema, visible object counts) and search result counts; consider focused table/FK neighborhood entry with a route back to full schema. | 1 | Canvas labels identify the active context and displayed scope; focused view includes only the selected object/neighborhood by declared rule and can return to full schema. |
| 6 · **Proposed P2** | missing | Baseline §4 (`DraftColumn` and mutation mapping; `diagram_design_panel_view.rs:34-125`); research F3 | Decide whether Design Mode is intentionally create-only or an incomplete #226 scope. If extending, add each operation only with draft representation, undo/discard, complete preview, stale check and provider capability gate; include propagation of `is_unique` and composite key definitions before claiming those controls. | 2–3; explicit scope and provider capabilities | Every offered draft field appears correctly in reviewed mutation plan; unsupported operation is absent or blocked with reason; undo/discard restores the prior draft. No create-only UI advertises unsupported editing. |
| 7 · **Proposed P2** | upgrade | Baseline §2 (`app_lifecycle.rs:213-216`; `explorer_connection_row_view.rs:202-213`); research F4 | Choose and apply consistent entry semantics for rail, connection menu and palette; make sidebar activity, central tab and diagram context understandable from each entry. Rename sidebar summary if needed to avoid confusing it with the canvas, subject to UX decision. | Owner decision on activity vs tab semantics | Each entry results in an accurately labeled active diagram context; sidebar state follows declared rule and never implies that the summary panel is the canvas. |
| 8 · **Proposed P3** | upgrade | Baseline §3; research F2 | Only after a demonstrated workflow need, consider custom subset/layout persistence or one portable export format. Avoid persistence and format expansion before scope/ownership is defined. | Decision on diagram scope/state ownership and concrete use case | Saved custom scope/layout reopens with the same objects/context, or export can be opened as the chosen portable format; no unrequested persistence is added. |
| 9 · **Proposed P1 for mutation claims** | verification | Baseline §8–9; research §5, §7 | Verify schema graph behavior and Design Mode provider mutations independently for PostgreSQL and SQLite; separate native UI evidence from automated logic. | Mutation items before support claims | Evidence records provider/runtime versions and observed results; no source-only or unit-only path is called provider-supported. |

## Rollout / dependency order

1. **Graph identity first:** land backlog 1 and regression scenario 4 before UX additions; keep graph/search/click/layout on one schema generation.
2. **Mutation integrity:** backlog 2 then 3 ensures draft provenance and honest runtime acknowledgment. Decide operation scope before backlog 6.
3. **Scope and navigation:** backlog 5 and 7 after stale-context correctness.
4. **Optional editor breadth:** backlog 6 only for explicitly selected operations with preview/undo/stale/provider contracts; backlog 8 only after a use case and persistence owner are agreed.
5. Run gates by provider as specified below; passing one provider does not infer the other.

## Provider/support matrix

Inputs establish no PostgreSQL or SQLite runtime outcome (baseline §9; research §7). Schema graph consumes introspected table details; that source wiring alone does not prove the introspector's complete provider behavior.

| Provider | Source-level/capability evidence | Required V3 disposition / runtime gate |
|---|---|---|
| PostgreSQL | No PostgreSQL-specific runtime verification in either input. Source shows generic graph construction and mutation dispatch (baseline §8–9). | Verify graph refresh/switch to equal-size schema and FK edges using a PostgreSQL instance. Separately verify each offered Design Mode DDL operation, constraints/keys, stale-schema rejection, dispatch failure and post-apply introspection. Gate/omit unsupported operations; no SQL success claim from UI wiring. |
| SQLite | No SQLite-specific runtime verification in either input. Source is provider-neutral at graph layer; mutation SQL/provider capability and transaction semantics are unproven (baseline §8–9; research F3). | Independently exercise same graph transitions and every enabled mutation on SQLite. Verify unsupported operations are blocked or have a tested strategy; observe failure/refresh semantics. Do not infer SQLite from PostgreSQL results or vice versa. |

## Verification gates still needed (not run)

- **Automated:** graph cache key distinguishes same-count different schemas; empty-schema and connection/schema transitions; stale layout worker cannot land; Design Mode fingerprint detects column/FK change; rejected dispatch retains draft and does not emit success. Source tests mentioned in baseline §6 were not run for this work.
- **Native UI:** observe rail/menu/palette entry behavior, graph/search/click after refresh and switch, loading/error state, and Design Mode preview/refusal/success feedback. No native UI traversal or screenshot was performed in the inputs.
- **PostgreSQL runtime:** independently exercise graph introspection and each claimed Design Mode mutation, including stale plan and dispatch/apply failure. Not run.
- **SQLite runtime:** repeat independently; verify operation capability and transaction/partial failure behavior rather than assuming PostgreSQL parity. Not run.
- **Performance/scheduling:** measure large-schema graph construction and pending-layout behavior before elevating repeated layout dispatch from source-level risk to a confirmed runtime issue. Not run.

## Out of scope / unresolved decisions

- Exact diagram scope (active schema, selected object neighborhood, custom subset) and persistence owner for layout/search/zoom (research §6).
- Export/print/notation parity absent a confirmed sharing/reporting need (research F2).
- Whether create-only Design Mode is deliberate or should meet broader #226 scope; whether draft mutation supports rename/remove/indexes/composite PK/FK, and which provider supports each (research F3, §6).
- Transaction, atomicity, partial-failure, retry and recovery semantics remain unresolved; never claim atomic mutation without runtime contract/evidence (baseline §8; research F3).
- Entry-point ownership of sidebar activity versus central workspace tab (research F4).
