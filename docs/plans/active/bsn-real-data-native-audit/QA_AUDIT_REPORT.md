# DB Pro QA Audit Report

**Audit type:** Read-only source, test, configuration, existing-evidence, and live BSN native UI review  
**Reviewed revision:** `cb6e00d633772de538498384a8b14d32d25f7915` plus the dirty working tree  
**Evidence basis:** Source/configuration review plus a bounded, non-mutating session to the saved BSN profile and Xvfb native screenshots. The repository already had unrelated/uncommitted changes before the audit. Source changes from a temporary regression experiment were removed before handoff; no product source edits are retained.  
**Production data:** BSN connection was used for connection/schema metadata and Table Data reads only. No DML, DDL, export, backup, transfer, admin, or settings operation was issued; no row values were retained in evidence.

## Executive priority

- **P0: 0. P1: 2 open verified defects:** targeted retry can discard unrelated staged edits after success (source-confirmed); switching BSN tables reuses the previous table’s filter and fails the new read (reproduced live).
- **P2: 22 findings:** 16 verified source/live/configuration or test-coverage findings, including 6 live BSN UI/safety-affordance findings and 1 synthetic-only status-bar collision; 6 source-traced runtime/performance risks remain suspected. No production-data mutation was performed.
- Verified live BSN UI findings now include clipped PostgreSQL types, a blank ER canvas despite a positive match count, stale Security success text, Drop actions for predefined roles, clipped Schema Workbench actions, and the synthetic-seed INSERT affordance with no Production confirmation on the connected profile. The status-bar collision remains synthetic-only.
- Full application-wide focus traversal, accessibility-tree and screen-reader behavior, visual i18n, dark mode, wide viewports, several feature details, and provider-specific failure cases remain NOT VERIFIED; per-feature status is in `docs/plans/active/bsn-real-data-native-audit/CHECKLIST.md`.

`VERIFIED` means the source/configuration proves the behavior or test gap; it does not mean a live provider reproduction was performed. `SUSPECTED` marks a plausible runtime impact that requires a real GUI/provider/benchmark reproduction. Coverage gaps are not presented as product defects.

## Findings

### P1 — VERIFIED source defect: Targeted retry discards unrelated staged edits

**Locations:** `crates/app/src/table/table_mutation_state.rs:134-146`; `crates/app/src/table/table_mutation_actions.rs:415-428,524-537`; `crates/app/src/runtime/operation_events.rs:378-395`.

**Evidence:** A retry target is taken and `build_apply_plan` filters the plan to that one target (`table_mutation_state.rs:134-145`). On successful completion, `on_table_row_operation_completed` routes the matching staged request to `staged_apply_completed()` (`operation_events.rs:384-385`). That completion unconditionally clears the entire `staged_changes` set and permits pending navigation (`table_mutation_actions.rs:524-537`). The failure path preserves the full set; existing tests verify preservation after failure but not after a targeted retry succeeds (`crates/app/src/app/tests/app_tests.rs:6332-6450`; retry-success scenario absent from the inspected tests).

**Failure scenario:** Stage changes for rows A and B; the transaction fails on B and is rolled back; reload/fix B and retry; only B is submitted, but successful completion clears A and B locally and reports all changes applied. A’s unsubmitted edit is silently lost, and pending navigation may proceed.

**Impact:** Loss of unsaved user work; success state does not match the mutations sent to the database. Shared app-state path affects PostgreSQL and SQLite after their transaction failure is mapped into this retry flow; neither provider was exercised for this sequence in this audit.

**Recommended fix:** Track the exact staged change identities/revisions included in the submitted retry and remove only those on success. Keep unrelated staged edits and their navigation guard. Add a multi-row fail → targeted retry → success regression that asserts both database state and remaining staged state.

**Verification:** Source-traced; no runtime or provider call made. This finding was not established by the previously read reports and contradicts the prior plan’s claim that unrelated staged rows remain untouched after a targeted retry.

---

### P2 — VERIFIED source defect: Invalid added-column definition is silently omitted from table ALTER plan

**Locations:** `crates/core/src/application/object_mutation_builders.rs:39-45,104-136,197-207`; `crates/core/src/application/ddl_builder.rs:64-87,618-639`.

**Evidence:** `build_alter_table_add_column` returns `Result` and rejects invalid raw type/default fragments. `alter_existing_table` handles that result with `if let Ok(add)` and silently continues on `Err`. It then still appends other schema changes, including drops for baseline columns absent from the edited definition (`object_mutation_builders.rs:197-205`).

