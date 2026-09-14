# RC1 P2-B — result-grid sort: claim verification and live re-measurement (#238)

- Session: issue-queue pass 4, 2026-09-15
- Issue: **#238** ([P2][RC1][Perf] Result-grid sort rebuilds an allocating projection on every frame)
  — filed by the #76 audit (`docs/release/rc1-p2-release-dispositions.md` §B)
- **Base head:** `main @ ba2f515` (worktree clean at session start)
- **Outcome:** defect **confirmed live and unfixed on `main`**; fix already claimed by
  **open PR #245** on an unmerged branch; issue **left open** with a precise comment rather than
  duplicated. No product code changed by this entry.

## 1. Claim check — is #238 already fixed or claimed?

The workstream rule for a `Fix RC1` child is to check for an existing claim before writing code.

```
$ git merge-base --is-ancestor a0452a32d38d2350dab0d3d28c5edac8153920bf main
$ echo $?
1
$ git log main --oneline --grep=238
$ echo $?
0
```

- `git log --all --oneline --grep=238` finds exactly one commit: `a0452a32 fix(ui): optimize result
  grid sorting and filtering allocations (#238)`, the head of `fix/rc1-p2-result-grid-sort-perf-7341115780816662696`.
- That commit is **not an ancestor of `main`** (exit 1 above), and `main`'s own history contains no
  #238 commit. The last change to the file on `main` is `ebf085b` (#80).
- `gh pr list --state all --search 238` → **PR #245, `OPEN`**, `mergeable: MERGEABLE`, 146 additions /
  27 deletions across `crates/ui/src/lib.rs`, `crates/ui/src/result_grid.rs` and four
  `docs/plans/active/rc1-p2-result-grid-sort-perf/` files.

Conclusion: **claimed, not landed.** Per the main-only workflow (no branches, no PRs, no checkouts,
no merging another session's work) this issue is not implemented here; it is verified and handed back.

## 2. The unfixed path on `main @ ba2f515`

| Site | State |
|---|---|
| `crates/ui/src/result_grid_view.rs:75-76` | `draw_result_grid` calls `crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc)` on every frame, uncached |
| `crates/ui/src/result_grid.rs:97-124` | `filtered_sorted_indexes` allocates the index `Vec`, lower-cases the filter, and allocates a `String` per cell comparison through `cell_text` (`:92`, `:146-152`) |
| `crates/ui/src/result_grid.rs:67-90` | the `Text`/`Text` arm runs `DateTime::parse_from_rfc3339`, then `NaiveDateTime::parse_from_str("%Y-%m-%d %H:%M:%S")`, then `NaiveDate::parse_from_str("%Y-%m-%d")` — ×2 operands, per comparison |

## 3. Measurement — temporary probe, re-derived and removed

Method: a temporary integration test (`crates/ui/tests/zz_temp_probe_238.rs`, `db-pro-ui` crate,
`#[test]`, run with `--nocapture`) built 200,000 rows × 4 columns and timed one
`filtered_sorted_indexes` call per case after one warm-up call. Same shape and profile (debug,
unoptimized) as the #76 audit probe, which was removed the same way. Raw output:

```
$ cargo test -p db-pro-ui --test zz_temp_probe_238 -- --nocapture
running 1 test
warmup (unsorted): 5.26 ms (200000 projected rows)
projection, no filter, no sort: 5.29 ms (200000 projected rows)
projection, sorted desc on text column: 167.33 ms (200000 projected rows)
projection, sorted asc on text column: 166.63 ms (200000 projected rows)
projection, sorted asc on timestamp text column: 6649.56 ms (200000 projected rows)
filter + sort: 72.91 ms (100 projected rows)
SUMMARY unsorted=5.29ms sorted_asc=166.63ms sorted_desc=167.33ms sorted_ts=6649.56ms filtered=72.91ms
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.32s
```

Fixture: `id: Number(index)`, `name: Text("customer-{index:06}")`, `status: Text("active"/"idle")`,
`created_at: Text("2026-01-{d:02} 12:00:00")`.

| Case | This run (`main @ ba2f515`) | #76 audit (`docs/release/rc1-p2-release-dispositions.md` §B) |
|---|---|---|
| projection, no filter, no sort | **5.29 ms** | 5.35 ms |
| projection, sorted asc, plain text column | **166.6 ms** | — |
| projection, sorted asc, timestamp-shaped text column | **6,649.6 ms** | **3,414 ms** ("sorted on a text column") |
| filter (100 matches) + sort | **72.9 ms** | 83.65 ms (200 matches) |

**Correction carried into the issue:** the audit's single "sorted on a text column" figure is not
uniform over text columns, and the worst case is worse than the number on file. A plain text column
fails all three temporal parses on the first bytes and costs 167 ms; a timestamp-shaped text column
(`2026-01-17 12:00:00`) makes `NaiveDateTime::parse_from_str` succeed on both operands of every one of
the ~3.5M comparisons and costs **6.6 s** — reached by any result grid sorted on a timestamp column
whose provider type arrives as text. The issue's own acknowledgement of "3,414 ms sorted" is therefore
a lower bound for the worst path, not the worst path itself.

Probe removed before this commit: `rm crates/ui/tests/zz_temp_probe_238.rs` (the `tests/` directory
did not exist before and is gone again). `git status --short` is clean at the recorded head.

## 4. What the existing claim does and does not cover (for the owner)

Read from `gh pr diff 245`:

- **Covered — option (b) of the issue:** the comparator becomes allocation-free
  (`cell_text_as_str(&UiCell) -> &str`, `cell_text` re-expressed over it) and the three temporal parses
  are gated behind a `looks_like_iso_temporal` byte check; the filter path drops per-cell
  `to_lowercase()` allocation behind an ASCII fast path with a Unicode fallback.
- **Not covered — option (a), the per-frame rebuild:** `result_grid_view.rs` is absent from the diff
  file list, so the projection is still recomputed from `draw_result_grid` on every frame. The change
  lowers the per-frame constant; it does not make the projection once-per-data-or-sort-change.
- **Not covered — acceptance items 1 and 3:** no cache keyed on (result identity, filter, sort column,
  direction) and no call-count/work-count test for the "no per-frame rebuild" property (item 1); no
  benchmark file in the diff, so no criterion case next to `project_without_filter_or_sort` for the
  sorted path (item 3).

Recorded so the owner can decide whether to merge #245 as a partial mitigation and keep #238 open, or
have the branch extended with the cache, the call-count test and the sorted-path criterion bench
(which, per §3, should sort a **timestamp-shaped** text column to cover the seconds-long path).

## 5. Disposition

- Issue **#238 stays open**; status comment posted with the measurement table, the claim check and the
  PR-review findings above.
- **No product code, test or document changed** by this entry except this evidence file.
- Ledger row updated in the same commit; issue count unchanged (the issue was and remains open).
