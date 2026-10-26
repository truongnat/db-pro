# Findings — perf-scan artifact-path resolution

Measured at `a31a2bec` (base) and at the fix SHA on this host. No speculation below:
every row is a command that was run.

## F-1 — size budget unmeasurable on a redirected target dir (P2, tooling, fixed here)

`perf-scan.sh` hard-coded `$PROJECT_ROOT/target/release/db-pro-native` after a successful
`cargo build --release --locked`. Two live configurations relocate that artifact:

| Config | Effect |
|---|---|
| `.cargo/config.toml:5` — `build.build-dir = "target-build"` | intermediates leave `target/` |
| `~/.cargo/config.toml` — `build.target-dir = "/data/cargo-target"` | **final artifacts** relocate; `cargo metadata` reports `target_directory = /data/cargo-target` |

Failure scenario, reproduced: the build succeeds, the check reports
`✗ Binary artifact: target/release/db-pro-native not found`, the summary prints
`Measured artifact: not measured in this run`, and the run exits 1 — while
`/data/cargo-target/release/db-pro-native` exists and is 45.0 MB.

Severity: **P2** — it blocks no product behaviour, but `perf.scan` is an
`evidence-manifest.json` gate row and the <50 MB binary-size budget is a release
budget row, so on any redirected machine the release audit either reads a false FAIL or,
if someone "fixes" it by skipping, silently loses the measurement. The fix keeps the FAIL
loud and makes the measurement real.

## F-2 — documents instruct the wrong path (P2, fixed here)

`.skills/perf-audit/SKILL.md` §1 (`ls -lh target/release/db-pro-native`), §8 (the printed
`Measured artifact:` line format) and `.skills/perf-audit/references/perf-budgets.md:18`
(the budget's Measurement column) all teach the same hard-coded path, so a human
following the skill reproduces F-1.

## Not defects — recorded as observed

- The pre-existing `--self-test` contract (status line + exit codes 0/1/2, `+dirty(N)`,
  named skipped sections) is sound; it simply had no coverage of path resolution.
  Added, not replaced.
- The "no size claim when the build fails" rule (`perf-scan.sh`, build-failure branch) is
  correct and was left untouched: measuring a stale artifact is exactly what the
  provenance text forbids.
- Other `target/…` strings in `docs/release/*` are **historical evidence quotes** from
  hosts where the path was real. Left as captured, per the repo's provenance rule
  (`docs/release/rc1-p2-release-dispositions.md:137`).

## Baseline impact

`.clean-code-baseline.json` unchanged; `cc-scan` on the touched script reports 100/A with
0 findings, and the repo-wide `--fail-on error` gate still exits 0.