**Impact:** A mixed table edit can omit an invalid new column while retaining other DDL, so the plan may represent only part of the user’s requested schema change. The user may see the incomplete preview, but the builder should fail rather than convert a validation error into a partial plan.

**Recommended fix:** Return `Result<Vec<String>, MutationError>` from the table ALTER planner and propagate builder errors. Add a regression with an invalid new-column fragment plus another requested schema change; assert planning fails and produces no executable partial plan.

**Verification:** Source-only; no DDL was run.


### P2 — VERIFIED source risk: Read-only query policy classifies effectful `SELECT` as read-only

**Locations:** `crates/core/src/domain/connection.rs:160-163`; `crates/core/src/domain/safety.rs:47-50,78-89`; `crates/core/src/domain/safety_policy.rs:53-76`; `crates/core/src/application/query_service.rs:143-177`.

**Evidence:** The read-only flag is explicitly enforced at the application layer. The classifier labels `SELECT` as `Read` unless it finds top-level `INTO`; accepted SQL is passed to the provider. It is documented as a best-effort heuristic, not a full SQL parser. A side-effecting database function called from `SELECT` is therefore allowed by this policy if the database credentials permit that function to mutate state. No provider-level read-only transaction/session enforcement was found in the reviewed connector source.

**Impact:** The UI’s read-only mode is not a security boundary against effectful SQL under write-capable credentials. Actual exploitability depends on provider semantics and the privileges of the configured database role; no function was invoked in this audit. Project security documentation describes this as application-layer enforcement, so this is reported as a limitation/risk, not as a reproduced unauthorized mutation.

**Recommended fix:** Do not present the app flag as authorization. For a hard guarantee, use read-only database credentials or provider-enforced read-only transactions; add provider-specific tests showing side-effecting `SELECT` is rejected when the product claims strict read-only behavior.

---

### P2 — VERIFIED source regression: MySQL table-copy path omits the configured query timeout

**Locations:** `crates/native-app/src/table_copy_port.rs:59-78`; `crates/infrastructure/src/mysql/connector.rs:317-361,549-671`; `crates/core/src/domain/connection.rs:144-147`.

**Evidence:** The current table-copy adapter forwards copy statements through `execute_parameterized_transaction`. MySQL `execute` wraps execution in the configured timeout (`connector.rs:322-357`), but `execute_parameterized_transaction` begins a transaction and awaits each statement and commit without loading or applying that timeout (`connector.rs:558-671`).

**Impact:** A copy INSERT/DELETE waiting on a MySQL lock can outlive `query_timeout_ms` and remain pending until the server/driver resolves it. This is an interaction between the current table-copy path and an existing provider timeout omission; it is not a claim that a live timeout was reproduced.

**Recommended fix:** Apply the configured deadline to the full parameterized transaction, with safe rollback and truthful failure-outcome reporting. Add a disposable-MySQL locked-write test through the table-copy adapter.

**Verification:** Current source reviewed; no MySQL service was available or contacted. Existing MySQL integration tests do not exercise a locked parameterized transaction.

---

### P2 — VERIFIED CI configuration defect: The ignored-test command runs provider tests without their fixtures

**Locations:** `.github/workflows/ci.yml:18-34,134-135`; `crates/infrastructure/tests/mysql_integration.rs:517-528`; `crates/infrastructure/tests/sqlserver_integration.rs:156-166`.

**Evidence:** The workflow provisions PostgreSQL only and sets a PostgreSQL `DATABASE_URL`, then runs `cargo test --all -- --include-ignored`. The MySQL cross-database integration test expects `setup()` to return a live MySQL connector and calls `expect` at line 526; it also requires `MYSQL_REFERENCE_DATABASE`. The SQL Server foreign-key test calls `expect` on a live SQL Server setup at line 165, but the workflow does not configure `SQLSERVER_URL`.

**Impact:** If the CI job reaches the integration-test step as configured, these ignored tests are activated without their provider fixtures and can panic instead of skip. The ordinary workspace test run that excludes ignored tests does not validate this CI step.

**Recommended fix:** Run provider integration suites only in jobs that provision that provider and its disposable fixture; keep the generic CI job on non-ignored tests. Make fixture absence an explicit skip only for intentionally optional local runs, not for a provider job that claims to execute the test.

