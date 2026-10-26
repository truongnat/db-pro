# perf-scan Artifact-Path Resolution — Plan

State: REVIEW
Branch: `fix/perf-scan-artifact-path` (base `main` at `a31a2bec2aa09067dea31a480108d2b3526cacd5`)

## Goal

`bash .skills/perf-audit/scripts/perf-scan.sh` must report the **measured** size and sha256
of the `db-pro-native` release binary that the build in the same run produced, whatever the
host's cargo configuration is, instead of failing (or silently measuring nothing) because it
looked for the binary at a path it guessed.

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
- Cause: the script hard-coded `$PROJECT_ROOT/target/release/db-pro-native`
  (pre-fix `perf-scan.sh:173`). The relocation on this machine comes from `build.target-dir`
  in the **user-level** `~/.cargo/config.toml` (`target_directory` resolves to
  `/data/cargo-target`). `.cargo/config.toml:5`'s `build.build-dir = "target-build"` is
  **not** the cause — it moves intermediates only and leaves final binaries in `target/`,
  as its own comment says.
- Why it matters rather than being cosmetic: `perf.scan` is an `evidence-manifest.json`
  gate row (`docs/release/rc1-p2-release-dispositions.md:137`) and the binary-size budget is
  a release budget row (`.skills/perf-audit/references/perf-budgets.md:18`), so on a
  redirected host the budget is unmeasurable while the run still looks like an audit happened.

## Scope

- The size check resolves the artifact from **cargo's own build output** (`--message-format=
  json-render-diagnostics`, then the `compiler-artifact` message's `executable` field), not
  from a reconstructed path. That covers, in one mechanism, every layout cargo can produce:
  a relocated `target-dir`, a `build.target`/`CARGO_BUILD_TARGET` triple subdirectory, and
  the Windows `.exe` name.
- The failure branch keeps reporting **no size at all** when the build fails, and now also
  when cargo names no executable: there is no guessed fallback, because a guessed path that
  exists may hold a STALE binary from an older configuration, which the file's provenance
  rule forbids measuring.
- JSON-escaped Windows paths (`C:\\…`) are unescaped before the filesystem check.
- `--self-test` asserts the extraction against canned cargo NDJSON (lib-only artifact, plain
  release, triple, `.exe`, another package's bin, empty log, escaped Windows path).
- Correct the documents that told the reader to `ls target/release/…`:
  `.skills/perf-audit/SKILL.md`, `.skills/perf-audit/references/perf-budgets.md`,
  and the README build-output comment.

## Non-goals

- No change to any budget threshold, benchmark id, or the size check's pass/warn/fail
  boundaries.
- No change to `.github/workflows/release.yml`, which names `target/release/db-pro-native[.exe]`
  per platform in its own upload matrix (`release.yml:306-314`). Those run on runners with the
  default layout; if that ever gains a `target-dir` override it is the same class of bug, and
  is recorded as a follow-up rather than folded into this PR.
- Release docs that quote historical `target/release/…` paths as captured evidence stay as
  captured.

## Acceptance criteria

1. `perf-scan.sh native` on this host reports a size and sha256 for the path cargo named,
   and exits 0.
2. `perf-scan.sh --self-test` passes, including the artifact-extraction assertions.
3. Nothing is measured when cargo names no executable: an artifact-less build output (e.g. a
   lib-only `compiler-artifact` message, whose `executable` is `null`) must produce a FAIL
   naming the empty result, never a path found by guessing. This is covered by the canned-JSON
   assertions plus the build-failure branch, so it does not depend on host state.
4. No budget row or threshold changes.
5. `cargo +nightly fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `python3 tools/cc-scan.py . --fail-on error` stay green.
