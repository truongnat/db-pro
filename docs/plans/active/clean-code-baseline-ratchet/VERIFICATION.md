# Verification — clean-code baseline & repo-wide ratchet

All commands were executed on this machine at HEAD `52507402`
(Linux/x86_64, cc-scan v1.0.1). Output is verbatim except where trimmed for length.

## 1. Inheritance proof (freeze precondition)

```bash
git diff --name-only d1d7dc4f..HEAD            # files the release-gap work touched
python3 tools/cc-scan.py . --json -o /tmp/cc.json --no-baseline
```

Intersection computed by script:

```text
findings: 149 files: 89
overlap with branch-changed files: ['crates/app/src/app/app_lifecycle.rs', 'crates/native-app/src/main.rs']
by severity: {'warning': 126, 'error': 23}
```

Hunk containment for the two overlapping files:

```text
app_lifecycle.rs  @@ -76,3 +76 @@   @@ -89,0 +88,19 @@    findings at 173, 326   → outside
main.rs           @@ -605,0 +606,6 @@                     finding at 353         → outside
```

Direct scan of the branch's Rust files:

```bash
python3 tools/cc-scan.py crates/app/src/app/app_lifecycle.rs \
  crates/core/src/domain/audit.rs crates/core/src/domain/connection.rs \
  crates/infrastructure/src/ssh/tunnel.rs \
  crates/app/src/settings/settings_system_view.rs --no-baseline
```

```text
cc-scan v1.0.1 · 5 file · /data/dev/work/db-pro
Clean Code score: 94.0/100 (grade A)  · error=1 warning=2 info=0

── crates/app/src/app/app_lifecycle.rs
     173  erro  HARD_COMPLEXITY  `prepare_frame` has ~18 logical branches (hard limit 15)
     173  warn  LONG_FUNCTION    function `prepare_frame` is 68 lines (recommended <= 50)
     326  warn  LONG_FUNCTION    function `draw_overlays` is 58 lines (recommended <= 50)
```

`audit.rs` scores 100/A with `error=0 warning=0` after `72feea0d`. The only
findings in branch-touched code belong to `prepare_frame` / `draw_overlays`, both
inherited. **Freeze precondition satisfied: zero session-introduced findings.**

## 2. Baseline content

Written from the 23 error-severity findings only (warnings deliberately left
unfrozen):

```text
rules: {'HARD_COMPLEXITY': 7, 'HUGE_FUNCTION': 15, 'HARD_PARAMS': 1}
total frozen: 23   → .clean-code-baseline.json, 33 lines, keyed rule → file → count
```

## 3. Gate is green at HEAD

```bash
python3 tools/cc-scan.py . --fail-on error   → exit 0
```

```text
Clean Code score: 0.0/100 (grade E)  · error=0 warning=126 info=0
(ignored 23 findings frozen in the baseline)
```

Runtime 8.2 s wall — acceptable as a CI step. The grade stays E because 126
warnings remain **visible**; the gate blocks errors only.

## 4. Gate is live, not decorative (negative test)

A 96-line function in a file the baseline does not name:

```bash
mkdir ratchet-probe && <generated probe.rs with a 96-line fn>
python3 tools/cc-scan.py . --fail-on error   → exit_code=1
```

```text
── ratchet-probe/probe.rs
     1  erro  HUGE_FUNCTION  function `probe` is 96 lines (must split above 70)
Clean Code score: 0.0/100 (grade E)  · error=1 warning=126 info=0
(ignored 23 findings frozen in the baseline)
```

Probe directory deleted afterwards (`rm -rf ratchet-probe`); verified absent and
never staged.

## 5. Warnings were not hidden

```bash
python3 tools/cc-scan.py . --fail-on warning → exit 1, 126 warnings reported
```

Confirms the freeze covers error severity only.

## 6. CI step

`.github/workflows/ci.yml` — new step after the existing diff scan:

```yaml
      - name: Clean code ratchet (repo-wide)
        run: python3 tools/cc-scan.py . --fail-on error
```

`python3` is already used by this job (`ci.yml:85` binds the fixture SSH port via
`python3 -c`), so no new toolchain step was added. **Not verified in CI**: no YAML
parser was available locally (`python3 -c "import yaml"` → `ModuleNotFoundError`),
so the step was checked by structural comparison with adjacent steps and by running
its command locally (item 3). First real confirmation is the next CI run on a PR.

## Evidence gaps / not tested

1. CI execution of the new step (see 6).
2. The ratchet is (rule, file)-count based, so "fix one + add one in the same file"
   passes silently — measured behaviour of `apply_baseline`
   (`tools/cc-scan.py:874-886`), documented in FINDINGS.md, not tested.
3. Whether the owner accepts a second clean-code gate in CI at all; the diff
   ratchet alone was already green.
4. Group C refactors — none performed here, so no test evidence exists for them.

## Tổng kết (Vietnamese)

Toàn bộ 23 lỗi `cc-scan` toàn repo đã được chứng minh là **nợ kế thừa**: giao cắt
giữa danh sách 89 file có finding và `git diff d1d7dc4f..HEAD` chỉ trả về hai file,
và các hunk của nhánh nằm ngoài `prepare_frame` / `initialize_sample_sqlite`. Ba
finding do nhánh tạo ra trong `audit.rs` đã bị loại bỏ bằng `72feea0d`, nên điều
kiện "không freeze để che lỗi mới" được thỏa trước khi đóng băng. Baseline mới chỉ
gộp **23 finding mức error** (15 `HUGE_FUNCTION`, 7 `HARD_COMPLEXITY`, 1
`HARD_PARAMS`); 126 warning vẫn được báo và vẫn làm `--fail-on warning` exit 1,
tức không có gì bị che. Cổng toàn repo đã được bật trong `ci.yml` và đã kiểm chứng
bằng phép thử phủ định: thêm một hàm 96 dòng ở file chưa nằm trong baseline khiến
scan exit 1 (`error=1`), sau đó file thử nghiệm đã bị xóa. Bảng phân loại trong
`FINDINGS.md` chia 21 hàm thành bốn nhóm — bảng token/ khai báo (chấp nhận, không
refactor), pipeline vẽ UI (chấp nhận, theo dõi), hàm phức tạp thật cần refactor
theo thứ tự giá trị (`sql_embeds_credential_literal` 27 nhánh đứng đầu), và tooling
capture — đồng thời ghi lại bốn điểm yếu đã biết của cơ chế baseline để không ai
tưởng nó mạnh hơn thực tế. Chưa kiểm chứng: bước CI mới chạy thật trên GitHub
(không có YAML parser ở máy local), và chưa refactor hàm nào.