**Verification:** Workflow and test control flow inspected; CI was not run.

---

### P2 — VERIFIED test false-green: SQL Server integration setup treats every connect error as a skip

**Locations:** `crates/infrastructure/tests/sqlserver_integration.rs:49-62,73-79,103-113,116-122`.

**Evidence:** `setup()` converts any `connector.connect(...).await` error to `None` with `.ok()?`. Several tests interpret `None` as “SQLSERVER_URL is unavailable” and return normally. This includes authentication, TLS, network, and provider failures after a URL was configured.

**Impact:** Individually selected SQL Server integration tests can report success without assertions when a configured fixture is broken. The foreign-key integration test uses `expect` and fails instead, so this is not a claim that every SQL Server test false-passes.

**Recommended fix:** Skip only when `SQLSERVER_URL` is absent. If configured, propagate connection errors as test failures. Keep the provider-specific CI job responsible for supplying the fixture.

**Verification:** Source-inspected; no SQL Server connection attempted.

---

### P2 — SUSPECTED concurrency race: A late connection attempt can restore a disconnected session

**Locations:** `crates/core/src/application/connection_service.rs:240-280,283-296`; `crates/runtime/src/worker.rs:1537-1597`; `crates/app/src/connection/connection_events.rs:69-87`; `crates/app/src/explorer/explorer_connection_row_view.rs:152-186`.

**Evidence:** Connect is detached and registers its provider handle after awaiting connect. Disconnect is handled on the command loop and treats “not found” as a completed disconnect. The row context menu offers Connect while the row is not connected, without checking an in-flight connection. `on_connected` rejects only a response whose request id conflicts with a still-present pending request; when no request is pending, it accepts the response.

**Failure scenario:** Start a slow connect, start a second connect that completes first, disconnect that active handle while the first attempt remains pending, then let the first attempt complete. The late attempt can register a handle and its event can restore connected UI state after the user disconnected.

**Impact:** A user-requested disconnect may not remain effective under overlapping connection operations. This is a source-traced race, not a reproduced provider bug.

**Recommended fix:** Serialize lifecycle operations per connection or add a connection-generation/cancellation token invalidated by disconnect; close handles from superseded attempts. Add deterministic tests controlling connect completion order and assert both registry and UI state after disconnect.

---

### P2 — SUSPECTED responsiveness risk: Runtime worker awaits table reads in its single command-dispatch loop

**Locations:** `crates/runtime/src/worker.rs:734-748,837-863` (dispatcher spans approximately lines 681-2762).

**Evidence:** The worker receives and matches commands serially. `IntrospectSchema` spawns an async task, but `LoadTableData` awaits `fetch_rows` directly before receiving the next command. Similar inline awaits exist for connection listing, table-info/DDL, and DDL execution.

**Impact:** A slow remote table fetch can delay later commands queued on the worker, including unrelated connection/query work. The UI thread is not shown blocking here; the risk is runtime command head-of-line blocking. Provider timeouts bound some calls, but may still exceed an interactive wait.

**Recommended fix:** Move long provider calls to cancellable per-request tasks while retaining request-generation checks and per-connection ordering where required. Add a deterministic worker test where a held table fetch does not prevent a subsequent independent command from completing.

**Verification:** Source-only; no latency or scheduling measurement.

---

### P2 — SUSPECTED performance risk, previously documented and reverified: Query output clones the whole active result per frame

**Locations:** `crates/app/src/query_workbench/query_view.rs:278-285`; `crates/ui/src/runtime/runtime_query_types.rs:12-26`.

**Evidence:** Each output-dock draw clones `active_result()` before selecting/drawing the output pane. The result owns nested rows and cell strings, so the clone duplicates the full result even when the selected pane is Messages or History.

**Impact:** Allocation and copy cost scales with all returned cell data on each dock repaint and can undermine row virtualization. A prior benchmark at revision `3dd988e2ff06ac71942c7ec331acf66389d3c5cb` measured clone-only cost on its fixture; that historical number is not current frame timing. This finding was already in `query-editor-large-schema-audit/FINDINGS.md` and remains present in current source.

**Recommended fix:** Borrow the active result through a split view context or use a scoped move/restore pattern; avoid cloning the result merely to choose an output pane. Measure frame latency and allocations on large results.

**Verification:** Source rechecked; no current benchmark run.

---

