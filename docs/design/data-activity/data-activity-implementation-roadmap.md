# Data Activity — Implementation Roadmap

**Source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (2026-09-24)

**Inputs:** [Data Activity baseline](data-activity-baseline.md); [feature research](data-activity-feature-research.md). No per-feature `AGENT_EVIDENCE.md` was present.

## Decision

**Recommendation, pending product-owner approval:** retain the fast-return capability, but decide whether its persistent list remains a separately visible activity (renamed to Tables or Pinned & Recent) or moves into Explorer with Quick Open retained. Before expanding functionality, repair pin/recent object identity so selection cannot silently resolve to a same-named object in the active context. Route table SQL draft generation through a provider-aware identifier-quoting boundary. Do not create another data grid; use the canonical Table workspace. These are recommendations, not implementation status.

## Current state

### Implemented (source-observed, not runtime-verified)

- Data activity displays pinned/recent table shortcuts and routes rows into the canonical Table workspace, not a second grid (`crates/ui/src/sidebar_activities_view.rs:62-94`; `crates/ui/src/table_view.rs:7-34,52-82`).
- Row/context actions include open data, open structure, new query, pin/unpin, and remove recent (`crates/ui/src/sidebar_data_view.rs:72-82,85-165`).
- Pins and recents are string table names persisted in local eframe storage; Quick Open exposes the same strings (`crates/ui/src/schema_explorer_state.rs:3-18,103-116`; `crates/ui/src/app_lifecycle.rs:53-65`; `crates/ui/src/palette_search_view.rs:134-174`).
- Opening a different table protects staged table changes with a confirmation path (`crates/ui/src/workspace_actions.rs:54-84`).

### Not implemented / absent in source evidence

- Pinned/recent entries do not carry connection, database/catalog, schema, or object-kind identity and are resolved using active connection/schema (`crates/ui/src/schema_explorer_state.rs:13-15`; `crates/ui/src/workspace_actions.rs:54-84`).
- No source-visible missing-reference handling, rebind workflow, row-level origin context, search/filter, or copy-qualified-name action exists in the sidebar action model (`crates/ui/src/sidebar_data_view.rs:6-20,72-165`).
- No source-visible recent query-result/export reference exists in the sidebar actions (`data-activity-feature-research.md:53-55,100-111`).
- Runtime multi-connection, persistence/reconnect, and provider-specific behavior were not tested.

### Needs fix (inherited findings; retain severity)

