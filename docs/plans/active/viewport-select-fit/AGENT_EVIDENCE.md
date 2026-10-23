# Agent evidence — Viewport Select and Radio Allocation Fit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · implementation lane (two delegated workers failed on provider usage limits; the work was completed in-session) |
| Issue(s) | Owner-reported horizontal overflow and poor complex Select layout; no issue number assigned |
| Task state | `In Progress` — implementation landed and self-verified; the plan stays `RUNTIME_VERIFY` |
| Baseline SHA | `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` |
| Branch / PR | `main` (direct edit per owner override) · no PR — pushed as `9e72de50` |
| Scope interpretation | Fix the two reproduced defects (horizontal selection-control allocation, query context picker geometry/identity) plus the component-first violation in the touched Appearance caller, and produce a runtime matrix for the changed surfaces. |
| Out of scope | Application-wide responsive certification; `responsive-layout-primitives` Grid/Container work; database behavior; speculative `SelectItem`/search/typeahead redesign. |

## 2. Progress checkpoint

- Current HEAD: `1e39b606` (docs) — code under test is `9e72de50fa821926a3874949e2ebc65289885d0a`
- Completed acceptance rows:
  - [x] Horizontal `Radio`/`Checkbox` allocate content width and stay inside the parent
  - [x] Vertical descriptive choices keep the full-row click target
  - [x] Picker width bounded and shrink-wrapped; no duplicate `ScrollArea` ID banners
  - [x] Red→green regression tests measuring rendered geometry
  - [x] 12-cell runtime matrix for the two changed surfaces, every PNG inspected
  - [x] Required Rust gates + locked native release build
  - [x] `cargo test --workspace` → 2,269 passed / 0 failed / 65 ignored
- Remaining acceptance rows:
  - [ ] The 65 `#[ignore]`d provider integration tests (need live PG/MySQL/SQL Server/SSH fixtures)
  - [ ] CI "Rust checks" + independent review (no PR opened)
  - [ ] Explorer toolbar capture (hook opens Welcome instead of the filter popover)
  - [ ] Dense schema list and picker loading/error/empty captures
  - [ ] 900/1080-height verification (host clamps to 838 logical)
- Findings / risks: F1–F6 in `FINDINGS.md`, all P2, all resolved. No P0/P1.
- Tests already run: `cargo test --workspace` exit 0 → 2,269 passed / 0 failed / 65 ignored; the
  touched-crate subset is `db-pro-ui` 724 + `db-pro-app` 679 passed / 0 failed.
- Dependency / blocker changes: two subagents could not start (`usage_limit_reached`); no scope change.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `9e72de50fa821926a3874949e2ebc65289885d0a` (code) · docs HEAD `1e39b606` |
| Commit list | `9e72de50` fix(ui): cap horizontal selection controls and compact the context picker · `f7669b93` docs(ui-audit): record findings, plan and 12-cell evidence · `f17a1d4b` docs(viewport-select-fit): coding-lane agent evidence handoff · `1e39b606` docs(viewport-select-fit): record workspace gates and re-verification |
| File / surface inventory | `crates/ui/src/components/selection/ui.rs` (`.compact(bool)` + parent-derived cap) · `selection/API.md` (width contract) · `selection/tests/layout_bounds_tests.rs` (new, 4 tests) · `radio_group/ui.rs` (propagate compact) · `radio_group/API.md` · `app/.../query_context_picker_view.rs` (width constants, `auto_shrink`, distinct `id_salt`, empty states) · `app/.../tests/query_context_picker_view_tests.rs` (new, 3 tests) · `app/.../settings_appearance_view.rs` (raw `ui.checkbox` → `components::Checkbox`) · `app/.../workspace_actions.rs` + `native-app/src/capture.rs` (`DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER` hook) · `docs/plans/STATUS.md` · `docs/plans/active/viewport-select-fit/**` · `docs/ui-audit/viewport-select-review/**` |
| Acceptance mapping | Overflow → `horizontal_radios_do_not_consume_entire_row_width`, `horizontal_checkboxes_do_not_consume_entire_row_width` · narrow parent → `compact_rows_honour_a_parent_wider_than_the_glyph_floor`, `compact_rows_keep_the_glyph_floor_in_a_sliver_parent` · picker geometry → `query_context_picker_has_compact_bounds_for_short_lists`, `query_context_picker_caps_to_a_narrow_viewport` · dismissal → `query_context_picker_action_on_escape_closes` |
| Commands and counts | `cargo +nightly fmt --all -- --check` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 / 0 diagnostics · `cargo check --workspace` exit 0 · `cargo build --release --locked -p db-pro-native` exit 0 · `cargo test --workspace` exit 0, **2,269 passed / 0 failed / 65 ignored** · `scripts/check-ui-architecture.sh` PASS · clean-code ratchet exit 0 (11 pass / 5 warn / 0 fail) |
| CI run IDs / status | not run — pushed directly to `main`; GitHub reported bypassed rules ("Changes must be made through a pull request", required check "Rust checks" expected) |
| Known limitations | See §2 remaining rows; plus picker keyboard coverage is Escape-only and pointer-outside dismissal was not re-tested. |
| Migrations / config implications | None. New opt-in capture env var `DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER`; `.compact(bool)` is additive API. |
| Out-of-scope changes | `none` |

