# Result-grid projection cache — the per-frame rebuild is removed, measured (#238)

- Session: issue-queue pass 5, 2026-09-15
- Issue: **#238** ([P2][RC1][Perf] Result-grid sort rebuilds an allocating projection on every frame),
  filed from the #76 audit (`docs/release/rc1-p2-release-dispositions.md` §B), carrying `QA-P2-12`
- **Base head:** `main @ 9a8dc5b` (worktree clean; PR #245 already merged as `00a3ce9`)
- **Outcome:** the issue's acceptance items **are met on `main` after this change** — the projection
  is no longer rebuilt per frame, the sorted path is benched, and a test counts the rebuilds. The
  per-frame cost of a sorted 200k-row result on a timestamp-shaped column drops from **10.65 s to
  ~40 ms** in debug. One residual per-frame rebuild of a *different* structure was found and is filed
  separately (§7); it is not part of this issue's acceptance items.

## 1. Why the merged PR was not the end of the issue

PR #245 (`00a3ce9`, "optimize result grid sorting and filtering allocations (#238)") implemented
**option (b)** of the issue — an allocation-free comparator (`cell_text_as_str`), an ISO-shape gate in
front of the temporal parsers (`looks_like_iso_temporal`) and an ASCII fast path in the filter — and it
auto-closed #238 on merge. It did **not** implement option (a) (no cache) and its diff touched only
`crates/ui/src/result_grid.rs` and `crates/ui/src/lib.rs`, so the issue's acceptance items 1 and 3 were
still open:

| #238 acceptance item | State after the #245 merge (checked on the merged tree) |
|---|---|
| "a sorted large result no longer rebuilds the projection every frame (cache or equivalent)" | **NOT met** — `draw_result_grid` still called `filtered_sorted_indexes` from the draw path (`crates/ui/src/result_grid_view.rs:76`), and the comparator still parses temporal text on every comparison |
| "the existing behaviour is unchanged: `result_grid_benchmarks`, `sorting_is_blocked_while_staged_changes_are_present` and the grid layout/selection tests stay green" | met |
| "a criterion measurement for the **sorted** path is added next to the existing `project_without_filter_or_sort`" | **NOT met** — the merged diff contains no benchmark change |

So this pass did not "re-verify and close": it implemented the missing item, benched it and pinned it.

## 2. Re-measurement on `main @ 9a8dc5b` (this host, debug profile, 200k rows × 4 columns)

Temporary probe compiled into `crates/ui/src/result_grid.rs`, run with `--nocapture`, **deleted before
the commit** (the tree carries no probe). Fixture: `id` integer-as-text, `customer-%06d` text,
`2026-01-%02d 12:%02d:%02d` timestamp-shaped text, `active`/`idle` text.

| Case | Before the #245 merge (pass 4, `providers/49`) | **On `main @ 9a8dc5b` (after #245)** |
|---|---|---|
| projection, no filter, no sort | 5.29 ms | **5.50 ms** |
| projection, sorted asc, plain text column | 166.6 ms | **11.60 ms** (14× better — the #245 gate works) |
| projection, sorted asc, timestamp-shaped text column | 6,649.6 ms | **10,528.0 ms** (unchanged in kind, and *worse* on this run) |
| filter (100 matches) + sort | 72.9 ms | **98.97 ms** |

Read plainly: the merged PR fixed the case it could fix cheaply (non-temporal text) and left the
seconds-long case exactly as it was, because that case's cost is four date/time parse attempts per
comparison, which an allocation-free comparator does not remove. Every one of these numbers was paid
**per frame** by the draw path.

## 3. What was implemented

One-entry memo for the projection, keyed on the inputs that decide it
(`crates/ui/src/result_grid.rs`, `GridProjectionKey` / `GridProjectionCache`):

```rust
pub struct GridProjectionKey { epoch: u64, filter: String, sort_column: Option<usize>,
                               sort_desc: bool, row_count: u64, column_count: usize }
```

- `epoch` is a monotonic id for the row data behind the grid (`DbProApp::invalidate_grid_projection`,
  `crates/ui/src/app.rs`). The draw path receives `&UiQueryResult`, and on the query path that value is
  a **per-frame clone** (`query_view.rs:35`), so pointer identity is not usable as a key; the epoch is
  advanced instead at every site where the displayed rows change.
- The draw path takes the projection out of the cache, uses it as an owned local for the frame, and
  restores it at the end (`result_grid_view.rs:75-99`), which keeps the rows from being copied per
  frame and avoids holding a borrow of `self` across the frame.
- Invalidation sites, all of them, with the row-identity cache where it already existed so that the
  set stays checkable by grep: `on_table_info_loaded`, `on_table_data_loaded`, `on_table_row_reloaded`
  (a single row replaced in place), `on_query_completed`, `on_query_multi_completed`,
  `open_agent_result_in_workspace`, `set_active_query_result`.
- One redundant rebuild was removed rather than cached: the header's auto-size action called
  `filtered_sorted_indexes` a second time in the same frame for one column-width change
  (`result_grid_view.rs:2328`), and now reuses the projection the frame already holds.

No behaviour change: same filter, same comparator, same order, same stable sort over original row
indexes. The cache is only allowed to answer with a projection built from the same epoch, filter,
sort column, direction, row count and column count.

## 4. Measurement, after (this host, debug profile)

Per-frame cost measured **through the real draw path** (`draw_result_grid` in a real egui pass, 200k
rows × 4 columns), with the cache switched off and on **in the same build** so the two columns are
directly comparable:

| Frame (200k rows, sorted on the timestamp-shaped column) | Cache **off** (pre-change behaviour) | Cache **on** |
|---|---:|---:|
| frame 0 (first, always a rebuild) | 10,652.3 ms | 10,731.1 ms |
| frame 1 | 10,651.0 ms | **41.1 ms** |
| frame 2 | 10,622.8 ms | **40.0 ms** |
| frame 3 | — | **42.7 ms** |
| frame 4 | — | **39.8 ms** |
| rebuilds after 5 frames | 2 → 4 (every frame) | 2 → 2 (first frame only) |

| Frame (200k rows, sorted on the plain text column) | Cache **off** | Cache **on** |
|---|---:|---:|
| frame 0 | 49.6 ms | 51.5 ms |
| frame 1 | 48.9 ms | **38.5 ms** |
| frame 2 | 48.4 ms | **37.8 ms** |

**10.65 s → ~40 ms per frame** on the worst case the issue named, with the sort order provably intact.
The steady-state ~40 ms is no longer the projection (§7 explains it).

## 5. Criterion benchmark (acceptance item 3)

Added `result_grid_projection_sorted` to `crates/ui/benches/result_grid_benchmarks.rs` next to
`result_grid_million_rows/project_without_filter_or_sort`: `sort_plain_text_column`,
`sort_temporal_text_column`, `filter_and_sort`, 200k rows × 4 columns, `sample_size(10)`.

`cargo bench -p db-pro-ui --bench result_grid_benchmarks -- result_grid_projection_sorted` (release):

| Benchmark id | Time | Throughput |
|---|---|---|
| `result_grid_projection_sorted/sort_plain_text_column` | **2.53–2.59 ms** | 77–79 Melem/s |
| `result_grid_projection_sorted/sort_temporal_text_column` | **566.8–576.1 ms** | 347–353 Kelem/s |
| `result_grid_projection_sorted/filter_and_sort` | **4.44–4.58 ms** | 43.7–45.1 Melem/s |

Budgets recorded in `.skills/perf-audit/references/perf-budgets.md`; the temporal row deliberately
carries **no target** rather than a number the code does not meet (see the note in that file).

## 6. Tests (acceptance item 1)

| Test | Pins | Falsified by |
|---|---|---|
| `projection_cache_reuses_the_projection_across_frames` | three frames, one rebuild | probe 1 |
| `projection_cache_rebuilds_when_an_input_changes` | filter / sort column / direction / epoch each rebuild | probe 2 |
| `projection_cache_rebuilds_when_the_row_shape_changes` | a shape change rebuilds even at the same epoch | probe 2 |
| `projection_cache_keeps_a_newer_entry_over_the_frame_copy` | a frame's stale copy cannot overwrite a newer entry | probe 1 |
| `result_grid_does_not_rebuild_the_projection_every_frame` | **through the real draw path**: 3 frames → 1 rebuild | probe 1 |
| `result_grid_rebuilds_the_projection_when_an_input_changes` | same path: each input change rebuilds, an unchanged frame does not | probe 2 |

Probe 1 made `GridProjectionCache::take` always miss (`if false && …`): **4 failed** — the two
"reuse" tests and both draw-path tests. Probe 2 dropped `epoch` from the key comparison (hand-written
`PartialEq`): **2 failed** — exactly the two tests that assert an epoch change rebuilds. Both probes
were reverted; the tree carries neither.

## 7. Residual found while measuring (filed, not fixed here)

The steady-state ~40 ms/frame is a **second per-frame rebuild of a different structure**:
`GridSelectionLookup::new(&indexes, &order)` (`crates/ui/src/result_grid_view.rs:15`) builds a
`HashMap<usize, usize>` over **every filtered row index** on every frame from `draw_grid_body`.

Measured on this host, debug, 200k indexes:

```
PROBE GridSelectionLookup::new(200k indexes): 36.535 ms (len 200000)
PROBE column_widths(4): 0.002 ms (4)
```

i.e. 36.5 ms of a ~40 ms frame. It is the same defect shape as #238 (a derived structure over the
whole result, rebuilt per frame), but it is **not** one of #238's acceptance items and fixing it needs
its own key (projection + column order) and its own tests, so it was filed as a separate issue rather
than absorbed here. Release-profile impact is smaller but not zero; no release number is quoted
because none was measured.

## 8. Gates (raw totals, this host)

| Gate | Command | Result | Baseline | Delta |
|---|---|---|---|---|
| Format | `cargo fmt --all -- --check` | exit **0** | — | — |
| Check | `cargo check --workspace` | exit **0** | — | — |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no warnings | — | — |
| Tests | `cargo test --workspace` | **895 passed / 0 failed / 27 ignored** | 889 / 0 / 27 | **+6** |
| CI-mirror (fixture up) | `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture cargo test --all -- --include-ignored` | **922 passed / 0 failed / 0 ignored** | 916 / 0 / 0 | **+6** |
| Release build | `cargo build --release --locked -p db-pro-native` | exit **0** | — | — |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | **PASS (partial)**, exit **0** — 4 executed passed, 0 warnings, 0 failed, 4 not executed; binary 22.7 MB, sha256 `e273f2820ead25e2…` | same shape | — |

The +6 is exactly the six new tests (four in `result_grid.rs`, two in `result_grid_view.rs`). The
PostgreSQL fixture `dbpro-v01-pg-fixture` was started for the CI-mirroring run and **stopped
afterwards**; credentials are redacted here. No `qltx-*` container was touched.

## 9. Files changed

| File | Change |
|---|---|
| `crates/ui/src/result_grid.rs` | `GridProjectionKey`, `GridProjectionCache`, 4 tests |
| `crates/ui/src/result_grid_view.rs` | draw path takes/restores the projection, header reuses it, 2 tests |
| `crates/ui/src/app.rs` | epoch field + `invalidate_grid_projection` / `invalidate_grid_row_caches`, key builder |
| `crates/ui/src/app_state.rs` | field defaults |
| `crates/ui/src/events.rs` | epoch advance on query-result, table-data and row-reload changes |
| `crates/ui/src/agent_state.rs` | epoch advance when the agent opens a result in the workspace |
| `crates/ui/src/lib.rs` | re-exports |
| `crates/ui/benches/result_grid_benchmarks.rs` | `result_grid_projection_sorted` group |
| `.skills/perf-audit/references/perf-budgets.md` | three budget rows with the reasoning above |