### P2 — SUSPECTED performance risk: Grid ellipsizing lays out complete large cell values

**Locations:** `crates/app/src/result_grid/result_grid_cell_surface_view.rs:125-150,154-174`.

**Evidence:** Each rendered text cell calls `ellipsize_to_width`, which copies the complete string and creates a galley for it before calculating the visible prefix. The hover/clipped check may also lay out the full value for shorter strings. Painting clips the visible result, but does not bound the input allocation/layout work.

**Impact:** Large text/JSON cells in the visible viewport can create repeated large temporary strings and process the entire value on repaint. For example, 40 visible 1 MiB text values require at least 40 MiB of full-string copies for the first ellipsize pass per paint, before accounting for galley storage. Actual frame impact is unmeasured.

**Recommended fix:** Bound the preview before copying/layout, Unicode-safely; retain the full value for an explicit details/hover action. Add a benchmark that exercises this render helper with large values and report allocation/frame cost.

**Verification:** Static only; no allocation or frame benchmark run.

---

### P2 — SUSPECTED performance risk: Enabled scheduled tasks force 50 ms repaint polling while idle

**Locations:** `crates/app/src/app/app_lifecycle.rs:144-171`.

**Evidence:** `prepare_frame` requests another repaint after 50 ms whenever any saved task has an enabled schedule, independent of its next deadline. The frame calls the scheduler before making that decision.

**Impact:** A single hourly task can cause approximately 20 repaint opportunities per second between executions, with corresponding scheduler/shell work and likely idle CPU/battery cost. This is a cadence calculation from source, not a measured frame rate or CPU result.

**Recommended fix:** Request repaint at the next scheduled deadline; keep short polling only while actual background work is in flight. Add an idle wakeup/CPU test for an enabled distant schedule.

**Verification:** Source-only; no idle profile.

---

### P2 — SUSPECTED performance risk: ER search lowercases table/column names during redraw

**Locations:** `crates/app/src/diagram_surface/diagram_view.rs:77-91,164-186,368-375`; `crates/app/src/app/app_types.rs:356-363`.

**Evidence:** `measure_diagram_frame` recomputes the listed-table count on each frame. Search matching allocates lowercase copies of table, schema, and column names; subset resolution separately scans graph nodes when search is active. A large catalog with a rare/no match traverses most names for each redraw.

**Impact:** On the observed 1,300-table / roughly 39,000-column catalog scale, even the table-summary count pass can lower-case many thousands of names per frame. Graph-node scan adds work when those nodes are materialized. This is a likely pan/zoom/typing overhead, not measured on the current runtime.

**Recommended fix:** Cache normalized names/search matches keyed by schema/design generation and normalized query; invalidate when either changes. Measure active-search pan/zoom on a large schema.

**Verification:** Source-only; no frame profile. The existing ER benchmark covers scene/index work, not these surrounding view-level passes.

---

### P2 — VERIFIED localization defect: Result-grid empty state bypasses locale catalogs

**Locations:** `crates/app/src/result_grid/result_grid_view.rs:51-60`.

**Evidence:** The no-column result path passes literal English strings “Statement completed” and “This statement did not return any rows to display.” directly to the empty-state component, rather than looking up locale keys.

**Impact:** Users using supported non-English locales see English copy in this query/result state. Catalog key parity tests cannot detect literals that never request a key.

**Recommended fix:** Add locale keys with translations for all supported locales, use the locale lookup in this state, and add a hard-coded user-visible-string check or targeted locale test.

**Verification:** Source-level user-visible strings verified; no GUI run. This is one confirmed instance, not an exhaustive hard-coded-string inventory.

---

### P2 — VERIFIED performance-tooling defect: Native perf scan points at obsolete UI benchmark package

**Locations:** `.skills/perf-audit/scripts/perf-scan.sh:196-209`; `crates/app/Cargo.toml:26-31`; `crates/app/benches/result_grid_benchmarks.rs:117-133`.

**Evidence:** The script checks for `crates/ui/benches/result_grid_benchmarks.rs` and runs Criterion for `db-pro-ui`. The current benchmark is registered in `crates/app` under package `db-pro-app` at `crates/app/benches/result_grid_benchmarks.rs`.

**Impact:** The native UI performance scan cannot invoke the current result-grid suite, weakening repeatable performance qualification. The runner was not executed in this audit.

**Recommended fix:** Update the benchmark path/package and baseline references to the current `db-pro-app` target; run the corrected script in its normal performance workflow.