Verification detail, matrix table and per-cell observations live in `VERIFICATION.md`.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review performed |
| Verdict | `ACCEPT WITH P2` for the self-verified scope only; **not** an independent approval |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 0 · inherited-and-fixed in scope: 0 / 0 / 6 |
| Findings | F1–F6 in `FINDINGS.md`; F5/F6 were found by the new narrow-width tests failing against the first implementation |
| CI disposition | not executed |
| Next task(s) unblocked | Provision provider fixtures and run the 65 ignored integration tests; add the Explorer-filter capture hook; seed a dense schema list for a long-list capture |

## 5. Research / audit handoff

n/a — this is the implementation lane for the assessment recorded in
`docs/ui-audit/viewport-select-review/AGENT_EVIDENCE.md`.

## 6. Tổng kết (Vietnamese summary)

Đã sửa hai lỗi được báo: (1) Radio/Checkbox trong layout ngang chiếm hết chiều rộng còn lại nên
Appearance đẩy "Dark" ra mép và cắt chữ — nay có `.compact(bool)`, mặc định suy ra từ hướng layout
của parent và cap theo chiều rộng parent đọc **trước** khi vào hàng con; (2) popup connection/schema
chỉ đặt `min_width` nên phình ~600pt và hai `ScrollArea` không co, trùng ID — nay width bị chặn theo
viewport (trừ cả inset của surface), hai vùng co theo nội dung và có `id_salt` riêng.

Test biên hẹp do reviewer yêu cầu đã **fail với implementation đầu tiên** và phát hiện hai lỗi thật
(F5: cap đọc sai nguồn width; F6: thiếu inset surface) trước khi xanh.

Gate đã chạy thật: fmt, `cargo check --workspace` 0, `cargo clippy --workspace --all-targets -D
warnings` 0 chẩn đoán, ui-architecture PASS, **release build locked PASS**, và `cargo test
--workspace` **2.269 pass / 0 fail / 65 ignored**. Matrix 12 ô được capture lại từ cây cuối và đọc
từng ảnh; 1440/1920 bị host clamp còn 838 logical, các ô `exit=-6` là crash TouchBar macOS **sau
khi** ghi ảnh — không phải lỗi UI. Smoke lại trên cây đã push: ô Appearance tái tạo **byte-identical**
(trùng SHA-256 manifest) nên fix deterministic; ô picker khác hash chỉ vì status bar in đường dẫn
data dir.

Còn thiếu: 65 integration test provider đang `#[ignore]` (cần fixture live), CI/review độc lập, ảnh
toolbar Explorer, danh sách schema dày và state loading/error/empty của picker, và chiều cao
900/1080 thật. Vì vậy plan giữ `RUNTIME_VERIFY`, chưa `COMPLETED`. Code đã commit và push
(`9e72de50`, docs tới `1e39b606`) trực tiếp lên `main` theo owner override, đồng thời đẩy luôn 3
commit local có sẵn (`a1730a3e`, `6c8b5fc8`, `9b50e7a2` — docs + colocation đã ghi trong STATUS.md),
không có commit lạ nào khác.
