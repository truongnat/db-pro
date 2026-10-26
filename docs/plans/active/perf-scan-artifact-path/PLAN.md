# perf-scan Artifact-Path Resolution — Plan

State: REVIEW
Branch: `fix/perf-scan-artifact-path` (base `main` at `a31a2bec2aa09067dea31a480108d2b3526cacd5`)

## Goal

`bash .skills/perf-audit/scripts/perf-scan.sh` must report the **measured** size and sha256
of the `db-pro-native` release binary it built in the same run, on any machine, instead of
failing the size check because it looks for the binary at a path it guessed.

## Evidence / baseline

- Reproduced at `a31a2bec`, on this host, before any edit:

  ```text
  ✓ cargo build -p db-pro-native: release build succeeded
  ✗ Binary artifact: target/release/db-pro-native not found
  ...
  Measured artifact: not measured in this run
  Status: FAIL — 1 failed, 3 passed
  ```

  The build had in fact produced a binary — at `/data/cargo-target/release/db-pro-native`.
- Cause: `.skills/perf-audit/scripts/perf-scan.sh:173` (pre-fix) hard-coded
  `$PROJECT_ROOT/target/release/db-pro-native`. Two configurations move that artifact:
  `.cargo/config.toml:5` sets `build.build-dir = "target-build"`, and this machine's
  `~/.cargo/config.toml` sets `build.target-dir = "/data/cargo-target"`.
  `cargo metadata --format-version 1` resolves `target_directory` to `/data/cargo-target`.
- Why it matters rather than being cosmetic: `perf.scan` is an `evidence-manifest.json`
  gate row (`docs/release/rc1-p2-release-dispositions.md:137`) and the binary-size budget
  row in `docs/architecture/performance-baseline.md` /
  `.skills/perf-audit/references/perf-budgets.md:18` is only quotable with a digest.
  A hard-coded path silently turns a measured budget into an unmeasured one on any
  redirected setup.

## Scope

- `resolve_release_binary` + `cargo_target_dir` in `perf-scan.sh`: candidates are
  cargo's resolved `target_directory`, then `CARGO_TARGET_DIR`, then
  `$PROJECT_ROOT/target`, first existing wins; on a full miss the first candidate is
  returned so the failure message names a real path and the values it tried.
- Extend the existing `--self-test` contract section with sandboxed assertions for that
  precedence (no build, no workspace needed).
- Correct the three documents that instruct the reader to `ls target/release/…`.

## Non-goals

- No change to any budget threshold, benchmark id, or the size check's pass/warn/fail
  boundaries.
- No change to packaging scripts or release docs that quote historical
  `target/release/…` paths as captured evidence — those are past observations.
- No Rust code touched; this is audit tooling.

## Acceptance criteria

1. `perf-scan.sh native` on this host reports a size and sha256, and exits 0.
2. `perf-scan.sh --self-test` passes, including the new precedence assertions.
3. A deliberately wrong `PERF_SCAN_TARGET_DIR` produces the failure message naming the
   paths it tried (the "cannot measure what is not there" path stays loud).
4. No budget row or threshold changes.
5. `cargo +nightly fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `python3 tools/cc-scan.py . --fail-on error` stay green.
