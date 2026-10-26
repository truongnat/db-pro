# Findings — perf-scan artifact-path resolution

Measured at `a31a2bec` (base) and at the fix SHA on this host. Every row below is a command
that was run, not an inference from reading code.

## F-1 — size budget unmeasurable on a relocated target dir (P2, tooling, fixed here)

`git show a31a2bec:.skills/perf-audit/scripts/perf-scan.sh` line 173 hard-coded
`$PROJECT_ROOT/target/release/db-pro-native` after a
successful `cargo build --release --locked`. On this machine the user-level
`~/.cargo/config.toml` sets `build.target-dir = "/data/cargo-target"`, and
`cargo metadata` confirms `target_directory = /data/cargo-target`.

Failure scenario, reproduced: the build succeeds, the check reports
`✗ Binary artifact: target/release/db-pro-native not found`, the summary prints
`Measured artifact: not measured in this run`, and the run exits 1 — while
`/data/cargo-target/release/db-pro-native` exists and is 45.0 MB.

Severity **P2**: it blocks no product behaviour, but `perf.scan` is an
`evidence-manifest.json` gate row and the `< 50 MB` binary-size budget is a release budget
row, so on a redirected machine the release audit either reads a false FAIL or, if someone
"fixed" it by skipping, silently loses the measurement. The fix keeps the failure loud and
makes the measurement real.

### Correction after review (cubic P2 on CHECKLIST:10, P3 on SKILL.md:32)

My first write-up blamed `build.build-dir` in `.cargo/config.toml:5` for moving the final
binary. That was wrong and is corrected here: `build-dir` relocates **intermediates**
(`deps/`, `build/`, `incremental/`) and leaves final binaries in `target/` — the config
file's own comment and `AGENTS.md`'s build-layout section both say so. The relocation on this
host comes solely from the user-level `build.target-dir`.

## F-2 — documents instruct the wrong path (P2, fixed here)

`.skills/perf-audit/SKILL.md` §1 (`ls -lh target/release/db-pro-native`) and §8 (the printed
`Measured artifact:` line), `.skills/perf-audit/references/perf-budgets.md:18` (the budget's
Measurement column) and the `README.md:197` build comment all teach the same hard-coded path,
so a human following the docs reproduces F-1. Corrected in all four places.

## Design history — two rejected mechanisms (review-driven)

Both earlier designs were implemented, measured, and rejected after review. Recorded because
the shape of the fix is the finding:

1. **Candidate list** (`cargo metadata` target dir → `CARGO_TARGET_DIR` → `$PROJECT_ROOT/target`,
   first existing wins). Rejected for two reasons. (a) It still missed any
   `build.target`/`CARGO_BUILD_TARGET` layout, where the artifact lives under
   `<target-dir>/<triple>/release/`, and the Windows name `<bin>.exe`. (b) Worse, its final
   fallback could succeed: if cargo had put the artifact elsewhere, an older binary left in
   `target/release/` would be found and reported as *this run's* measurement — the exact
   stale-artifact case `perf-scan.sh`'s own header forbids. A gate that can silently measure
   the wrong file is worse than one that fails.
2. **`PERF_SCAN_TARGET_DIR` env override** as the test seam. Rejected: an inherited
   environment value would redirect a *production* audit too, producing a false miss or a
   measurement of a stale file. Replaced by a pure function over cargo's own build output,
   which the self-test can drive with canned fixtures without any host state.

Final mechanism: read the `executable` field out of the `compiler-artifact` message of the
build the run just performed. There is nothing left to guess, so there is nothing stale to
grab, and every layout cargo can produce is correct by construction.

## Not defects — recorded as observed

- The pre-existing `--self-test` contract (status line + exit codes 0/1/2, `+dirty(N)`,
  named skipped sections) is sound; it simply had no coverage of artifact resolution.
  Added, not replaced.
- The "no size claim when the build fails" rule is correct and untouched.
- `.github/workflows/release.yml:306-314` names `target/release/db-pro-native[.exe]` per
  platform. Those jobs run on `macos-14`/`ubuntu-latest`/`windows-latest` with cargo's default
  layout and no `target-dir` override, so the paths are accurate today; changing them is not
  needed by this fix and would touch the release pipeline. Left as a follow-up note.
- Two different kinds of `target/…` strings live in `docs/release/*`, and only one of them is
  history. Genuine historical quotes — recorded measurements at a named SHA — are left exactly as
  captured, per the provenance rule; `docs/release/rc1-p2-release-dispositions.md:137` is the
  disposition that put the `Measured artifact: target/release/db-pro-native sha256 …` line into the
  record, and it states the rule for the manifest's past `perf.scan` row only, not for every document.
- Still-live guidance that hard-codes the same default-layout path is **not** a historical quote and
  was not touched here: `docs/release/0.1.0-packaging.md:165` (binding packaging rule: "archives are
  built from the binary in `target/release/`") and `0.1.0-handoff.md:125` (local
  `package-macos.sh 0.1.0 target/release/db-pro-native <outdir>` command). Both are correct for the
  CI/default layout and wrong on a host with a `target-dir` override, like this one. Changing them
  means editing a binding release rule, which is out of scope for a tool fix → follow-up.

## Baseline impact

`.clean-code-baseline.json` unchanged; `cc-scan` on the touched script reports 100/A with 0
findings, and the repo-wide `--fail-on error` gate still exits 0.
