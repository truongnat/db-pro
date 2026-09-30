# Verification

## Source evidence

- Baseline: `bd787db1e5d75862ecd08c86539e910d4e651ebd` (main).
- Branch: `feature/theme-core-token` (created successfully this session — this
  corrects the previous record claiming `.git` was read-only and the branch
  creation was blocked; `.git` is writable in this environment).
- Source slice commit: `8a188e84a3dc7b43663f952f5edc2be6024e3dac`
  (`feat(ui): build core token contract (primitive → semantic → component)`),
  20 files, +1211/−343.

## Commands (executed this session, results recorded verbatim)

| Command | Result |
|---|---|
| `git status --short --branch` / `git rev-parse HEAD` | PASS — branch `feature/theme-core-token`, HEAD `bd787db1…` before the slice commit |
| `cargo fmt --all -- --check` | PASS (exit 0), run after the final edit |
| `CARGO_TARGET_DIR=/tmp/db-pro-target cargo check -p db-pro-ui` | PASS (exit 0, 0 warnings) |
| `CARGO_TARGET_DIR=/tmp/db-pro-target cargo check --workspace` | PASS (exit 0) — extra guard: no consumer outside `db-pro-ui` uses the moved tokens |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | PASS — 13 ✓ / 3 ⚠ / 0 ✗, exit 0; warnings explained in FINDINGS |
| `git diff --check` | PASS (exit 0) |
| `CARGO_TARGET_DIR=/tmp/db-pro-target cargo test -p db-pro-ui --lib` | PASS — 927 passed / 0 failed / 0 ignored, exit 0 (17s) |

Full workspace test suite was **not** run (not requested by the task).

## Static contract checks

| Check | Result |
|---|---|
| No new raw color/radius/spacing/stroke in component rendering | PASS — component diffs only replace literals with named roles/consts (`STROKE_THICK`, `border.focus`, `theme.semantic.status.*`); alert's two opacity literals moved to `semantic::STATUS_*` |
| Every semantic role has light and dark treatment | PASS — `every_role_group_has_light_and_dark_treatment` |
| State precedence explicit (`disabled → loading → active → focus → hover → default`) | PASS — `component::STATE_PRECEDENCE` + `button::shows_loading` + `input::resolve_chrome`, each pinned by tests |
| Existing button dimensions and theme values unchanged | PASS — `dimensions_preserve_the_existing_rendered_values`, `light_tokens_follow_warm_minimalism_surface_contract`, `dark_tokens_follow_database_workstation_surface_contract`, `flat_facade_fields_are_built_from_the_semantic_roles` |
| All imported symbols exist; no dead compatibility alias introduced | PASS — `cargo check --workspace` clean; the `FieldChromeState` re-export in `input/layout.rs` is consumed by password/text/textarea/search |

## Runtime evidence

**NOT VERIFIED** — no UI runtime session (screenshot/screen recording at
1280×800, 1440×900, 1920×1080) was captured in this slice. Source inspection
and unit tests are not runtime evidence. The two precedence fixes
(disabled+loading button, disabled input chrome) are observable in the
component gallery and must be checked in a runtime pass before this feature
moves to `RUNTIME_VERIFY`/`COMPLETED`.

## Tổng kết bằng tiếng Việt

Đã xác minh lại toàn bộ bằng chứng cho slice hợp đồng token ba lớp: `.git` có
thể ghi được nên branch `feature/theme-core-token` đã được tạo (sửa lại ghi chú
sai của lần trước), mã nguồn được commit tại `8a188e84`. Bốn gate theo yêu cầu
đều PASS (fmt, check `db-pro-ui`, clean-code scan 13✓/3⚠/0✗, `git diff --check`),
check thêm toàn workspace PASS, và 927 test của package `db-pro-ui` chạy xanh
100%. Các kiểm tra tĩnh (không thêm giá trị thô, đủ role light/dark, thứ tự
state precedence, kích thước button giữ nguyên, không import chết) đều đạt.
Bằng chứng runtime giao diện vẫn ở trạng thái **NOT VERIFIED** — cần một phiên
chụp màn hình gallery ở 3 độ phân giải trước khi đưa feature sang `RUNTIME_VERIFY`.
