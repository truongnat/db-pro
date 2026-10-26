# Verification — perf-scan artifact-path resolution

Base SHA (failure reproduced): `a31a2bec2aa09067dea31a480108d2b3526cacd5`
All commands below were executed on this host (`linux x86_64`, cargo 1.96.1) with the
working tree at that base + this diff. Exit codes are the command's own, measured without
a pipe (`$?` after a pipe measures the last pipe element).

## 1. Defect reproduced before the fix

`bash .skills/perf-audit/scripts/perf-scan.sh` at `a31a2bec`, verbatim:

```text
  Building db-pro-native (release)...
  ✓ cargo build -p db-pro-native: release build succeeded
  ✗ Binary artifact: target/release/db-pro-native not found
...
  Measured artifact: not measured in this run
  Passed: 3 / Warnings: 0 / Failed: 1
  Status: FAIL — 1 failed, 3 passed
```

The artifact existed: `find /data/cargo-target -name db-pro-native -type f` →
`/data/cargo-target/release/db-pro-native`. `cargo metadata --format-version 1` →
`"target_directory":"/data/cargo-target"`. So the failure was the script's path guess,
not a missing build.

## 2. Fix measured green after the change

`bash .skills/perf-audit/scripts/perf-scan.sh native`:

```text
  Measured artifact: /data/cargo-target/release/db-pro-native sha256
  0698c5bfad1f3c2378994f0683d2626b8ada6df28902aa4c9444c0e0458802b7 (45.0MB)

  Passed: 3
  Warnings: 0
  Failed: 0
  Not executed: 0
  Status: PASS — 3 executed check(s) passed
```

Exit code 0. The size line is a real measurement against the documented budget
(`db-pro-native` target < 50 MB, critical > 100 MB —
`.skills/perf-audit/references/perf-budgets.md:18`), and the run prints the digest the
provenance rule requires.

## 3. Contract self-test (`--self-test`), no build involved

`bash .skills/perf-audit/scripts/perf-scan.sh --self-test` → exit 0,
19 assertions ✓, 0 ✗, including the 5 added for path resolution:

```text
  ✓ a full miss returns the first candidate so the error names it
  ✓ resolution falls back to target/release
  ✓ CARGO_TARGET_DIR wins over target/release
  ✓ cargo's target dir wins over CARGO_TARGET_DIR
  ✓ the cargo target dir is used verbatim, not guessed from the project path
```

They run in a `mktemp -d` sandbox with `PERF_SCAN_TARGET_DIR` stubbing the cargo query, so
they assert precedence without depending on the host's cargo config, and they do not touch
the real target dir.

## 4. The gate is still live, not decorative (negative test)

`PERF_SCAN_TARGET_DIR=/nonexistent-box perf-scan.sh native` → exit **1**:

```text
  ✗ Binary artifact: no db-pro-native at /nonexistent-box/release/db-pro-native
     (cargo target dir '/nonexistent-box', CARGO_TARGET_DIR=unset, /data/dev/work/db-pro/target/release/)
  Status: FAIL — 1 failed, 2 passed
```

The fix had to keep a miss loud: resolving a path could easily have degraded into
"silently measure nothing". The message now names every location it tried.

## 5. Static gates

| Command | Result |
|---|---|
| `bash -n .skills/perf-audit/scripts/perf-scan.sh` | exit 0 |
| `python3 tools/cc-scan.py .skills/perf-audit/scripts/perf-scan.sh --no-baseline` | `100.0/100 (grade A) · error=0 warning=0 info=0` |
| `python3 tools/cc-scan.py . --fail-on error` | exit 0 (repo-wide ratchet, baseline count unchanged) |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | exit 0 |
| `cargo +nightly fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace -- --test-threads=4` | exit 0 — **2320 passed / 0 failed / 65 ignored** across 27 suites (unchanged from the base SHA; no `.rs` file is in this diff) |
| `bash scripts/check-ui-architecture.sh` | n/a — no `crates/ui` file touched |

`git diff --stat`: `.skills/perf-audit/SKILL.md`, `.skills/perf-audit/references/perf-budgets.md`,
`.skills/perf-audit/scripts/perf-scan.sh` only (plus this plan directory). No `.rs` file,
no `Cargo.toml`, no threshold, no baseline entry.

## 6. Evidence gaps / not tested

- **No Rust behaviour changed**, so the test suite is a no-op regression check here rather
  than coverage of this fix. Its value is proving the tooling change broke nothing.
- The new path-resolution assertions are covered by `--self-test`, not by `cargo test`:
  the code under test is a shell function, and the script's own self-test section is the
  harness the repo already uses for its result contract.
- `perf-scan.sh` sections `er`, `db`, `rust` were **not executed** (need a window server /
  live DB connection). The script counts them as `Not executed`; this PR does not claim them.
- The `native` section also ran the criterion suites and printed 8 timing lines
  (all completed, none failed), but the tail-captured output does not carry the benchmark
  ids, so this run does **not** map those times to individual budget rows. Any budget
  comparison must be quoted from a run that prints the ids.
- Only one host's cargo configuration was exercised for the redirect case
  (`build.target-dir` at user level). The `build-dir` case is covered by the sandboxed
  precedence assertions, not by a second real machine.

## Tổng kết (Vietnamese)

Cổng audit binary-size báo FAIL oan và in "not measured in this run" trên mọi máy có
`target-dir`/`build-dir` chuyển hướng, vì script đoán đường dẫn `target/release/`.
Sau sửa, cùng lệnh đo thật: 45.0MB kèm sha256 của artifact mà cargo đã tạo, exit 0.
Thứ tự phân giải là cargo target dir → `CARGO_TARGET_DIR` → `target/`, có 5 assertion
chạy trong sandbox `mktemp -d` và một phép thử âm tính chứng minh cổng vẫn FAIL (exit 1)
khi không tìm thấy binary, kèm message liệt kê đủ các chỗ đã thử. Ba tài liệu hướng dẫn
đường dẫn sai cũng đã được sửa; không thay đổi ngưỡng budget hay code Rust nào.
