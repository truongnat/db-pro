# Agent evidence — Viewport Select and Radio Allocation Fit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · implementation lane (two delegated workers failed on provider usage limits; the work was completed in-session) |
| Issue(s) | Owner-reported horizontal overflow and poor complex Select layout; no issue number assigned |
| Task state | `In Progress` — implementation landed and self-verified; the plan stays `RUNTIME_VERIFY` |
| Baseline SHA | `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` |
| Branch / PR | `main` (direct edit per owner override) · no PR — pushed as `7e81a380` (code: `457cd645`) |
| Scope interpretation | Fix the two reproduced defects (horizontal selection-control allocation, query context picker geometry/identity) plus the component-first violation in the touched Appearance caller, and produce a runtime matrix for the changed surfaces. |
| Out of scope | Application-wide responsive certification; `responsive-layout-primitives` Grid/Container work; database behavior; speculative `SelectItem`/search/typeahead redesign. |

## 2. Progress checkpoint

- Current HEAD: `7e81a380` (docs) — code under test is `457cd6457480ff5f23f8a3e5000e9efcc771ad0b`
- Completed acceptance rows:
  - [x] Horizontal `Radio`/`Checkbox` allocate content width and stay inside the parent
  - [x] Vertical descriptive choices keep the full-row click target
  - [x] Picker width bounded, list regions sized to their rows, no duplicate `ScrollArea` ID banners
  - [x] Light chrome separated from the canvas without collapsing hover/selected states
  - [x] Red→green regression tests measuring rendered geometry (10 new tests: 4 selection, 5 picker, 1 palette ladder)
  - [x] 17-cell runtime matrix for the changed surfaces, every PNG inspected
  - [x] Required Rust gates + locked native release build + the 65 live provider tests
- Remaining acceptance rows:
  - [ ] CI "Rust checks" + independent review (no PR opened)
  - [ ] 900/1080-height verification (host clamps to 838 logical)
  - [ ] Dense data views and the ER canvas re-inspected under the new light chrome step
  - [ ] Deep-scroll traversal of a 24-row schema list (first five rows captured)
- Findings / risks: F1–F7 in `FINDINGS.md`, all P2, all resolved. No P0/P1.
- Tests already run: `cargo test --workspace` exit 0 → 2,272 passed / 0 failed / 65 ignored, and the
  65 ignored provider tests then ran live: pg_integration 40/40, pg_object_identity 2/2,
  mysql_integration 12/12, sqlserver_integration 9/9, ssh_backup_runtime_verification 1/1,
  data_diff_smoke 1/1. Touched-crate subset: `db-pro-ui` 725 + `db-pro-app` 681 passed / 0 failed.
