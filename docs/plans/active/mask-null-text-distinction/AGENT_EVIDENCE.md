# Agent evidence — truongnat/db-pro#502 mask_cell NULL vs text "null"

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | opencode (deepseek-v4.1-flash-free) · fix lane |
| Issue(s) | truongnat/db-pro#502 and #503 |
| Task state | `Review` |
| Baseline SHA | `d5aea9568430c5391bb6150a9748332fd2e0d62e` — everything below is asserted against this tree |
| Branch / PR | `fix/mask-null-text-distinction` · no commit / no PR (parent publishes) |
| Scope interpretation | Make `mask_cell` distinguish SQL NULL from present text (#502), and make the preview warn when a configured masking rule matches no header without changing exact case-sensitive identifier semantics (#503). |
| Out of scope | Changing `TransferRow.cells` (`Vec<String>`) to carry NULL; wiring masking to `UiCell`/a live export path; changing `MaskRule`/`ColumnMask`. |

## 2. Progress checkpoint

- Current HEAD: `d5aea9568430c5391bb6150a9748332fd2e0d62e` (branch, uncommitted working-tree change)
- Completed acceptance rows:
  - [x] `mask_cell` takes `Option<&str>`; `None` → `NULL`; `Some` always rules
  - [x] Removed both `eq_ignore_ascii_case("null")` checks
  - [x] `mask_row`/`preview_masking` null-aware; `mask_transfer_batch` documents pipeline NULL-as-`""`
  - [x] App callers updated; regression tests added for #502 and #503 warning semantics
- Remaining acceptance rows: none for #502
- Findings / risks: `P1` bypass (fixed) — `FINDINGS.md` F1; `P2` transfer cannot express NULL — F2; `P2` preview display conflation — F3
- Tests already run: see §3
- Dependency / blocker changes: none

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | baseline `d5aea9568430c5391bb6150a9748332fd2e0d62e`; change uncommitted on `fix/mask-null-text-distinction` |
| Commit list | none (not committed per task instruction) |
| File / surface inventory | `crates/core/src/domain/masking.rs` — `mask_cell(Option<&str>)`, null-aware `mask_row`, `preview_masking`, `mask_transfer_batch`, private `render_cell(s)` helpers; `crates/core/src/domain/tests/masking_tests.rs` — 8 regression tests added, one prior test renamed; `crates/app/src/tools/masking.rs` — `build_preview` builds `Option` rows; `docs/plans/STATUS.md` — new row |
| Acceptance mapping | "text `null` masked" → `literal_null_text_is_masked_by_every_rule`; "unmatched rule is visible" → `preview_masking_warns_on_unmatched_rule`; "real NULL stays NULL" → `sql_null_is_rendered_not_masked`; "row/preview/transfer callers" → `mask_row_distinguishes_sql_null_from_text_null`, `preview_masking_masks_textual_null`, `transfer_batch_masks_textual_null_cells` |
| Commands and counts | `cargo test -p db-pro-core masking` → 12 passed / 0 failed; `cargo test -p db-pro-app masking` → 2 passed / 0 failed; `cargo test --workspace` → 1957 passed / 0 failed / 42 ignored after both issues; `cargo check --workspace` exit 0 after both issues; `cargo +nightly fmt --all -- --check` clean after both issues; `cargo build --release --locked -p db-pro-native` exit 0; clean-code scan pass 14 / warn 2 / fail 0 |
| CI run IDs / status | not run |
| Known limitations | clippy gate **skipped** — `cargo clippy --workspace --all-targets -- -D warnings` fails on 3 inherited `collapsible_match` errors in `crates/core/src/domain/saved_task.rs` (file byte-identical to `main`); masking files clean. Transfer-path NULL rendering unchanged (F2). |
| Migrations / config implications | none |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | self-review only (implementer ≠ independent approval) |
| Verdict | PENDING independent review |
| P0 / P1 / P2 counts | introduced 0 / 0 / 3 (F2, F3, F4 documented); inherited: clippy baseline 3 errors (non-blocking for this fix) |
| Findings | see `FINDINGS.md` |
| CI disposition | not run |
| Next task(s) unblocked | optionally F2 (transfer NULL model) as a separate plan |

## 5. Research / audit handoff

- Source date: 2026-10-16
- Source URLs / references: `crates/core/src/domain/masking.rs`, `crates/ui/src/runtime/runtime_query_types.rs:12-20`, `crates/core/src/domain/transfer.rs:204-207`, `crates/core/src/application/table_copy.rs:294`, `application/export_formats.rs:122`, `application/json_excel_transfer.rs:146`, `crates/app/src/tools/masking.rs`, `crates/app/src/tools/transfer_harness_view.rs:273-404`
- Factual findings: the bug is real and reproduced at the baseline SHA test `masking_tests.rs:51`; masking has no `crates/ui` caller and is reached only from preview/harness code.
- Inference: practical sensitivity of the bypass is limited to the exact token `null`; classified as P1 because it is the security masking boundary and the output is silently wrong.
- Decision / recommendation: fix the primitive with `Option<&str>`; do **not** refactor the transfer model now (F2). Owner: repository owner.
- Unresolved questions: whether `TransferRow` should carry `Option<String>` before a live masked-export path ships.
- Downstream tasks activated: none (F2 would be a new issue).

## 6. Tổng kết (Vietnamese summary)

Đã gom #502 và #503 trong cùng cụm masking: `mask_cell` phân biệt NULL thật (`None`) với text (`Some`), nên text `null`/`NULL` luôn qua rule; preview giữ so khớp tên cột chính xác, nhưng cảnh báo và nêu tên rule nào không khớp header. Test lọc mới nhất: core 12/12, app 2/2; `cargo check --workspace`, fmt và `git diff --check` pass sau cả hai thay đổi. Full workspace 1.957 pass / 42 ignored sau cả hai thay đổi; release build chạy trước khi thêm #503. Clippy chưa đạt do 3 lỗi `collapsible_match` có sẵn trong `saved_task.rs`. Giới hạn: `TransferRow` không phân biệt NULL với chuỗi rỗng (F2); chưa commit/push/PR — agent cha sẽ publish.
