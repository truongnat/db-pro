# RC1 P2-E — performance tooling and release-measurement semantics (#79)

- Session: issue-queue pass 3, 2026-09-15
- Issue: **#79** ([RC1][P2-E] Audit performance tooling, bundle evidence, and release-measurement
  semantics) — parent **#27**
- **Base head:** `main @ d9c9eae` (worktree clean at session start)
- **Dispositions:** `docs/release/rc1-p2-release-dispositions.md` §E
- **Child issue:** **#240** (the three `Fix RC1` rows of §E)

## 1. What was audited

The eight scope bullets of the issue. Unlike #75–#78, this audit has **no `QA-P2-*` row to settle**:
`docs/plans/active/rc1-full-product-qa/FINDINGS.md` carries 25 rows and none of them is a perf-tooling,
bundle or measurement-semantics finding. The audited objects are therefore the tooling artifacts
themselves — `.skills/perf-audit/` (script, skill, budgets reference), the React-era perf reports and
PR #13 — and §E records one disposition per bullet.

The tool is not decorative: the perf scan is a **recorded release gate**.
`docs/release/evidence-manifest.json` has a `perf.scan` row (`status: pass`,
`observed_at_sha 85a7fa3`), and the readiness table, the release checklist, the final report and the
handoff all quote `PASS — 4 passed / 0 warnings / 0 failed`. Its wording is release evidence.

## 2. Measured before/after

### 2.1 `all` — was `Status: PASS — All checks passed` while four sections were not run

`bash perf-scan.sh` (old script, from `git show HEAD:.skills/perf-audit/scripts/perf-scan.sh`), raw
output with ANSI stripped:

```
═══ Native UI Binary ═══
  Building db-pro-native (release)...
  ✓ cargo build -p db-pro-native: release build succeeded
  ✓ Binary size: 22.7MB (target < 50MB)
═══ Rust Static Analysis ═══
  ✓ cargo check: Workspace compiles
  ✓ cargo clippy: No warnings
  Note: 'all' skips benchmarks and ER/DB runtime checks.
  Run with 'native' or 'rust' for benchmarks, or verify ER/DB manually.
═══ Audit Summary ═══
  Passed: 4
  Warnings: 0
  Failed: 0
  Status: PASS — All checks passed
```

Exit status 0. Nothing in that output names *what* was skipped in a way that survives being quoted —
and the release documents quote exactly the `Status: PASS` line.

Same command after the fix (working tree of this change, i.e. `d9c9eae+dirty(3)`):

```
  Source revision: d9c9eae…+dirty(3)
  ✓ Binary size: 22.7MB (target < 50MB) sha256 5686bc1949054a90…
  ✓ cargo check: Workspace compiles
  ✓ cargo clippy: No warnings
  • Native UI benchmarks: not executed — run section 'native'
  • Rust backend benchmarks: not executed — run section 'rust'
  • ER diagram runtime: not executed — needs a window server; run section 'er'
  • DB query performance: not executed — needs a live connection; run section 'db'
  Source revision: d9c9eae…+dirty(3)
  Measured artifact: target/release/db-pro-native sha256 5686bc1949054a90b43f7f3c5208bedfd344d7ae321d5dfdff132d1a4e64fc83 (22.7MB)
  Passed: 4   Warnings: 0   Failed: 0   Not executed: 4
  Status: PASS (partial) — 4 executed check(s) passed; not run: Native UI benchmarks, Rust backend
  benchmarks, ER diagram runtime, DB query performance [source d9c9eae…+dirty(3): working tree not committed]
```

### 2.2 Exit codes — a warning exited 0

Measured on this host, `er` section (the cheapest reproducible warning path):

| Command | Old script | New script |
|---|---|---|
| `bash perf-scan.sh er` | `Status: WARN — 1 warning(s)` · **exit 0** | `Status: WARN — 1 warning(s)` · **exit 2** |

An automated gate that reads only the exit status could certify a warned run as green; it now cannot.
The contract is `PASS` 0 / `WARN` 2 / `FAIL` 1, documented in the script header and in SKILL.md §8.

### 2.3 Benchmark inventory (grounds the documentation corrections)

`grep -n 'bench_function\|benchmark_group'` over the two bench files:

| File | Registered criterion ids |
|---|---|
| `crates/ui/benches/result_grid_benchmarks.rs` | `result_grid_million_rows/project_without_filter_or_sort`, `result_grid_scroll_window/materialize_100_visible_rows`, `result_grid_requested_sizes/build_visual_maps_{1_000,10_000}_rows_50_columns` |
| `crates/infrastructure/benches/sqlite_benchmarks.rs` | `sqlite_connect_and_disconnect`, `introspect_small_db`, `introspect_large_schema`, `query_rows/select_10k`, `query_rows/select_100k`, `query_json_blob/select_json_metadata_5k`, `serialize_large_text/select_1k_large_text`, `explain_query` |

