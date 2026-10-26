# Verification — perf-scan artifact-path resolution

Base SHA (defect reproduced): `a31a2bec2aa09067dea31a480108d2b3526cacd5`
All commands were executed on this host (`linux x86_64`, cargo 1.96.1, user-level
`build.target-dir = /data/cargo-target`). Exit codes are the command's own, taken without a
pipe (`$?` after a pipe measures the last pipe element).

## 1. Defect reproduced before the fix

`bash .skills/perf-audit/scripts/perf-scan.sh` at `a31a2bec`, verbatim:

```text
  Building db-pro-native (release)...
  ✓ cargo build -p db-pro-native: release build succeeded
  ✗ Binary artifact: target/release/db-pro-native not found
...
  Source revision: a31a2bec2aa09067dea31a480108d2b3526cacd5
  Measured artifact: not measured in this run
  Passed: 3 / Warnings: 0 / Failed: 1
  Status: FAIL — 1 failed, 3 passed
```

The artifact existed: `find /data/cargo-target -name db-pro-native -type f` →
`/data/cargo-target/release/db-pro-native`. So the failure was the script's path guess, not
a missing build.

## 2. Fix measured green after the change

`bash .skills/perf-audit/scripts/perf-scan.sh native`, exit code **0**, verbatim lines 4, 8-9
and 29-37 of `/tmp/ps_native.log`:

```text
  Source revision: 425dc9b8496c46dc08944b960c11c70c154f5e2b+dirty(1)
  Building db-pro-native (release)...
  ✓ cargo build -p db-pro-native: release build succeeded
...
  Source revision: 425dc9b8496c46dc08944b960c11c70c154f5e2b+dirty(1)
  Measured artifact: /data/cargo-target/release/db-pro-native sha256
    0698c5bfad1f3c2378994f0683d2626b8ada6df28902aa4c9444c0e0458802b7 (45.0MB)

  Passed: 3
  Warnings: 0
  Failed: 0
  Not executed: 0
  Status: PASS — 3 executed check(s) passed [source 425dc9b8…+dirty(1): working tree not committed]
```

The size line is a real measurement against the documented budget (`< 50 MB` target, `> 100 MB`
critical — `.skills/perf-audit/references/perf-budgets.md:18`), reported with the digest and
revision the provenance rule requires. `+dirty(1)` is the uncommitted docs of this plan; no
measured source file was dirty.

## 3. Contract self-test — hermetic, no build, no host state

`bash .skills/perf-audit/scripts/perf-scan.sh --self-test` → exit **0**, 20 assertions ✓, 0 ✗.
The 6 added for artifact resolution run against canned cargo NDJSON, so they cannot be
influenced by the machine's cargo config or environment:

```text
  ✓ a lib-only artifact yields no path (nothing stale gets measured)
  ✓ the plain release artifact is extracted
  ✓ a target-triple / .exe layout is extracted (last artifact wins)
  ✓ another package's artifact is never measured as ours
  ✓ an empty build log yields no path
  ✓ a JSON-escaped Windows path is unescaped before the filesystem check
```

## 4. Gate is live, not decorative (negative tests)

Two independent proofs, neither dependent on host state:

- **Canned output**: a `compiler-artifact` line with `"executable":null` (a lib) and one
  naming a *different* package's bin both extract to the empty string. `audit_native_binary`
  then hits `check "Binary artifact" "fail" "cargo's build output named no db-pro-native
  executable to measure (reported path: <none>)"` and returns — it has no fallback path left
  to guess, which is what makes a stale measurement impossible rather than merely unlikely.
- **Real failing build**: `cargo build --release --locked -p nope-not-a-package` with the same
  redirection captured `error: package ID specification 'nope-not-a-package' did not match any
  packages` into the diagnostics variable and left the JSON file at **0 bytes**, confirming the
  build-failure branch (`no binary size reported — the artifact of this run was not produced`)
  still sees its error text and cannot measure anything.

## 5. Static gates

| Command | Result |
|---|---|
| `bash -n .skills/perf-audit/scripts/perf-scan.sh` | exit 0 |
| `python3 tools/cc-scan.py .skills/perf-audit/scripts/perf-scan.sh --no-baseline` | `100.0/100 (grade A) · error=0 warning=0 info=0` |
| `python3 tools/cc-scan.py . --fail-on error` | exit 0 (repo-wide ratchet, baseline unchanged) |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | exit 0 |
| `cargo +nightly fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace -- --test-threads=4` | exit 0 — **2320 passed / 0 failed / 65 ignored**, 27 suites |
| `bash scripts/check-ui-architecture.sh` | n/a — no `crates/ui` file touched |

Files changed: `.skills/perf-audit/scripts/perf-scan.sh`, `.skills/perf-audit/SKILL.md`,
`.skills/perf-audit/references/perf-budgets.md`, `README.md` (build-output comment), and this
plan directory. No `.rs` file, no `Cargo.toml`, no threshold, no baseline entry.

## 6. Evidence gaps / not tested

- **No Rust behaviour changed**; the suite is a no-regression check here, not coverage of this
  fix. The fix's own coverage is the `--self-test` section, which is the harness this script
  already uses for its result contract.
- **End-to-end triple/`.exe` run not performed.** Only `x86_64-unknown-linux-gnu` is installed
  here, and building for it under `--target` means a full from-scratch rebuild into a new
  directory. The triple and `.exe` cases are therefore covered by canned cargo NDJSON
  assertions plus the fact that cargo names the path, not by a second real build.
- `perf-scan.sh` sections `er`, `db`, `rust` were **not executed** (window server / live DB).
  The script counts them as `Not executed`; this PR does not claim them.
- The `native` run also executed the criterion suites and printed 8 timing lines, all
  completed, none failed. `perf-scan.sh` prints only `time:` lines without benchmark ids, so
  this run does **not** map those times to individual budget rows; quoting a budget comparison
  needs a run that prints ids.

## Tổng kết (Vietnamese)

Lỗi là script tự đoán đường dẫn binary; trên máy có `build.target-dir` chuyển hướng, build
thành công mà cổng size vẫn FAIL và in "not measured in this run". Fix bỏ hẳn việc đoán: đọc
trường `executable` từ JSON output của chính lần build, nên mọi layout (target-dir đổi chỗ,
triple `build.target`, `.exe` Windows) đều đúng và không còn khả năng đo nhầm binary cũ — khi
cargo không tên executable, cổng FAIL rõ ràng chứ không fallback. Trước sửa exit 1; sau sửa
exit 0, đo 45.0MB kèm sha256 và revision. `--self-test` 20 assertion xanh (6 assertion mới
chạy trên NDJSON dựng sẵn, không phụ thuộc máy), test âm tính với build thất bại chứng minh
cổng vẫn chặn. fmt/clippy/cc-scan/test đều xanh, không đổi ngưỡng hay chạm code Rust.