---

### P2 — VERIFIED architecture debt, previously documented and rechecked: Feature methods still use `DbProApp` as a shared mutable facade

**Locations:** `crates/app/src/agent/agent_actions.rs:30-51`; `crates/app/src/explorer/explorer_connections.rs:61-83`; prior evidence `docs/plans/active/native-core-architecture/FINDINGS.md:384-386`.

**Evidence:** Feature modules continue to implement view/action methods directly on `DbProApp`; examples reach across agent, table, query, workspace, and connection state through the root app. The architecture plan records moving remaining view/reducer APIs to feature-owned contexts as unresolved.

**Impact:** Sibling features can couple to the full composition-root state instead of narrow contracts. This increases change radius and makes feature tests/setup less isolated. No specific wrong runtime result is claimed.

**Recommended fix:** Continue extracting narrow feature contexts and state-owned transitions at touched boundaries; avoid a broad rewrite without a concrete dependency seam. Keep this finding open until the residual `impl DbProApp` cross-feature access is measured/retired.

---

### P2 — COVERAGE GAP: Existing grid benchmark does not exercise cell rendering or scroll paint

**Locations:** `crates/app/benches/result_grid_benchmarks.rs:117-133`; rendering path `crates/app/src/result_grid/result_grid_body_view.rs:68-120`.

**Evidence:** `materialize_100_visible_rows` sums each selected row’s length. It does not materialize cells, call the cell layout helper, paint the grid, or exercise desktop scrolling. The current suite therefore does not qualify the large-cell rendering path implicated above.

**Impact:** Existing benchmark passes cannot establish acceptable scroll/paint performance for large values or realistic grid frames.

**Recommended fix:** Add a deterministic benchmark/smoke measurement that exercises the cell preview/layout and visible grid path with both ordinary and large text values; report actual allocations or frame time. This is a coverage gap, not a reproduced slowdown.

---


### P1 — VERIFIED live UI and source defect: switching tables reuses the previous table’s data filter

**Locations:** `crates/app/src/workspace/workspace_actions.rs:80-121`; `crates/app/src/table/table_data_view.rs:12-27`.

**Evidence:** Connected the native app to the saved BSN profile, opened `master.m_all_bank`, then navigated to `master.m_article`. The new Data view showed its own `article_cd` filter placeholder and the normal-looking `m_article has no rows` / `0 rows` empty state, while the footer/toast reported that the prior table’s `all_bank_cd` column did not exist. No row values were retained. Source confirms `open_table_in_schema` updates `selected_table` and dispatches a new table-data read without resetting `TableDataQueryState`; `build_load_data_command` forwards the retained filters and sorts.

**Impact:** A normal table switch fails the new read and presents the failure beside an empty-table state, so users can mistake an unsuccessful fetch for an actually empty table. The toolbar is scoped to the new table while the request still uses prior-table state.

**Recommended fix:** Reset table-scoped query filters, sorts, inline result, page and error before dispatching the new table’s read. Add a regression that switches between tables with different key columns and asserts the second request contains no prior-table filter. The audit did not retain a source fix.

**Runtime evidence:** `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/bsn-table-switch-error-light-1280x800.png`; live empty result `.../bsn-master-empty-light-1280x800.png`; connection/tree evidence and SHA-256 manifest `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/MANIFEST.md`. Screenshots are pre-fix (the source remains unfixed at handoff).

### P2 — VERIFIED live UI layout issue: BSN column data types are clipped in the Structure table

**Location:** Table workspace → `master.m_all_bank` → Structure, 1280×800; fixed 200 px Data Type column at `crates/app/src/table/table_structure_surface_view.rs:162-179`; screenshot `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/bsn-table-structure-master-light-1280x800.png`.

**Evidence:** The live BSN metadata table showed `character varying(50…)` and `timestamp without t…` in the Data Type column at 1280×800 while other columns and the Actions column were visible. Hovering one type showed a tooltip with the full `character varying(50)` value (`bsn-table-structure-type-tooltip-light-1280x800.png`), so the information is recoverable but not scannable from the table.

**Impact:** Users cannot compare full PostgreSQL types at a glance; long type names require hover. This is a verified non-blocking readability/layout issue. The hover tooltip mitigates the information loss; 1440×900, 1920×1080, dark mode, keyboard tooltip access and other table schemas were not tested.