Before the fix, SKILL.md §2 budgeted five UI operations that appear in **neither** file (grid
visible-range computation, grid hit-testing, cell codec round-trip, Quick Open index, statement split),
§4 named three backend ids only partially and listed "Execution registry" as if it were a benchmark,
and `references/perf-budgets.md` repeated both plus a `strip` row for a packaging step that no workflow
performs (`grep -rn strip .github/workflows/ docs/release/0.1.0-packaging.md` → no matches).

### 2.4 CI wiring (checked, not assumed)

`grep -rn perf .github/workflows/` → **no matches**. No perf check, benchmark or `perf-scan.sh`
invocation is wired into `ci.yml` or `release.yml`; the perf scan is a local gate.
`docs/release/0.1.0-handoff.md` already records "(local gate; not run in CI)" and
`docs/architecture/performance-baseline.md` states "Budget targets are NOT CI gates". SKILL.md §7
previously listed a `Recommended CI checks` YAML block that could be misread as the CI reality; it now
states the position explicitly and labels the block a candidate, with the reason it is not wired.

## 3. Defects found and fixed (child issue #240)

| # | Defect | Fix |
|---|---|---|
| 1 | `all` printed an unconditional `Status: PASS` while deliberately not executing the two benchmark sections and the ER/DB runtime sections | skipped sections are counted (`skip()`), named in the status line, and the status reads `PASS (partial)` |
| 2 | a `WARN` result exited 0, so a status-only gate could not distinguish it from a pass | exit contract `PASS` 0 / `WARN` 2 / `FAIL` 1 |
| 3 | no run named the source revision or the artifact it measured, so a recorded size could not be tied to a commit or a binary | header and summary print `Source revision: <sha>[+dirty(N)]` and `Measured artifact: … sha256 <digest> (<size>MB)`; a dirty tree qualifies the status line |
| 4 | SKILL.md §2/§4 and `references/perf-budgets.md` named benchmarks that do not exist and a packaging step that does not exist | the exact registered criterion ids are listed; rows with no benchmark behind them are marked as such; the `strip` row is marked "not a v0.1 step" |

Unchanged on purpose: the build-failure path already refuses to report a size (it returns before the
size check), so a stale binary was never measured — this is recorded in §E as positively verified
rather than "fixed". The candidate manifest's historical `perf.scan` row is left as captured.

## 4. Self-test and falsification

```bash
bash .skills/perf-audit/scripts/perf-scan.sh --self-test
→ 14 assertions, SELF-TEST PASS, exit 0   (no build, no benchmark)
```

Covered: clean full pass ⇒ unqualified `PASS` + exit 0; uncommitted tree named, still exit 0; warning
⇒ `WARN` + exit 2; failure ⇒ `FAIL` + exit 1; skipped section ⇒ `PASS (partial)` + section named;
`skip()` accounts and records; `sha256_of` agrees with `openssl dgst -sha256`.

Falsified with three probes on copies before the test was trusted:

| Probe | Result |
|---|---|
| warning path returns 0 again | 1 assertion failed, exit 1 (`a warning exits 2, never 0: expected '2', got '0'`) |
| skipped sections folded back into a bare `PASS` | 2 assertions failed, exit 1 |
| dirty-tree qualifier removed | 1 assertion failed, exit 1 |

Restored script: 14/14, exit 0.

## 5. Gates

Run on this host, working tree of this change (`d9c9eae + docs/tooling`, no Rust source touched):

| Gate | Result | Delta vs 875/0/27 |
|---|---|---|
| `cargo fmt --all -- --check` | exit 0 | — |
| `cargo check --workspace` | exit 0 | — |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 warnings | — |
| `cargo test --workspace` | **875 passed / 0 failed / 27 ignored** | 0 |
| `cargo build --release --locked -p db-pro-native` | exit 0 | — |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | `PASS (partial)` — 4 passed / 0 warnings / 0 failed / **4 sections not executed**, qualified with `+dirty(3)` | new status wording; same 4 executed checks |
| `bash …/perf-scan.sh --self-test` | `SELF-TEST PASS` — 14 assertions, exit 0 | new |

No Rust source changed in this audit, so the CI-mirroring `--include-ignored` leg is unaffected; the
clean-tree confirmation of the perf scan is recorded in the closing comment of #79.

## 6. Not claimed

- No benchmark was run as part of this audit: `all` does not run them (now stated in its output) and no
  perf budget is asserted as met here. The only numbers measured are the tool's own behaviour.
- No claim that the budgets are enforced anywhere: they are guidelines, and §E says so.
- The React-era bundle numbers (main chunk 1,182 → 684 KB, dependencies 87 → 53) are **not** re-derived
  or claimed as current; they belong to the archived frontend.
- The historical release rows quoting `PASS — 4 passed / 0 warnings / 0 failed` are left as captured;
  they record what the old tool printed at `85a7fa3`. Any future scan can be tied to a tree and an
  artifact because the tool now prints both.
