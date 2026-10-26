# Checklist — clean-code baseline & repo-wide ratchet

## Planning
- [x] Measured baseline recorded (empty `.clean-code-baseline.json`, 23 errors +
      129 warnings, `cc-scan` not wired to any workflow) — PLAN.md "Evidence / baseline"
- [x] Freeze precondition defined before freezing: no session-introduced finding may
      remain open (gap report § 3 Nhóm 2)
- [x] Scope/non-goals explicit: no refactor in this change; warnings not frozen; no
      mass `cc-scan:allow` comments
- [x] Provider matrix: n/a — static analysis, no database access

## Verification of the precondition
- [x] Finding-file set ∩ `git diff d1d7dc4f..HEAD` = 2 files, both proven outside
      their own hunks (VERIFICATION.md § 1)
- [x] `audit.rs` re-scanned at 100/A after `72feea0d` — the last session-introduced
      findings removed before any freeze
- [x] Direct scan of all branch-touched Rust files: nothing attributable to the branch

## Implementation
- [x] Baseline frozen with **error-severity only**: 15 `HUGE_FUNCTION`,
      7 `HARD_COMPLEXITY`, 1 `HARD_PARAMS` = 23
- [x] Repo-wide gate decision recorded and **applied**: `ci.yml` step
      "Clean code ratchet (repo-wide)" running `python3 tools/cc-scan.py . --fail-on error`
- [x] Gate proven green at HEAD (exit 0, "ignored 23 findings frozen")
- [x] Gate proven live by negative test (synthetic 96-line function → exit 1),
      probe deleted afterwards
- [x] Warnings proven still visible (`--fail-on warning` → exit 1, 126 reported)
- [x] All 23 findings classified into 4 groups with the reason read from the source
      (FINDINGS.md), including 21-functions/23-findings reconciliation
- [x] Baseline weaknesses documented rather than oversold (FINDINGS.md § "Known
      weaknesses", items 1-4)

## Open / follow-up
- [ ] **Owner sign-off on the second CI gate.** Applied on `main` and unpushed; the
      diff ratchet alone was already green, so this is added protection, not a fix
      for a broken gate. Revert = delete one step.
- [ ] First real CI run of the new step (no local YAML parser to pre-validate)
- [ ] Group C refactors, in the value order of FINDINGS.md:
      `sql_embeds_credential_literal` (27) → `is_user_visible_schema` (20) →
      `alter_existing_table` (18) → `parse_sqlite_explain_json` (18) → `execute_batch` →
      `backup` → `prepare_frame` → `plan_design_draft`. One PR each, with the baseline
      count for that file lowered in the same PR.
- [ ] Group D: replace `open_data_cleanup_for_capture`'s 171-line dispatch chain with a
      lookup table (cheapest structural win, capture-only risk)
- [ ] Do not repeat the boilerplate allow `— linear pipeline — one cohesive pass`
      at new sites without a site-specific argument

## Exit criteria met
Baseline is frozen from verified-legacy findings only, the repo-wide gate is green
and demonstrably blocking, and every accepted violation has a written reason.

## Tổng kết (Vietnamese)

Điều kiện tiên quyết đã được kiểm chứng trước khi đóng băng: không còn finding nào
do nhánh này tạo ra (giao cắt chỉ 2 file và các hunk nằm ngoài hàm bị báo lỗi;
`audit.rs` đạt 100/A sau `72feea0d`). Baseline chứa đúng 23 finding mức error,
warning vẫn được báo đầy đủ. Cổng `cc-scan --fail-on error` đã vào `ci.yml`, đã
xanh tại HEAD và được chứng minh là thật chặn bằng phép thử hàm 96 dòng. Phần còn
lại là quyết định của owner (duyệt cổng thứ hai trong CI) và danh sách refactor
Group C/D theo thứ tự giá trị — mỗi việc một PR, hạ count baseline tương ứng trong
chính PR đó.
