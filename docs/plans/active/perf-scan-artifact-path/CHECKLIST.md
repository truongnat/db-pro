# Checklist — perf-scan artifact-path resolution

Branch `fix/perf-scan-artifact-path`, base `a31a2bec`.

## Planning
- [x] Reproduce the failure at the base SHA with the exact command and output (`PLAN.md` §Evidence).
- [x] Establish the cause from the tool itself: `cargo metadata` reports
      `target_directory = /data/cargo-target` while the script looked in `$PROJECT_ROOT/target`.
- [x] Attribute the cause correctly (review finding, cubic P2/P3): the relocation here is
      `build.target-dir` in the user-level `~/.cargo/config.toml`; `build.build-dir` in
      `.cargo/config.toml:5` moves intermediates only and is **not** the cause.
- [x] Confirm the fix is tooling-only — no Rust source, no budget threshold.
- [x] Confirm `perf-audit` is not in `skills-lock.json` (`grep -c perf-audit skills-lock.json` → 0),
      so editing the vendored skill text does not desync a lock.

## Implementation
- [x] The build runs with `--message-format=json-render-diagnostics`; stdout (artifact
      messages) goes to a temp file, stderr (human diagnostics) stays in the failure output.
- [x] `cargo_executable_from_json <bin> <ndjson>` extracts the `executable` cargo actually
      wrote, un-escaping JSON `\\` so a Windows path is usable.
- [x] No path guessing anywhere: no candidate list, no `cargo metadata` fallback, no
      `PERF_SCAN_TARGET_DIR` environment override that could redirect a real audit
      (both earlier designs were rejected after review — see `FINDINGS.md` §Design history).
- [x] `MEASURED_ARTIFACT` prints the path actually measured (provenance contract).
- [x] Build-failure branch still reports no size at all, and now frees the temp file.
- [x] `.skills/perf-audit/SKILL.md` §1 and §8, `references/perf-budgets.md`, and the
      `README.md:197` build-output comment corrected.

## Verification
- [x] `bash -n` on the script → 0.
- [x] `perf-scan.sh --self-test` → SELF-TEST PASS, exit 0, 20 assertions ✓ / 0 ✗.
- [x] `perf-scan.sh native` → size **and** sha256 of `/data/cargo-target/release/db-pro-native`,
      exit 0 (was FAIL / "not measured" at base).
- [x] Negative proof, no host state needed: canned NDJSON whose only artifact is lib-only
      (`"executable":null`) and a canned log naming another package's bin both extract to the
      empty string, i.e. the gate FAILs rather than measuring a guess; and a real failing build
      (`cargo build -p nope-not-a-package`) leaves the JSON file at 0 bytes with the error in
      the captured diagnostics.
- [x] `python3 tools/cc-scan.py .skills/perf-audit/scripts/perf-scan.sh --no-baseline` → 100/A, 0 findings.
- [x] `python3 tools/cc-scan.py . --fail-on error` → exit 0 (baseline count unchanged).
- [x] `cargo +nightly fmt --all -- --check` → 0, `cargo clippy --workspace --all-targets -D warnings` → 0,
      `cargo test --workspace -- --test-threads=4` → 0 (2320 passed / 0 failed / 65 ignored).
      No `.rs` file is in this diff, so the suite is a no-regression check, not coverage.

## Open / follow-up
- `.github/workflows/release.yml:306-314` still names `target/release/db-pro-native[.exe]`
  explicitly in its per-platform upload matrix. Correct on the default-layout runners it uses
  today; worth the same treatment if it ever moves to a redirected target dir.
- The `er`, `db` and `rust` perf sections still need a window server / live connection and were
  not executed here; they are reported as `Not executed`, not as pass.
- Release documents split two ways (`FINDINGS.md` §Not defects). Past evidence quotes
  (`docs/release/0.1.0-release-notes.md:229`) are **left as captured** — rewriting a past
  observation is out of scope. But `0.1.0-packaging.md:165` (binding packaging rule) and
  `0.1.0-handoff.md:125` (local packaging command) are **live guidance** that hard-codes the
  default-layout path; correct on the CI runners, wrong on a `target-dir` host. Editing a binding
  release rule is out of scope for a tool fix → follow-up for the release owner.

## Exit criteria met
The size budget is measured from the artifact cargo reports for this exact run, on any host
layout, and every path that cannot be resolved stays a loud FAIL.

## Tổng kết (Vietnamese)

Điều kiện tiên quyết đã kiểm chứng: lỗi nằm ở việc script tự đoán đường dẫn binary, còn nơi
cargo đặt nó là do cấu hình máy (`build.target-dir` của user-level config ở đây; `build-dir`
của repo chỉ chuyển intermediate chứ không dời binary). Fix không đoán nữa mà đọc trực tiếp từ JSON output của chính lần build đó (`executable` của
artifact `db-pro-native`), nên mọi layout
— target-dir đổi chỗ, triple `build.target`, tên `.exe` trên Windows — đều đúng mà không cần
fallback. Không có đường đoán dự phòng, nên không thể vô tình đo một binary cũ: khi cargo không
tên executable nào, cổng FAIL rõ ràng. Đã thêm các assertion chạy trên NDJSON dựng sẵn (không phụ
thuộc máy), chạy thật thấy exit 0 đo 45.0MB kèm sha256, và test âm tính chứng minh cổng vẫn
chặn.
