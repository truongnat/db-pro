# Findings — Query Editor at large schema sizes

Exact source SHA for every anchor below: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`. Audited implementation files are unchanged from that HEAD, verified with git diff --quiet. Source hashes and optimized library fingerprint: evidence/manifest.json. Other checkout modifications are not attributed to this audit.

## [P1] Completion panics on UTF-8 identifiers

`crates/ui/src/query/schema_completion.rs:1094`: extract_word_prefix slices text[end-1..end] and decrements byte offsets. `SELECT tên` panics inside ê. The executable probe calls actual SchemaCompletionProvider::provide and catches the panic; two runs reproduced it. `query_editor_surface_view.rs:302` calls the provider synchronously during completion; typing/IME while popup is open refreshes it, and manual completion is reachable too. Not tied to PostgreSQL or 500 tables. This is a proven crash-capable code path, NOT confirmation of the owner's actual startup crash.

Minimal fix: walk char_indices backwards and slice only character boundaries; check qualifier and cursor/replacement byte-offset boundaries. Add regressions for Vietnamese, accented identifiers, multibyte separators and IME edits. No such fix is included here.

## [P1] Completion pipeline expands/renders an unbounded suggestion list

`schema_completion.rs:555`: fallback pushes every matching column across all table_details (the comment says active schema, but code does not filter it). No maximum item count. `schema_completion.rs:1076`: full sorting repeatedly lowercases labels inside comparator. `query_completion_popup_view.rs:98-103`: ScrollArea max_height only constrains viewport; it still runs the row closure for every item, with badge/label/detail widgets. No show_rows or equivalent virtualization.

Actual optimized provider: 500 × 40 = 20,000 column suggestions, median 26.365 ms, nearest-rank p95 27.731 ms across 15 calls. At 1000 tables: 40,000 items, median 54.445 ms, p95 57.430 ms. These generation times alone exceed a 16.7 ms frame budget, before popup layout. Popup frame-time itself was not measured: source confirms O(total suggestions) widget work, not a measured FPS claim.

Counter-evidence: qualified `t.col` with one referenced table returns 40 items at 0.028 ms median for 500 tables. Raw lookup of a scoped column is not slow in this fixture. Reported one-table lag may instead involve unscoped fallback, popup paint, document size or output-pane work; an actual trace is required to isolate it.

Minimal fix: semantic table/schema scope first; deduplicate by qualified identity, precompute normalized labels, bounded top-k ranking and virtualized popup. Preserve hidden-column access/search and keyboard selection; do not merely truncate correctness away.

## [P2] Whole query result cloned on every output-pane frame

`crates/ui/src/query_view.rs:269`: active_result().cloned() precedes draw_output_pane each frame. This deep-copies strings/cells, independent of which Query interaction caused repaint. Real UiQueryResult::clone fixture: 500 rows × 20 columns × 8KiB text = 78.125 MiB payload; upper median of 10 clones 8.445 ms, max 17.963 ms. This is clone time only; allocation/destruction/render total is higher. Probe peak RSS 266,010,624 bytes covers the combined harness, not the app or this clone alone. No OOM reproduced.

Minimal fix: borrow/split disjoint state or share immutable result storage; clone only small view state. Verify Save/refresh/selection and old-result lifetime before changing ownership.

## [P2] Schema index built synchronously on UI load transition

`crates/ui/src/schema_events.rs:57` calls SchemaSymbolIndex::build before state replacement. `query/intelligence.rs:120-135` stores column help under bare, table-qualified and schema-qualified keys, cloning strings into separate records. Single-build fixture timings: 24.771 ms for 500 × 40; 53.117 ms for 1000 × 40. First probe had 27.582 / 89.260 ms, indicating cold/scheduling variation; do not read single values as stable percentile budgets. This can visibly pause load; no load crash reproduced.

Minimal fix: build once per schema generation off the UI thread, publish atomically under connection/request generation checks; share symbol records rather than duplicating strings. Avoid stale indices from a different connection.

## [P2] Runtime schema summary performs repeated global metadata scans

`crates/runtime/src/api.rs:470-484`: for each table, filter all columns and all foreign keys. Complexity O(T×C + T×FK), instead of one partition pass. 500 tables × 20,000 columns implies 10 million column predicate checks (algorithmic count, not measured latency). Runs on worker side; therefore it cannot alone establish an egui-thread crash.

PostgreSQL provider `crates/infrastructure/src/postgres/introspect.rs:54` already runs ten bulk metadata queries with tokio::join!. No evidence that it sends one metadata SQL query per table. Live PostgreSQL latency, permissions, schema-change races, provider error output and cache behavior remain unmeasured.

Minimal fix: group metadata by (schema, table) once, preserve ordering and qualified identity. Benchmark summary assembly separately from database query time.

## Additional evidence and limits

- 66,027-byte SQL buffer: qualified completion still yields 40 items but takes 9.404 ms in one run. `schema_completion.rs:80-85` scans/parses full document before qualified dispatch; cache semantic context by buffer version and avoid unrelated statements. This is a contributor, not another asserted P1.
- Main editor renderer already virtualizes visible lines and uses CachedSqlTokens; `editor/renderer.rs:784` is not a full-line rendering loop. Do not replace that correctly scoped path on speculation.
- PostgreSQL is owner's affected provider; engine/UTF-8/result-copy findings are shared with SQLite, but live SQLite/PostgreSQL tests were not run.
- No matching DB Pro crash report was found in this host's macOS DiagnosticReports. Exit-vs-hang and exact startup failing frame remain unresolved; no credential or company data was read.
- Host free disk around 144 MiB during probe. Existing optimized rlib was reused to avoid a large build; benchmarks are local CPU microprobes, not clean-room frame profiling.

## Remediation order and learning pass

1. Eliminate proven UTF-8 panic with a regression capable of going red.
2. Bound producer and virtualize popup together; keep scoped completion correct.
3. Remove result-copy work from paint path.
4. Move shared schema-index construction off UI and partition metadata once.
5. Collect actual company crash/backtrace and native frame/memory traces before claiming the load crash fixed.

Reusable lessons: max_height is not virtualization; schema-wide completion budgets should cover wide column counts, not just table names; rendering must not deep-copy payloads; scoped lookup can be cheap while the surrounding frame remains expensive. Recorded here; no global memory write authorized.
