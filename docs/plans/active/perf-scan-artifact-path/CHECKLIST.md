# Checklist — perf-scan artifact-path resolution

Branch `fix/perf-scan-artifact-path`, base `a31a2bec`.

## Planning
- [x] Reproduce the failure at the base SHA with the exact command and output (`PLAN.md` §Evidence).
- [x] Establish the cause from the tool itself, not from a guess: `cargo metadata` returns
      `target_directory = /data/cargo-target` while the script looked in `$PROJECT_ROOT/target`.
- [x] Confirm the two configs that move the artifact (`.cargo/config.toml:5` `build-dir`,
      user-level `~/.cargo/config.toml` `target-dir`).
- [x] Confirm the fix is tooling-only — no Rust source, no budget threshold.
- [x] Confirm `perf-audit` is not in `skills-lock.json` (`grep -c perf-audit skills-lock.json` → 0),
      so editing the vendored skill text does not desync a lock.

## Implementation
- [x] `cargo_target_dir()` asks cargo; `PERF_SCAN_TARGET_DIR` overrides it for tests.
- [x] `resolve_release_binary()` precedence: cargo target dir → `CARGO_TARGET_DIR` →
      `$PROJECT_ROOT/target`; first existing wins; full miss returns the first candidate.
- [x] Failure message names the resolved dir, the env value, and the fallback it tried.
- [x] `MEASURED_ARTIFACT` prints the path actually measured (provenance contract).
- [x] `--self-test` extended with 5 sandboxed precedence assertions (no build, no network).
- [x] `.skills/perf-audit/SKILL.md` §1 and §8 text corrected.
- [x] `.skills/perf-audit/references/perf-budgets.md` measurement cell corrected.

## Verification
- [x] `bash -n` on the script.
- [x] `perf-scan.sh --self-test` → SELF-TEST PASS, exit 0.
- [x] `perf-scan.sh native` → size **and** sha256 reported, exit 0 (was FAIL/exit 1 at base).
- [x] Negative test: `PERF_SCAN_TARGET_DIR=/nonexistent-box` → FAIL, exit 1, message names the
      paths tried (the gate still bites; it was not turned into a silent skip).
- [x] `python3 tools/cc-scan.py .skills/perf-audit/scripts/perf-scan.sh --no-baseline` → 100/A, 0 findings.
- [x] `python3 tools/cc-scan.py . --fail-on error` → exit 0 (baseline count unchanged).
- [x] `cargo +nightly fmt --all -- --check` → 0, `cargo clippy --workspace --all-targets -- -D warnings` → 0,
      `cargo test --workspace -- --test-threads=4` → 0 (2320 passed / 0 failed / 65 ignored).
      No `.rs` file is in this diff, so the suite is a no-regression check, not coverage.

## Open / follow-up
- The `er`, `db` and `rust` sections still need a window server / live connection and were
  not executed here; they are reported as `Not executed`, not as pass.
- Release documents that quote historical `target/release/db-pro-native` evidence
  (`docs/release/0.1.0-release-notes.md:229`, `0.1.0-packaging.md:165`) are **left as captured** —
  rewriting past observations is out of scope.

## Exit criteria met
The size budget is measured from the artifact cargo actually produced, on a redirected
target dir, and the failure mode stays loud and named.

## Tổng kết (Vietnamese)
Cổng audit binary-size luôn báo FAIL trên máy có `target-dir`/`build-dir` chuyển hướng vì
script đoán đường dẫn `target/release/` thay vì hỏi cargo. Sửa thành phân giải thứ tự
(cargo target dir → `CARGO_TARGET_DIR` → `target/`), in đúng đường dẫn đã đo kèm sha256,
và khi không tìm thấy thì báo FAIL kèm đầy đủ các vị trí đã thử. Đã thêm 5 assertion vào
`--self-test` và chạy thật: trước sửa exit 1 "not found", sau sửa exit 0 đo được 45.0MB;
thử cố tình chỉ target sai vẫn exit 1, nên cổng không bị biến thành skip im lặng.
