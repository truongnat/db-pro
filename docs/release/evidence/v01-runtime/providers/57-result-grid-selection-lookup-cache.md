# Result-grid selection lookup — the per-frame rebuild is removed, measured (#246)

- Session: issue-queue pass 6, 2026-09-15
- Issue: **#246** ([P2][RC1][Perf] Result-grid frame rebuilds a HashMap over every filtered row
  index), filed by pass 5 from the #238 measurement (`providers/53`), where it was the residual the
  projection cache exposed
- Fix commit: **`71ca86c1`** (`fix(grid): memoize selection lookup per-frame (close #246)`), merged
  into `main` as `84e3746a`
- **Re-measurement base head:** `main @ d4df6373` (worktree clean)
- **Outcome:** the fix is present on `main` and the defect is gone. The lookup is built **once per
  (projection, column order)** instead of once per frame: steady-state frame cost falls from the
  issue's **36.5 ms** to **0.0002–0.0004 ms/frame** measured in the same debug profile on the same
  fixture. The issue's acceptance is met; nothing was left open.

## 1. What the issue measured, and what the fix changes

Pass 5 measured, with a temporary probe that was removed before its commit
(`providers/53-result-grid-projection-cache.md` §7):

```text
PROBE frame 1 (temporal sort): 41.127 ms  rebuilds=2
PROBE frame 2 (temporal sort): 40.030 ms  rebuilds=2
PROBE frame 3 (temporal sort): 42.748 ms  rebuilds=2
PROBE GridSelectionLookup::new(200k indexes): 36.535 ms (len 200000)
```

That is **36.5 ms of a ~40 ms steady-state frame** paid by one call: `draw_grid_body` built
`GridSelectionLookup` — two `HashMap`s, one entry per filtered row index plus one per column — on
every frame, although it only changes when the projection or the visual column order changes. With
the projection already cached (#238), this map build was the frame.

`71ca86c1` adds `GridSelectionCache`, the same one-entry take/restore idiom as `GridProjectionCache`,
keyed on `(GridProjectionKey, column_order)`: the draw path takes the lookup before the frame and
hands it back after, and `draw_grid_body` / `handle_grid_keyboard` receive it borrowed instead of
rebuilding it (`crates/ui/src/result_grid_view.rs`).

## 2. Re-measurement on `main @ d4df6373` (this host, debug profile, 200k rows × 4 columns)

Temporary probe compiled into the crate's own test module (`crates/ui/src/result_grid_view.rs`),
appended, run with `--nocapture`, **deleted before the pass-6 commits** — the tree carries no probe.
The probe drives exactly the draw path's protocol (`take` → build only on a miss → `restore`) and
reads the cache's own `rebuilds()` counter. Three runs, same build:

| Measurement | Run 1 | Run 2 | Run 3 | Issue's figure (pass 5) |
|---|---:|---:|---:|---:|
| `GridSelectionLookup::new(200k indexes, 4 cols)` — the rebuild the frame used to pay | 40.887 ms | 39.225 ms | 62.811 ms | **36.535 ms** |
| Frame 1 (miss → rebuild) | 36.469 ms | 39.361 ms | 43.316 ms | ~40 ms/frame |
| Steady state, frames 2..=301 (`take` hit + `restore`) | 0.115 ms total → **0.000383 ms/frame** | 0.054 ms → **0.000181 ms/frame** | 0.053 ms → **0.000176 ms/frame** | 36.5 ms/frame |
| `rebuilds()` after 301 frames | **1** | **1** | **1** | 1 per frame |

Read plainly: the first frame still pays the map build once (~36–43 ms, unchanged — the probe
reproduces the issue's number on this host), and every frame after it pays a key comparison and moves
the `Option` out and back, **three orders of magnitude below one millisecond**. The rebuild count
frozen at 1 over 301 frames is the property that makes it true: the cache is not rebuilt per frame,
only when the projection or the column order changes.

## 3. Committed evidence (no probe needed)

| Test (`cargo test -p db-pro-ui --lib <filter>`) | What it pins |
|---|---|
| `selection_cache_reuses_the_lookup_across_frames` | one rebuild; the next frame gets the same lookup back, with the same row/column positions |
| `selection_cache_rebuilds_when_the_projection_or_the_column_order_changes` | a column reorder, a sort and a new result epoch each rebuild exactly once; an unchanged frame does not |
| `selection_cache_keeps_a_newer_entry_over_the_frame_copy` | the take/restore protocol cannot overwrite a newer entry |
| `result_grid_rebuilds_the_selection_lookup_when_the_column_order_changes` | the same property through the real draw path, not the cache unit |
| `selection_lookup_is_reachable_from_the_crate_root` | added in pass 6 (`6afc5ba6`): the criterion bench is a separate crate and reaches the type only through the crate-root re-export, so this test says so in the library's own suite |

The criterion group `result_grid_selection_lookup`
(`crates/ui/benches/result_grid_benchmarks.rs`, case `build_200k_rows_4_columns`) measures the cost
of one rebuild on the same 200k × 4 fixture, so the avoided per-frame cost stays under a benchmark
that runs against the public API rather than a copy of the body. It is not run by the v0.1 quality
gate (`perf-scan` reports the native benchmark sections as "not executed" unless asked), so the
numbers above are the probe's.

## 4. What this does not claim

- **No release-profile number.** Both the issue's figure and this re-measurement are debug
  (`cargo test`, unoptimized) — comparing them like-for-like is the point. The draw path is not
  measured end-to-end here; the probe isolates the lookup, as the issue did.
- **No claim about the first frame.** A cache miss still pays the full map build (36–43 ms at 200k
  rows, debug). What the fix removes is paying it every frame.
- **No claim about `GridProjectionCache`.** The projection side (#238) is unchanged and is recorded in
  `providers/53-result-grid-projection-cache.md`.