- **P1:** bare table-name strings lose target identity; changing active connection/schema can open a different same-named table, conflicting with Issue #212 acceptance (`crates/ui/src/schema_explorer_state.rs:13-15`; `crates/ui/src/workspace_actions.rs:54-84`; `data-activity-feature-research.md:66-85`).
- **P1 review:** New query for table and Explorer OpenQuery interpolate raw schema/table identifiers into SQL (`crates/ui/src/sidebar_activities_view.rs:79-85`; `crates/ui/src/explorer_details.rs:121-124`). This can produce malformed drafts for quoted/reserved identifiers; exploitability or auto-execution is not established.
- **P2:** activity label “Data” describes the central grid more than the pinned/recent list; overlap exists with Quick Open and Explorer (`data-activity-baseline.md:94-110`; `data-activity-feature-research.md:50-62`).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 (inherited) | fix | `data-activity-baseline.md:72-76,88-92`; `data-activity-feature-research.md:66-85` | Replace bare strings with stable table references containing connection id, database/catalog where relevant, schema, object name, and kind. Use the same identity for Data, Quick Open, pin/recent removal, and selection; migrate persisted string state deliberately. | Define pin/recent scope and migration behavior for legacy name-only entries. | Two same-named tables across connections/schemas remain distinct; switching active connection does not retarget an item; legacy entries are resolved/rebound explicitly rather than silently targeting active context. |
| P1 (inherited review) | fix | `data-activity-baseline.md:102-105`; `data-activity-feature-research.md:113-125` | Send a typed table reference through a centralized provider-aware SQL-generation boundary; quote each identifier component separately. Remove raw SQL formatting from Data and Explorer UI action paths. | Structured object identity; provider capability boundary and exact output contract. | Names with spaces, reserved words, quotes, and delimiter characters produce a valid single-statement draft for each provider; no generated SQL is auto-executed. Safety-classifier behavior is assessed separately, not assumed. |
| Proposed P1 — safe target resolution | verification | `data-activity-baseline.md:88-105`; `data-activity-feature-research.md:77-85,119-125` | Validate identity migration, same-name resolution, stale targets, and generated SQL independently on PG and SQLite. | Identity and SQL-boundary changes. | Tests/scenarios prove target is not changed by active-context switching; missing connection/object is explicit; provider-specific quoting is correct and draft-only. |
| P2 (inherited) | upgrade | `data-activity-baseline.md:94-100`; `data-activity-feature-research.md:56-62,87-98` | Decide and implement content-accurate activity naming or consolidate pin/recent sections into Explorer while retaining keyboard Quick Open. If retained, show connection/schema context and explain scope in empty state. | Product IA decision; identity model. | Activity label matches shortcut-list content; origin context disambiguates repeated names; no duplicate grid/object tree is introduced. |
| Proposed P2 | upgrade | `data-activity-feature-research.md:89-98` | Add search/filter/grouping and explicit remove/clear affordances only if chosen list scale/use case warrants them; keep Data as a launcher, not object tree. | Keep-activity decision and structured identity. | User can narrow a long list by documented fields and remove one exact identity without affecting another same-name entry. |
| Proposed P2 | missing | `data-activity-feature-research.md:100-111` | After identity work, consider Copy qualified name. Add query-result references only with metadata-only storage and reopen semantics; provide export shortcut only by invoking canonical Transfer workflow. | Stable identity/provider-aware formatter; explicit result lifecycle; canonical Transfer integration. | Copy yields correct provider-qualified identifier; any result reference stores no implicit row payload and clearly indicates unavailable source; export reaches canonical Transfer with target/format/progress/error feedback. |
| Proposed P2 — release gate | verification | `data-activity-baseline.md:70-84,112-114`; `data-activity-feature-research.md:147-149` | Collect native UI and persistence/reconnect evidence for the chosen activity and identity semantics. | Prior implementation and IA decision. | Verify pin/recent open, structure, query draft, Quick Open, remove/rebind, connection switch, and restart independently on PG and SQLite. |

## Rollout / dependency order

1. Define structured table identity, pin/recent scoping, and legacy persisted-string migration/rebind behavior.
2. Implement that identity consistently across Data, Quick Open, persistence, and row actions; show target context and stale-reference behavior.
3. Route both Data and Explorer query generation through provider-aware identifier formatting; keep generated text as a draft.
4. Decide whether the shortcut list remains a named activity or is consolidated into Explorer; implement the chosen entry-point cutover.
5. Add search/list upgrades only when justified; add reusable result/export references only after their lifecycle and canonical Transfer contracts are explicit.
6. Verify provider and restart scenarios before claiming safe target selection.

## Provider/support matrix

Inputs require independent provider validation and do not report runtime results. Do not infer one provider’s identity or quoting semantics from the other.

| Provider | Source/research statement | Runtime support evidence |
|---|---|---|
| PostgreSQL | Data’s source actions use active connection/schema and bare table names; provider-aware quoting is a proposed requirement (`data-activity-baseline.md:74-76,102-105`). | No multi-connection, stale-reference, persistence, or identifier-quoting run collected. |
| SQLite | Same source-level identity limitation; its naming/quoting semantics are not established by the PG path (`data-activity-feature-research.md:79-83,121-125`). | No SQLite multi-connection, stale-reference, persistence, or identifier-quoting run collected. |

## Verification gates still needed (not run)

- Multi-connection/schema scenario with duplicate table names, active context changes, disconnect/drop/rename, and explicit remove/rebind behavior.
- Persistence/restart and reconnect preserving full identity, separately per PG and SQLite.
- Identifier-generation cases with whitespace, reserved words, embedded quote characters, and delimiters for each provider; confirm the action creates a draft only and inspect safety-classifier behavior before making any exploit claim.
- Native UI traversal: open data/structure, new query, pin/unpin, remove recent, Quick Open, row context, and selected-state correctness.
- If selected, result-reference reopen and canonical Transfer export flow.

No build, tests, lint, provider runtime, or UI traversal was run for this roadmap.

## Out of scope / unresolved decisions

- No source changes or edits to existing baseline/research documents.
- Pending: separate activity versus Explorer consolidation; final label; global/project/per-connection pin and recent scope; migration behavior for legacy string-only pins; query-result reference persistence; whether export shortcut is warranted.
- No duplicate Data Editor/grid, no claim that an injection exploit exists, and no assumption of provider parity.