**Recommended fix:** Give the type column an adaptive/minimum width or a clearly discoverable keyboard-accessible full-value disclosure; preserve the current hover tooltip. Verify at all required viewports before changing widths.

### P2 — VERIFIED live ER search usability defect: matching BSN tables are counted but no node is visible

**Location:** ER Diagram → search `m_all_bank` → 1 hop, at 1280×800; source path `crates/app/src/diagram_surface/diagram_view.rs:164-186` and `crates/app/src/diagram_surface/diagram_canvas_view.rs:302-373`.

**Evidence:** On the live BSN schema (1,302 tables, 76 FK relationships), entering `m_all_bank` changed the badge to “5 tables.” The canvas remained an empty grid after selecting 1 hop, manually increasing zoom to 100%, and waiting five seconds. The retained image is `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/bsn-er-diagram-filtered-search-blank-canvas-light-1280x800.png`. The source intends to resolve matching graph nodes to a BFS neighborhood and fit the viewport after the search debounce; the live visual result did not show any node. The exact cause was not isolated.

**Impact:** A user receives a positive match count but cannot see or select the matching tables in the diagram. This blocks using the large-schema search path as observed; no all-tables layout was forced.

**Recommended fix:** Reproduce with a deterministic large-schema GUI test; ensure a non-empty match set is visible after search debounce and 1-hop selection, including auto-fit/pan reset for persisted layouts. Preserve the large-schema opt-in so searching does not require laying out all 1,302 tables.

**Verification:** Live BSN UI observation only; 1-hop, manual zoom, and elapsed wait exercised. No diagram code or production database was modified.

### P2 — VERIFIED live UI/source defect: Security success feedback remains on unrelated activities

**Locations:** `crates/app/src/shell/feedback_state.rs:3-14`; `crates/app/src/connection/connection_status.rs:154-163`; `crates/app/src/shell/chrome/shell_chrome_view.rs:120-148`; `crates/app/src/shell/chrome/shell_statusbar_view.rs:113-121`.

**Evidence:** `FeedbackState.runtime_message` stores an unscoped string without age/activity metadata. The status selector returns any non-empty message, and shell chrome passes it to the status bar on every activity. After a safe BSN role-metadata refresh, the footer still displayed `Security · 18 role(s)` after navigating to Tools/Welcome. See `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/bsn-tools-hub-live-light-1280x800.png`.

**Impact:** The status bar communicates stale activity success as though it describes the currently selected surface; it can mislead users about which action just completed.

**Recommended fix:** Scope status feedback to its activity/request and clear or expire non-error success on navigation; retain actionable errors with explicit source context. Add a reducer/UI regression proving stale activity feedback is absent after switching surfaces.

**Verification:** Reproduced live against BSN with only role metadata refresh and navigation; no write or SQL was issued.

---

### P2 — VERIFIED live UI/source affordance: PostgreSQL predefined roles show a destructive Drop action

**Locations:** `crates/core/src/domain/user.rs:3-10`; `crates/app/src/security/security_roles_view.rs:43-86`; confirmation path `crates/app/src/security/security_confirmation_view.rs:19-29,56-70`.

**Evidence:** The domain role model has no provider-defined/system-role kind. The view renders the red Drop action for every listed role. The live BSN PostgreSQL role list included predefined roles, and the privacy-redacted one-row capture shows a Drop button beside a role: `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/bsn-security-role-drop-one-row-crop-live-light-1280x800.png`. The explicit confirmation remains present. No Drop action or server response was exercised.

**Impact:** The UI offers a destructive action for provider-defined roles that should not be droppable, inviting a predictable failed operation and exposing a misleading capability. This is an affordance finding, not a claim of confirmation bypass or successful deletion.

**Recommended fix:** Preserve provider role kind/capability in the security model and disable or omit Drop for predefined/system roles with a clear explanation; retain confirmation for roles that can be dropped. Add provider-backed rendering/capability tests.

**Verification:** Live affordance and source rendering verified; no destructive action activated.

---

### P2 — VERIFIED live UI layout issue: Schema Workbench view actions are clipped

**Location:** Schema Workbench → Views on the live BSN session, Light 1280×800; `crates/app/src/schema/schema_workbench_form.rs:403-442`; crop `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/bsn-workbench-view-actions-crop-live-light-1280x800.png`.

