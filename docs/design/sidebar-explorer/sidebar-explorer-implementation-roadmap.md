# Sidebar Explorer Implementation Roadmap

- **Source baseline SHA:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b` (reported by the paired feature-research document; the baseline inventory itself does not print a SHA).
- **Input documents:** [Sidebar Explorer Baseline](sidebar-explorer-baseline.md); [Feature Research](sidebar-explorer-feature-research.md)
- Research date: 2026-09-24. Source claims are limited to code inventory at the SHA above. This roadmap does not convert source paths, reference-product documentation, or capability gates into runtime evidence.

## Decision

Preserve the working connection/schema/table navigation and current action integrations; do not replace them with a broad object-browser rewrite. Sequence improvements around reliable scale and truthful provider capability: first make filters and refresh status understandable, then add inspect/navigation workflows and selected-object compare/diagram entry points, then consider object breadth and persisted organization. Keep Generate SQL, Plan mutation, and Execute mutation separate, and retain existing connection/transaction/staged-change/delete guards.

The research's F1–F11 are proposals, not implementation commitments or established severities. Priorities below are explicitly proposed priorities. Provider-specific folders/actions must only appear where introspection and the relevant operation are actually supported.

## Current state

### Implemented (source-observed only)

- Explorer source has connection catalog/lifecycle, database → schema → tables, search input, schema refresh, table data/structure/DDL and SQL-generation entry points; columns, FK and index detail; views, functions/procedures and triggers folders; workspace integrations and several operation guards. Evidence: baseline §5, §10–14 (`explorer_toolbar_view.rs`; `explorer_schema_objects_view.rs`; `explorer_table_details_view.rs`; `explorer_schema_object_folders_view.rs`; `explorer_navigation.rs`).
- Connection rows can connect, disconnect, reconnect, refresh, open ER Diagram, copy identifiers, and enter related workflows; row clicks vary with lifecycle state. Evidence: baseline §6, §12 (`explorer_connections.rs`; `explorer_navigation.rs`).
- Table row actions open data/structure/DDL, generate SQL, modify/drop through Schema Workbench, copy names, refresh, or open Agent. View/function/trigger actions are not identical: function/procedure rows do not offer modify/drop; trigger rows do not offer query/modify. Evidence: baseline §9, §11 (`explorer_details.rs`; `explorer_schema_object_row_view.rs`; `explorer_folders.rs`).
- Tables use offscreen row rendering and a navigation cache; functions are gated by a capability flag. Evidence: baseline §8, §10, §14 (`explorer_schema_objects_view.rs`; `explorer_table_details_view.rs`).
- Input does not establish native UI traversal, PostgreSQL/SQLite runtime behavior, or provider feature completeness (baseline §15; research §9).

### Not implemented / incomplete

- Filter is one search value normalized to lowercase; its main filtering behavior is tables in active schema. Views/functions/triggers are drawn by schema, with no shared per-kind model/modes/clear/filter state in the source inventory. Evidence: baseline §8 (`explorer_schema_objects_view.rs`); research F1.
- User-defined connection folders, complete favorites/pins/recent objects and persistent filter/navigation context are not shown as Explorer features. Evidence: research F2/F11; baseline §14 lists current inventory.
- Unified quick documentation/metadata preview, dependency/usage navigation, and granular folder/object refresh/loading/cancel/retry are not shown. Evidence: research F3/F8/F9; baseline §7–11.
- Object folders/actions are narrower than product references and vary by type; provider-specific object categories/actions cannot be assumed. Research names possible PostgreSQL and SQLite categories but these are recommendations, not evidence of introspector or runtime support (research F4/F5; baseline §11).
- Explorer does not currently expose a source/target compare-selection flow, and ER Diagram opens at connection level rather than selected schema/table. Evidence: research F6/F7; baseline §6, §14.
- Keyboard tree navigation is partial; source inventory identifies some shortcuts but not complete focus/arrow/enter behavior. Persistent width and some egui row state exist, but not comprehensive saved navigation state. Evidence: research F10/F11; baseline §3, §6.

### Needs fix

No P0/P1/P2 correctness defect is reported as an inherited finding by these inputs. Recommendations below are **proposed priorities**, not retroactive severity findings. Preserve the source-observed safety boundaries: a visible action must not imply unsupported SQL, drop must retain confirmation, and connection changes must respect staged-change/open-transaction guards (baseline §6, §11–12; research §7).

## Ordered V3 backlog

| Order / priority | Type | Evidence | Concrete change and outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| 1 · **Proposed P1** | missing | Baseline §8 (`explorer_schema_objects_view.rs`); research F1, §5 | Introduce explicit filter scope and semantics appropriate to each supported object kind; show applied state, clear action and per-folder match counts. Do not silently hide folders with zero matches. Persist filters only after defining connection/schema ownership. | Define filter semantics per kind; metadata available for searchable fields | Filter applies to every declared supported kind, counts match visible objects, zero-match folders remain discoverable with clear reason, clear restores unfiltered results. Unsupported query modes are not accepted as if implemented. |
| 2 · **Proposed P1** | missing | Baseline §7, §12 (`explorer_schema_tree_view.rs`; `explorer_navigation.rs`); research F8 | Make refresh/loading/error/staleness status granular enough for the scopes actually refreshed; preserve current data identity while request is pending, and expose retry/cancel only if runtime supports them. Begin with connection/schema refresh; do not advertise per-folder refresh without per-folder introspection. | Runtime introspection request/result/cancel contract | A refresh visibly identifies scope and pending state; success updates that scope, failure retains a distinguishable error and retry path; stale data is labeled rather than presented as newly refreshed. |
| 3 · **Proposed P1** | upgrade | Baseline §10–11 (`explorer_table_details_view.rs`; `explorer_schema_object_folders_view.rs`); research F3 | Add keyboard-accessible quick metadata preview for metadata already available; identify unavailable fields instead of fabricating details. Extend to object definitions/dependencies only after provider introspection supports them. | 2 for freshness labels; per-object metadata capabilities | Preview opens for a selected object using current metadata; displays only supplied fields and labels unavailable fields; keyboard and pointer paths can open/close it without losing tree context. |
| 4 · **Proposed P1** | missing | Baseline §6, §9, §11 (`explorer_connections.rs`; `explorer_details.rs`; `explorer_folders.rs`); research F6, §6 | Add explicit compare source/target selection from connection/schema/object context, then open Schema Compare with both sides named. Start with scope supported by existing compare model; do not auto-apply or conflate selection with migration execution. | Schema Compare must accept stable source/target context; unsupported-kind decisions | Source and target connection/database/schema/object are shown end-to-end; identical/invalid pair is rejected with reason; opening compare never executes mutation; returning to Explorer retains context. |
| 5 · **Proposed P1 selected-scope; P2 breadth** | missing | Baseline §6 (`explorer_connection_row_view.rs:202-213`); research F7 | Add schema/table-focused diagram actions only if the Diagram workspace accepts and labels scope; selected table can start with related FK neighborhood. | Diagram context/scope contract and graph freshness correctness | Selected scope is explicit in the canvas; only declared objects/relations appear; return to full schema is available; stale graph cannot show a prior connection's objects. |
| 6 · **Proposed P1 for safe action availability** | fix | Baseline §9, §11–12 (`explorer_details.rs`; `explorer_folders.rs`; `explorer_navigation.rs`); research F5, §7 | Audit menu visibility against actual provider capability and runtime action handler. Keep generate/plan/execute distinct; ensure every drop path retains confirmation/dependency policy. Add typed create/modify workbench only for operations whose provider plan is complete. | Provider capability metadata and mutation service contract | For each shown action, invocation reaches its named outcome or a reasoned unsupported state; unsupported provider never receives emitted DDL; Drop cannot bypass confirmation; SQL generation alone has no database side effect. |
| 7 · **Proposed P1 PostgreSQL / P2 SQLite breadth** | missing | Baseline §8, §11, §14; research F4 | Extend schema-object categories only as provider-specific introspection and capability support become real. Candidate PostgreSQL categories include sequences, materialized views, types and extensions; candidate SQLite categories include indexes, triggers, views and supported pragmas/virtual tables. These lists are research suggestions, not provider support claims. | Per-kind provider introspection, capability, rendering and action contracts | PG-only folders are absent on SQLite; SQLite objects are shown only when introspected; every folder's refresh/open/create/modify/drop behavior is gated independently and has runtime evidence before support is claimed. |
| 8 · **Proposed P1 for table/view navigation; P2 for routines** | missing | Baseline §10–11; research F9 | Add go-to referenced table and incoming/outgoing dependency/usage navigation where provider metadata can resolve dependencies. Do not infer complete dependency graphs from visible FK rows alone. | Provider dependency metadata and object identity | Navigation opens the correct qualified object; unavailable dependency kinds are explicitly unsupported; results identify the provider/schema scope and do not claim completeness when metadata is partial. |
| 9 · **Proposed P2** | upgrade | Baseline §6, §9; research F2/F10 | Add predictable connection grouping/favorites and baseline tree keyboard navigation. Treat drag/drop, recent objects and persistent state as separate optional items after scope/ownership decisions. | Persistence model decision for grouping/state | Keyboard arrows, Enter, Space, Escape and focus restoration have consistent tree outcomes; groups/favorites survive the declared lifetime and do not change underlying connection identity or lifecycle guards. |
| 10 · **Proposed P2** | verification | Baseline §15; research §7, §9 | Verify core flows and capability boundaries independently in native UI, automated behavior and provider runtimes. | Relevant backlog items | Evidence distinguishes rendered UI, automated result, PG runtime and SQLite runtime; each declared action/provider combination has a result or explicit unsupported outcome. |

## Rollout / dependency order

1. Deliver filter semantics and truthful refresh status (1–2); use these as prerequisites for trustworthy preview and object expansion.
2. Add preview/navigation (3), then selected-context workflows (4–5) only after the consuming Schema Compare/Diagram surfaces preserve context and freshness.
3. Audit existing action contracts and guards (6) before adding menus or typed mutation affordances.
4. Add object families and dependency navigation (7–8) category-by-category, provider-gated, not as one cross-provider folder set.
5. Add organization and complete keyboard/persistence behavior (9) after state ownership decisions.
6. Execute matrix gates independently per provider and native UI surface (10). Do not infer runtime support from a populated tree or capability flag.

## Provider/support matrix

The baseline documents generic Explorer plumbing, badges and a function capability gate, but does not prove PostgreSQL or SQLite runtime behavior or exhaustive provider support (baseline §6, §8, §11, §15). The research's PG/SQLite lists are proposed object coverage, not validated introspection or DDL support.

| Provider | Source/research claim | Required V3 disposition / runtime gate |
|---|---|---|
| PostgreSQL | Connection display can show `PG`; function folders are capability-gated. Research proposes PG-specific sequences, materialized views, types, extensions, foreign tables/FDW, policies, publications/subscriptions, event triggers and collations; none is thereby proven implemented/supported (baseline §6, §8; research F4). | Verify actual introspection, filtering, refresh, preview, and action behavior for each enabled kind on a PostgreSQL runtime. Render only returned/capability-backed categories; block unsupported DDL and retain existing confirmations/guards. No broad PostgreSQL support claim until tested per category/action. |
| SQLite | Connection display can show `SQLITE`; research proposes indexes, triggers, views and only supported pragmas/virtual tables, explicitly excluding PostgreSQL-only folders. This is not proof of runtime coverage (baseline §6; research F4). | Independently verify SQLite introspection and each enabled category/action. Do not display PostgreSQL-only categories or fallback fabricated objects; expose only operations the SQLite runtime supports. Confirm refresh/error and mutation guard behavior on SQLite. |
| Other providers | Research references other commercial tools but does not establish DB Pro provider support for additional providers (research §2, §7). | No extrapolation from the generic connection display URI or source branches; add a provider only when its own introspection/capability/runtime evidence is available. |

## Verification gates still needed (not run)

- **Automated:** object-kind filtering/count semantics; refresh state transitions and error retention; action-to-outcome/capability gating; drop confirmation; connection switching under staged-change/open-transaction guards. Inputs describe source behavior, not executed tests; none were run for this roadmap.
- **Native UI:** traverse empty/connected/disconnected/failed Explorer states; keyboard/focus behavior; filter results and zero-count folders; refresh loading/error; quick preview; compare and diagram entry points; confirmations. No native UI traversal or screenshot is established by either input.
- **PostgreSQL runtime:** independently exercise each enabled object folder's introspection and refresh, filter, preview, reference navigation, and mutation action. Not run; research object lists remain candidates only.
- **SQLite runtime:** repeat independently for enabled SQLite objects and confirm PG-only folders/actions are absent. Not run.
- **Scale/accessibility:** verify large table lists, offscreen navigation cache, keyboard focus and count consistency; baseline source cache/rendering does not establish rendered performance or accessibility. Not run.

## Out of scope / unresolved decisions

- Full DBeaver/DataGrip/pgAdmin parity; research references are workflow ideas, not requirements (research §2, §7).
- Exact object-category support per provider, metadata completeness, and create/modify/drop operation coverage (research F4/F5).
- Filter regex/owner/comment/system-object semantics and persistence ownership (research F1).
- Connection folder/favorite/recent-object persistence lifetime and drag/drop affordance (research F2/F11).
- Whether object metadata preview includes definitions, row estimates, comments or dependencies by provider/security policy (research F3).
- Compare selection lifecycle and whether source/target selection persists beyond the session; apply remains explicit and separate (research F6, §6).
- Diagram scope/layout persistence and export formats (research F7).
- Granular refresh/cancellation only if runtime supports those contracts; do not add UI affordances without them (research F8).
