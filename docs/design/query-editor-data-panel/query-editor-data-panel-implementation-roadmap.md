# Query Editor and Data Result Panel — implementation roadmap

## 1. Source baseline and inputs

- **Exact source baseline SHA:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`
- **Input documents:** [query-editor-data-panel-baseline.md](query-editor-data-panel-baseline.md), [query-editor-data-panel-feature-research.md](query-editor-data-panel-feature-research.md), [query-editor-data-panel-feature-menu-baseline.md](query-editor-data-panel-feature-menu-baseline.md), [query-editor-data-panel-investment-baseline.md](query-editor-data-panel-investment-baseline.md).
- All DB Pro source facts are limited to that SHA. The menu and investment docs are overlapping product directions, not proof of delivery or additional independent features. Proposed priorities are preserved where stated; inferred correctness blockers remain labeled as such. No provider/UI runtime evidence is implied.
In abbreviated citations, `baseline.md`, `feature-research.md`, `investment-baseline.md`, and `feature-menu-baseline.md` refer to the correspondingly prefixed input files listed above; `research :N` denotes a line anchor in the feature-research input.

## 2. Decision

Invest in the **result surface foundation** while presenting it as the user-visible Results layout direction: keep Bottom as default and add an optional resizable Right dock only after stable document/request/result identity, projection identity and the table-data-vs-query-result edit boundary are preserved. Then bring Record/Value inspection into that surface, add result lifecycle, typed filtering/sorting, SQL object navigation and query-history management, and only then compare/display-mode/export upgrades. This reconciles the research's flexible docking and lifecycle gaps with the menu baseline's order (`Right Results → Inspector → Lifecycle → Filter/Sort → Navigation → History → Compare`), while the investment baseline supplies prerequisite identity/safety work; these are one sequenced roadmap, not duplicate feature sets. (`feature-menu-baseline.md:194-301,313-331`; `investment-baseline.md:41-61,63-148,150-215`; feature research `:108-180`.)

Do not claim the feature complete because a bottom dock or grid already exists. Keep provider-specific capabilities and evidence independent for PostgreSQL and SQLite. Avoid true native-window detach, full Tree/Text/Transpose suite, and compare until product decisions and foundations are settled (`investment-baseline.md:217-263`; `feature-menu-baseline.md:303-311`).

## 3. Current state

### Implemented at source baseline (source fact, not runtime proof)

- Native egui SQL editor with context chrome, document-scoped connection/schema, run/cancel capability feedback, editor buffer/undo, dialect highlighting, search, diagnostics, completion, prediction, signature help, hover, format, save, snippets and bind parameters (`query-editor-data-panel-baseline.md:35-42,44-94,96-220,227-260,262-360`).
- Results are in a **bottom output dock**, with Results, Chart, Messages, Explain and History panes. Dock supports vertical resize/maximize/restore/close; result sets have an active index per document (`query-editor-data-panel-baseline.md:6,402-430`; research `:108-114`; investment `:17-24`).
- Result grid includes virtualized rows, selection, filtering/sorting, column layout, copy/export, record and value inspectors. Table-data editing uses ChangeSet/write policy, distinct from read-only query results (`query-editor-data-panel-baseline.md:402-696,823-832`; investment `:17-24`).
- Explain supports explicit Explain Analyze confirmation because it executes the statement; plan tree/raw fallback and history/replay exist (`query-editor-data-panel-baseline.md:747-785`).

### Not implemented / incomplete (source fact or absent evidence)

- No persistent right-side/in-editor result layout, detach, or side-by-side result surface is shown; current dock is bottom-only (`query-editor-data-panel-baseline.md:6,823-829`; research `:108-128`).
- Results have multiple sets and active index, but no rename/pin/individual close/close-others/detach lifecycle is shown (`query-editor-data-panel-baseline.md:402-430`; research `:146-180`).
- Record/value inspector primitives exist but are not the proposed unified persistent right-side inspector. Typed filter builder and explicit client/server semantics are not present as the proposed contract (`query-editor-data-panel-baseline.md:665-694`; investment `:108-128,168-183`; menu baseline `:212-255`).
- Completion/hover exist, but Ctrl/Cmd-click/F4 object navigation and query outline are not shown (`query-editor-data-panel-baseline.md:163-209`; research `:184-220`). History exists but is not the proposed filtered Query Manager (`query-editor-data-panel-baseline.md:773-785`; investment `:185-199`).
- No native UI runtime or PostgreSQL/SQLite runtime evidence is established; source inventory explicitly says so (`query-editor-data-panel-baseline.md:849-851`; research ends `:749`).

### Needs fix / correctness and safety gates

The investment baseline identifies these as **BLOCKER / correctness** or **BLOCKER / UX**, not completed capabilities; they remain gates before layout/lifecycle expansion:

- **Inferred correctness blocker — stable result identity:** current active-document + active-result-index access is insufficient as identity for right dock/detach/compare; anchor result state to document, request and result identity (`investment-baseline.md:41-44,88-105`).
- **Inferred UX blocker — layout semantics:** define sizing/minimums and interaction contract so editor and output do not compete unpredictably (`investment-baseline.md:43-44,65-85`).
- **Data correctness risk:** client-side filtering/sorting must preserve original row identity so edit/copy/inspect target the correct row (`investment-baseline.md:45,168-183`).
- **Safety boundary:** query results stay read-only; only table-data mode with provider/write policy may stage edits (`investment-baseline.md:46,130-148`; baseline `:825-832`).
- **Provider capability risk:** cancel, parameters, explain, edit and export must be checked independently by provider; never infer from PostgreSQL to SQLite (`investment-baseline.md:47,130-148`).

## 4. Ordered V3 backlog

Types distinguish repairs to correctness/safety (`fix`), absent capability (`missing`), incremental capability (`upgrade`) and evidence not run (`verification`). Research priorities (P1/P2) are retained; proposed priorities are explicitly marked.

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 (research) | missing | Output is bottom-only; right-side request is explicit: `feature-research.md:108-142`; menu baseline invest 1 `feature-menu-baseline.md:198-210` | Deliver user-visible Bottom/Right results layout with Bottom default, horizontal resize, min/max sizing, close/reopen/maximize/restore in either orientation; no true detached window in this phase. Persist only the state agreed by product. | Stable identity row below is foundational despite this appearing first in user-facing investment order; layout semantics and persistence decision. | Switching orientation preserves active document/result, selection and staged table-data state; panel resize respects editor/output minimums; closed/maximized/restored states work in both orientations. |
| Proposed P1 (inferred blocker) | fix | Identity risk: `investment-baseline.md:41-44,88-105`; multi-result source: `baseline.md:402-430` | Establish stable document + execution/request + result identity; route output tabs, selection and inspector state by that identity rather than active index alone. | Existing document/request/result model inventory; precedes layout/lifecycle/compare. | Two documents running identical SQL and multiple result sets remain isolated across tab/layout switches; return to a document restores its own active result; no state follows a reused numeric index into another execution. |
| Proposed P1 (inferred risk) | fix | Projection/edit risk and explicit safety boundary: `investment-baseline.md:45-47,130-148`; baseline `:825-832` | Preserve original row identity through client-side sort/filter; enforce query-result read-only vs table-data ChangeSet edit boundary; display provider/write-policy reason for unsupported edit/cancel/explain/parameter actions. | Stable row/result identity; provider capability inventory. | After filter/sort, copy/inspect/edit targets the displayed original row; query result has no mutation action; unsupported actions emit no command and explain why; layout changes do not discard staged edits. |
| P1 (research) | missing | No result naming/pin/close/detach: `feature-research.md:146-180`; menu baseline `:226-239` | Add result naming (statement/table/alias default plus rename), pin, close current/others, statement source/range, and reopen from history. Treat detach only as dock/surface behavior in this stage. | Stable identity and result surface layout. | Rename/pin/close actions affect only selected document/request/result; close does not erase query/history; statement range/source is shown and reopen resolves the intended result context. |
| Proposed P1 | missing | Existing inspectors are primitive; user direction right-side Record/Value: `investment-baseline.md:108-128`; menu baseline `:212-224`; baseline `:665-694` | Unify Record and Value inspector in resizable right-side area; retain Raw/Pretty/Tree JSON and Raw/Hex/Base64 bytes, field navigation, selected-row identity and visible write-policy state. | Right results surface; stable row identity and edit boundary. | Selecting a row/field opens corresponding inspector without changing selected record; all existing value modes remain available; apply affects ChangeSet only, never executes; filter/sort does not retarget inspector. |
| P1 (research for selected items) / proposed P1 | missing | Research prioritizes row count/direct export P1: `feature-research.md:224-262`; baseline shows current execution actions `:96-118,696-716` | Add current-statement execution as a clear cursor action; evaluate efficient row-count and direct query export proposals separately against providers and long-running/cancel behavior. Keep existing selection/all execution. | Execution target visibility; capability contracts; do not conflate with results layout. | Current-statement action runs only statement under cursor; row count does not materialize full results if claimed; export states selected/visible/all scope and handles large results without silent truncation. Unsupported provider path is explicit. |
| P2 (research) | missing | Proposed typed builder and client/server distinction: `investment-baseline.md:168-183`; menu baseline `:241-255`; research `:266-301` | Add typed filters for text/number/date/boolean/NULL with AND/OR, predicate preview, client/server mode and clear-all. Keep query-result filters from modifying source SQL; parameterize table-data server predicates. | Stable row identity and edit boundary; provider predicate capability. | NULL/empty/large numeric boundaries preserve semantics; filter mode and loaded/matched/total scope are clear; query SQL unchanged; server predicate is parameterized; copy/edit/inspect preserve original target. |
| P1 (research) / proposed P1 | missing | Editor completion/hover exist, navigation absent: `feature-research.md:184-220`; menu baseline `:257-270`; investment `:201-215` | Add safe Go to definition (Ctrl/Cmd-click/F4/action), open data/copy qualified name and query outline over resolvable symbols. | Provider-aware schema symbol resolution and source ranges. | Unresolved/ambiguous tokens never open the wrong object; navigation keeps connection/schema and does not alter cursor/selection; PG and SQLite resolution verified separately. |
| P2 (research) / proposed P2 | upgrade | Existing simple history pane vs Query Manager direction: `baseline.md:773-785`; investment `:185-199`; menu baseline `:272-285` | Extend history filters by connection/schema/status/time and outcome metadata; offer replay in original or current context without automatic execution; bounded retention/list. | Stable document/request context; explicit replay behavior. | Replay-original never silently runs on a different connection; filter does not delete entries; failures retain error summary; replay requires the normal run action/confirmation policy. |
| P2 (research) / proposed P2 | upgrade | Compare should follow stable identity/lifecycle: `investment-baseline.md:217-227`; menu baseline `:287-301` | Add compare for selected result sources/key columns and added/removed/changed output; export diff; generated reconciliation script is review-only, never auto-applied. | Result identity, lifecycle, stable row identity, explicit compare scope. | User can identify both sources and key; diff categories are reproducible; generated script is not executed; result compare is not mislabeled schema compare. |
| P2 / deferred | upgrade | Modes absent as unified feature: `feature-research.md:266-301`; menu explicitly says not invest full suite yet `feature-menu-baseline.md:303-311`; investment `:229-245` | After core surface, evaluate Transpose first and value-editor integration; defer full Tree/Text modes and streaming/background export until consumer and provider contracts exist. | Result identity/layout; user/product demand and performance/provider contract. | Each approved mode preserves result/row identity and selection; large-result operations expose scope/progress/cancel and do not freeze the UI; no mode is represented as present before UI delivery. |
| Proposed P1 | verification | All sources mark runtime evidence absent: `baseline.md:849-851`; research end `:749`; investment `:288-324` | Capture native UI vertical slices for docking, multi-document identity, selection/filter, inspectors, edit boundary and lifecycle; separately exercise providers. | Corresponding feature slices implemented; isolated DB fixtures. | Artifacts show observable interactions and state transitions; tests alone or code presence do not qualify native UI. |

## 5. Rollout and dependency order

User-facing sequence follows the menu investment baseline, with technical prerequisites made explicit rather than treated as duplicate product investments:

1. **Right Results surface:** first define stable document/request/result identity and layout contract, then ship Bottom/Right dock (Bottom remains default). This is menu Invest 1; foundation M0.1/M0.2 dependencies are prerequisites, not extra menu features.
2. **Record/Value inspector:** add right-side shared inspector only once surface, row identity, projection correctness and query-result/table-data write boundary are safe (menu Invest 2; investment M0.3/M0.4).
3. **Result lifecycle:** add rename/pin/close/source/reopen after stable identity; no result-index-only routing.
4. **Typed filter/sort:** distinguish client/server and preserve row identity; validate parameterized provider operations.
5. **SQL object navigation:** build on existing completion/hover and provider-aware resolution.
6. **Query history manager:** add context-aware filter/replay with no silent execution or wrong-connection replay.
7. **Compare:** only after result identity, lifecycle, row projection and filter semantics are reliable.
8. Consider Transpose/value editor, background export, broader view modes and plan comparison only after an explicit consumer decision and provider-specific feasibility. True multi-window detach remains deferred.

## 6. Provider/support matrix

The source baseline and research mention PostgreSQL and SQLite separately; source paths or a shared UI are not proof of either runtime. Keep every provider gate independent. Fill capability results only from actual runs; do not borrow claims across columns.

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Docking, selection and local presentation | UI-only; native UI exercise still required; database operation not applicable. | UI-only; native UI exercise still required; database operation not applicable. |
| Query execution, cancellation, current statement and row count | Runtime/capability verification not present in inputs; test provider-specific behavior and explain unsupported operations. | Runtime/capability verification not present; independently establish behavior and unsupported operations. |
| Explain / Explain Analyze | Explain pane and confirmation path exist in source; actual provider plan/runtime behavior not established (`baseline.md:747-765,825-851`). | Must independently establish explain support, plan representation and analyze safety; do not infer from PostgreSQL. |
| Table-data edit / ChangeSet and typed server filter | Source has write policy/ChangeSet, but provider runtime matrix not run (`investment-baseline.md:130-148`). | Independently verify write policy, parameterized predicate and provider behavior; no extrapolation. |
| Export | Source includes CSV/JSON/Markdown/INSERT and PostgreSQL-style COPY helpers (`baseline.md:696-716`); runtime behavior not qualified. | Verify independently, especially SQL dialect/quoting and supported format paths. |
| Object navigation/history/compare proposals | Provider-aware resolution and execution context need separate evidence; currently unverified. | Provider-aware resolution and execution context need separate evidence; currently unverified. |

## 7. Verification gates still needed (not run)

- **Not run:** native UI right/bottom layout, resize/minimum constraints, maximize/close/reopen, cross-document/multi-result identity, inspector and state preservation.
- **Not run:** correctness of row identity through filters/sorts, query-result read-only policy, table-data ChangeSet behavior across layout changes.
- **Not run:** PostgreSQL execution/cancel/Explain/Explain Analyze/edit/filter/export/object resolution scenarios.
- **Not run:** SQLite execution/cancel/Explain/Explain Analyze/edit/filter/export/object resolution scenarios. SQLite must be qualified independently.
- **Not run:** history replay context, result lifecycle persistence, large-result export or compare; feature ideas are not runtime proof.
- **No tests/build/lint/formatter were run for this roadmap.** The four inputs identify source/document review and proposals, not UI or database runtime verification (`baseline.md:849-851`; `feature-research.md:749`; `investment-baseline.md:385`).

## 8. Out of scope / unresolved decisions

- No source, baseline/research/menu/investment docs or plans changed. No claim that any proposal is implemented or provider-supported.
- Product decisions remain open: right panel default versus option; layout persistence scope; true detach versus dock-only; compare source types (result/table/schema); and whether result state persists between sessions (`investment-baseline.md:32-39`).
- Keep out of this roadmap's initial vertical slice: true native-window/multi-window detach, full DataGrip-style Tree/Text/Transpose suite, broad export-format expansion, plugin extractor framework, Explain plan compare, full visual builder rewrite, unrelated editor polish (`investment-baseline.md:256-263`; `feature-menu-baseline.md:303-311`).
- No provider support may be inferred from shared code paths or another provider's verification.