**Evidence:** The actual view designer places its title, checkbox, and three preset buttons in a single `ui.horizontal` row. In the live view, `+ Aggregate Summary` is cut off at the right edge; the following `+ Multi-Table JOIN` action is not visible. No preset was activated and no DDL was generated.

**Impact:** Users cannot read the complete action label and may not discover all SQL presets at the current viewport. Other viewport sizes and keyboard reachability were not tested.

**Recommended fix:** Use an adaptive/wrapped action layout or a compact overflow strategy that keeps every action discoverable and keyboard-accessible. Verify at 1280×800, 1440×900, and 1920×1080.

**Verification:** Reproduced visually on the live BSN session; screenshot crop is 965×80 from the 1280×800 framebuffer. No database operation was issued.

---

### P2 — VERIFIED live safety affordance, conditional risk: synthetic INSERT can target the connected BSN profile without a second confirmation

**Locations:** `crates/app/src/tools/transfer_activity_surface_view.rs:72-97,167-178`; `crates/app/src/tools/transfer_activity_view.rs:82-103`; query dispatch `crates/app/src/query_workbench/events_query_dispatch.rs:248-288`; safety classification `crates/core/src/domain/safety/tests.rs:30-39`.

**Evidence:** The live BSN-connected Transfer surface displayed an active `Apply INSERT (run)` button under `SYNTHETIC TABLE SEED` without a Production warning or confirmation checkbox. Source shows the extra checkbox is required only when the selected profile’s environment equals `Production`; after building the seed SQL, the handler dispatches it through the active query runtime. Query safety classification categorizes `INSERT` as `Write`, not `Destructive`, so the destructive-run confirmation does not provide a second gate. No table was selected, no apply action was activated, and no SQL was sent.

**Impact:** If a real target is connected through a profile not marked `Production`, a user who selects a table and uses this dev/test seed tool can issue synthetic INSERTs without the Production-specific confirmation. The actual BSN environment classification and intended production policy were not independently established; this is a conditional risk, not an observed database mutation.

**Recommended fix:** Make the synthetic-seed tool opt-in for explicitly designated test/demo targets and show the exact target and environment beside the action. Keep it disabled for ordinary saved connections; do not rely on a profile label alone as the only safeguard for a live target.

**Verification:** Live affordance and source dispatch/classifier path verified; no table selection or write path was exercised.

---


### P2 — VERIFIED synthetic layout defect: Status-bar labels collide at 1280×800

**Locations:** `crates/app/src/shell/chrome/shell_statusbar_view.rs:47-64,69-120,123-138`.

**Evidence:** A native presentation-fixture Results screen shows `0 functions` touching `SQL Editor` in the status bar. The privacy-safe crop is `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/statusbar-overflow-crop-synthetic-light-1280x800.png` (1280×28 crop from a 1280×800 fixture). The view puts dynamic connection/database/schema labels, a fixed-width runtime message and right-side context in one horizontal row.

**Impact:** At the tested fixture width, adjacent status fields are visually ambiguous. This is verified only for the synthetic fixture and is not claimed as a BSN live defect.

**Recommended fix:** Budget the row with flexible widths and priority-based truncation/ellipsis for identity labels; keep context and actionable status readable. Recheck live BSN at narrow and wide supported widths before treating provider-specific behavior as established.

**Verification:** Current-source native fixture screenshot; no BSN-specific overlap reproduced.


## Other reviewed areas and non-findings

- The native UI architecture boundary script passed. The clean-code scanner reported 17 error-class and 116 warning-class findings; examples include the 105-line/18-branch `alter_existing_table` planner and a duplicate 14-line worker block. This is maintainability evidence, not 133 independently verified user-visible defects. The CI/ratchet invocation completed; existing-baseline findings remain for review.
- `python3 tools/arch-scan.py crates --fail-on none` reported 98/100, but 602 of 1,108 files (54.3%) were unclassified and the configured `application` layer was unused. Treat that score as incomplete mapping, not proof of full architectural conformance.
- No verified SQL injection path or credential disclosure was established. The live PostgreSQL role list exposes a red Drop action for predefined roles; confirmation remains present and no Drop action was activated. Separately, the live Transfer surface exposed the synthetic-seed INSERT affordance without its Production checkbox because the connected profile was not flagged Production; no seed action was executed, and risk depends on whether the profile environment matches the real target. The app-level read-only limitation remains as described above; the BSN profile is writable.
- The user-reported focus disorder remains **NOT VERIFIED** as an app-wide defect. Three Tab presses produced a visible focus treatment on one shell navigation button; the complete focus order, activation path, document-switch restoration and accessibility tree were not established.
- Prior table-copy rollback fix is present; do not reopen it as a current bug. SQLite rollback regression was recorded passing in existing evidence. PostgreSQL rollback runtime remains pending because the disposable fixture was unavailable/authentication failed in prior verification. The SQL Server rollback integration test checks the reported outcome but does not query persisted data afterward (`crates/infrastructure/tests/sqlserver_integration.rs:118-153`); it is weaker evidence than the PostgreSQL/SQLite assertions.
- Long localized table-footer reflow remains a suspected narrow-width risk in prior audit material; not repeated here as a new finding because no actual overlap was reproduced.