- Dependency / blocker changes: two subagents could not start (`usage_limit_reached`); no scope change.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `457cd6457480ff5f23f8a3e5000e9efcc771ad0b` (code) · docs HEAD `7e81a380` (later docs-only commits do not touch code) |
| Commit list | `9e72de50` fix(ui): cap horizontal selection controls and compact the context picker · `f7669b93` docs(ui-audit): findings, plan and matrix evidence · `f17a1d4b` docs(viewport-select-fit): coding-lane handoff · `1e39b606` docs(viewport-select-fit): workspace gates and re-verification · `be45ed15` docs(viewport-select-fit): pin records to committed SHAs · `8a8f1749` fix(ui): give light chrome its own step above the canvas · `457cd645` fix(ui): keep a distinct hover step above the new light chrome · `ddc6a207` docs(viewport-select-fit): pin verification to the corrected-palette SHA · `493e3b9b` docs(viewport-select-fit): correct the handoff records against the final tree · `5829c0ce` docs(plans): open the layout aesthetics audit · `7e81a380` docs(viewport-select-fit): correct F2 resolution and plan acceptance |
| File / surface inventory | `crates/ui/src/components/selection/ui.rs` (`.compact(bool)` + parent-derived cap) · `selection/API.md` · `selection/tests/layout_bounds_tests.rs` (new, 4 tests) · `radio_group/ui.rs` + `API.md` · `crates/ui/src/theme/tokens/semantic.rs` + `primitive.rs` (light chrome step, `NEUTRAL_525`, dead steps removed) · `crates/ui/src/theme/tests/mod_tests.rs` (palette contract + ladder test) · `app/.../query_context_picker_view.rs` (width constants, allocated list regions, distinct `id_salt`s) · `app/.../tests/query_context_picker_view_tests.rs` (new, 5 tests) · `app/.../settings_appearance_view.rs` (`ui.checkbox` → `components::Checkbox`) · `app/.../explorer_toolbar_view.rs` (stable popover id) · 4 transient popups → `surface_elevated` · `app/.../workspace_actions.rs` + `native-app/src/capture.rs` (picker/dense/popover/light fixtures) · `docs/plans/STATUS.md` · `docs/plans/active/viewport-select-fit/**` · `docs/ui-audit/viewport-select-review/**` |
| Acceptance mapping | Overflow → `horizontal_radios_do_not_consume_entire_row_width`, `horizontal_checkboxes_do_not_consume_entire_row_width` · narrow parent → `compact_rows_honour_a_parent_wider_than_the_glyph_floor`, `compact_rows_keep_the_glyph_floor_in_a_sliver_parent` · picker geometry → `query_context_picker_has_compact_bounds_for_short_lists`, `query_context_picker_caps_to_a_narrow_viewport`, `query_context_picker_expands_its_scroll_window_for_long_lists` · dismissal → `query_context_picker_action_on_escape_closes`, `query_context_picker_closes_on_click_outside` · chrome separation → `light_surface_ladder_keeps_hover_and_selected_distinct_from_chrome` |
| Commands and counts | `cargo +nightly fmt --all -- --check` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 / 0 diagnostics · `cargo check --workspace` exit 0 · `cargo build --release --locked -p db-pro-native` exit 0 · `cargo test --workspace` exit 0, **2,272 passed / 0 failed / 65 ignored** · those 65 ignored provider tests then ran live and passed 65/65 (40 + 2 + 12 + 9 + 1 + 1) · `scripts/check-ui-architecture.sh` PASS · clean-code ratchet exit 0 (11 pass / 5 warn / 0 fail) |
| CI run IDs / status | not run — pushed directly to `main`; GitHub reported bypassed rules ("Changes must be made through a pull request", required check "Rust checks" expected) |
| Known limitations | See §2 remaining rows; picker keyboard coverage is Escape-only (the option rows expose no arrow/Enter handling) while pointer-outside dismissal is covered. |
| Migrations / config implications | None persisted. Opt-in capture env vars: `DB_PRO_CAPTURE_QUERY_CONTEXT_PICKER` (`dense` seeds 24 long-named schemas; no `empty` mode exists — the caller always supplies one schema), `DB_PRO_CAPTURE_EXPLORER_FILTER_POPOVER`, `DB_PRO_CAPTURE_EXPLORER_FILTER_LIGHT`. `.compact(bool)` is additive API; the light palette change is visual only (no persisted preference key changes). |
| Out-of-scope changes | `none` |

Verification detail, matrix table and per-cell observations live in `VERIFICATION.md`.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review performed |
| Verdict | `ACCEPT WITH P2` for the self-verified scope only; **not** an independent approval |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 0 · inherited-and-fixed in scope: 0 / 0 / 7 |
| Findings | F1–F7 in `FINDINGS.md`; F5/F6 were found by the new narrow-width tests failing against the first implementation, F7 by the owner's light-region feedback, and the hover/panel collision by review of the first palette attempt |
| CI disposition | not executed |
| Next task(s) unblocked | Open a PR for independent review; re-inspect dense data views and the ER canvas under the new light chrome step; drive a deep-scroll traversal of a long schema list |

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

Gate đã chạy thật trên cây cuối (`457cd645`): fmt 0, `cargo check --workspace` 0, `cargo clippy
--workspace --all-targets -D warnings` 0 chẩn đoán, ui-architecture PASS, **release build locked 0**,
`cargo test --workspace` **2.272 pass / 0 fail / 65 ignored**; 65 test provider `#[ignore]` sau đó chạy
live và **pass 65/65** (pg 40 + object_identity 2 + mysql 12 + sqlserver 9 + ssh 1 + data_diff 1).
Matrix 17 ô được capture lại từ cây cuối và đọc từng ảnh; 1440/1920 bị host clamp còn 838 logical,
các ô `exit=-6` là crash TouchBar macOS **sau khi** ghi ảnh — không phải lỗi UI.

Về độ tách vùng: panel sáng đổi `#f9f9f9` → `#f6f6f6` (một bậc rõ so với canvas trắng), nhưng **cố ý
không dùng `#f1f1f1`** vì trùng `hover` — nếu dùng thì hover trên sidebar không nhìn thấy. Bậc thang
canvas → panel → hover → selected giờ có test khoá. Chữ tertiary/muted nâng lên `#6b6b6b` (4.93:1).

Còn thiếu: CI/review độc lập (chưa mở PR), chiều cao 900/1080 thật, soi lại dense data views và ER
canvas dưới bậc sáng mới, và cuộn sâu trong danh sách 24 schema. Vì vậy plan giữ `RUNTIME_VERIFY`,
chưa `COMPLETED`. Code đã push lên `main` theo owner override (code `457cd645`, docs tới
`ddc6a207`).