## Verification performed for this audit

- `cargo test --workspace`: **PASS** — 2,133 passed, 0 failed, 56 ignored. Ignored provider tests are not covered by this run.
- `cargo test -p db-pro-core safety`: **PASS** — 56 passed, 451 filtered.
- `bash scripts/check-ui-architecture.sh`: **PASS**.
- `python3 tools/cc-scan.py . --fail-on error`: completed; see scanner qualification above.
- `python3 tools/arch-scan.py crates --fail-on none`: completed; coverage qualification above.
- Continued native BSN UI review on the final-source binary (`45314f7aebddae65e2c01e8a3f227c7e9d79faa41378722a1103798c6241d2e8`), Light 1280×800: inspected live table Data/Structure/Indexes/Foreign Keys, Tools hub (including scrolled content), Security roles, Monitor, Schema Workbench (including Views), empty Execution History, Settings General and the offline Agent panel. Additional surfaces included Schema Compare, Files/Data/Problems empty states, Saved Tasks and unsaved SQL/backup task forms, plus safe `SELECT 1` and its query history. Transfer was inspected without selecting a table or applying seed SQL; its synthetic INSERT action was visible without a Production confirmation on the connected profile. No mutating UI action, settings change, or sensitive row capture was made. New screenshots are hashed in `docs/plans/active/bsn-real-data-native-audit/evidence/runtime/MANIFEST.md`. Dark mode, wide sizes, full keyboard traversal and accessibility tree remain NOT VERIFIED.
- `cargo test -p db-pro-app --lib` — 610 passed while a temporary regression patch was present; the patch and test were removed before handoff, so this is not a verification run of the final source tree.
- `cargo build --release --locked --features capture -p db-pro-native` — PASS on the temporary experiment tree; the binary was subsequently rebuilt from the final source by `cargo run --release --locked --features capture -p db-pro-native`. The final-source run opened the app and completed safe `SELECT 1`; inherited unused-import and capture-only dead-code warnings remain.

## Recommended order of work

1. Reset table-scoped filters/sorts/errors when switching tables; add a BSN-independent regression for tables with different key columns. The live BSN switch currently fails.
2. Fix targeted-retry staged-edit loss with a multi-row failure → retry → success regression.
3. Reproduce the ER search mismatch on a deterministic large-schema GUI fixture; ensure matching nodes become visible and auto-fit after the debounce without laying out all tables.
4. Propagate table DDL validation errors; ensure a partially valid table edit cannot become a partial DDL plan.
5. Fix the provider test matrix and make configured SQL Server connection failures fail tests.
6. Close the MySQL timeout gap in parameterized transactions; verify on a disposable MySQL fixture.
7. Decide and document whether app read-only is UX-only or a strict safety guarantee; enforce strict mode at the database/session boundary if required. Add deterministic connection lifecycle race tests.
8. Profile query-result cloning, large-cell rendering, scheduled idle repaint, and large-schema ER search before optimizing; correct the stale perf runner and extend the grid benchmark.
9. Improve live Structure type readability; complete native visual evidence for focus, localization and supported viewports, and verify PostgreSQL rollback on a disposable fixture.
10. Clear or scope activity-specific status feedback so an earlier Security success cannot persist on another active surface; add flexible width budgeting to prevent status text from colliding with editor context.
11. Identify provider-defined/system roles and disable the destructive Drop affordance for roles that cannot be dropped; preserve the explicit confirmation for droppable roles.
12. Reflow/collapse the Schema Workbench view preset actions and validate discovery at supported widths; ensure synthetic seed INSERT cannot run on ordinary saved/live profiles without explicit test-target opt-in and visible target confirmation.
