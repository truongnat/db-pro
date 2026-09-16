# Verification

## State

`IMPLEMENTING` — slices 1–3 source work is done (slice 4 partially verified by inspection); runtime
evidence and ER dense labels are pending.

## Implementation slice 1

- `crates/ui/src/workspace_view.rs`: bounded tab title width, measured ellipsis truncation, stable tab
  sizing constants and full-title hover tooltip.
- `crates/ui/src/query_view.rs`: query file path and title labels now truncate instead of extending the
  header indefinitely, with full-content hover tooltips.
- `crates/ui/src/sidebar_view.rs`: active connection/workspace label now truncates and exposes the full
  name on hover.
- `crates/ui/src/components/dialog.rs`: dialogs and sheets clamp their width to the viewport; titles
  truncate with a tooltip while the close action remains in a reserved layout slot.
- `crates/ui/src/components/select.rs`: dropdowns clamp their horizontal position/width to the viewport
  and long options truncate inside the menu.

## Implementation slices 2–4 (this session)

- `crates/ui/src/query_view.rs`: header connection/schema selectors now use fixed widths
  (`HEADER_CONN_COMBO_WIDTH` 170 px, `HEADER_SCHEMA_COMBO_WIDTH` 130 px) with char-elided labels
  (`elide_chars`, budget 28 chars) and full-name hover tooltips, so long connection/schema names cannot
  push the Run/Builder/overflow actions off-screen at 1280x800 or below.
- `crates/ui/src/query_helpers.rs`: new pure `elide_chars(text, max_chars)` helper, char-count based so
  Vietnamese/Japanese names elide the same as ASCII; unit tests cover short, long, Vietnamese and
  Japanese inputs.
- `crates/ui/src/components/dialog.rs`: dialog body is rendered inside a vertical `ScrollArea` capped
  at `screen.height() - 120`, so long errors or large-font translated content scroll instead of pushing
  the footer actions off-screen; short content still hugs. Painter params grouped into
  `DialogCardPaint` to keep the function signature within limits.
- Sidebar (slice 2) and grid/output scroll ownership (slice 4) verified by inspection as already
  satisfied — see `FINDINGS.md`; no code change was needed.

## Commands

- `cargo check -p db-pro-ui` — pass after gallery module split/refactor.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings` — pass.
- `cargo test -p db-pro-ui` — **483 passed / 0 failed / 0 ignored**, exit 0.
- `cargo fmt --all` — pass.
- `git --no-pager diff --check` — pass.
- `gh issue view 300 --comments` — issue requirements retrieved.
- This session:
  - `cargo fmt --all` — pass; `cargo fmt --all -- --check` — pass.
  - `cargo check -p db-pro-ui` — pass.
  - `cargo test -p db-pro-ui` — **486 passed / 0 failed / 0 ignored**, exit 0 (includes the 3 new
    `elide_chars` tests).
  - `cargo clippy -p db-pro-ui --all-targets -- -D warnings` — pass (after moving the test module to
    the end of `query_helpers.rs` and grouping `paint_dialog_card` params).
  - `cargo clippy --workspace --all-targets -- -D warnings` — pass.
  - `cargo test --workspace` — pass; all workspace test binaries completed successfully.
  - `cargo build --release --locked -p db-pro-native` — pass.
  - `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` — pass: 0 blocking
    failures, 1 pre-existing warning (structural debt).

## Runtime evidence

Still blocked in this environment. `target/release/db-pro-native` was launched this session and the
process appeared in the application list, but screen capture is denied (`Screen Recording is denied
for ZCode`) and the app exposes no accessibility tree, so no screenshot could be taken. No resize
traversal is claimed. UI14 cannot be marked complete from source inspection alone; deliberate
worst-case traversal remains required at 1024x700, 1280x800, 1440x900, 1920x1080 and maximized.

## Tổng kết bằng tiếng Việt

Tiếp tục issue #300 (UI14): selector connection/schema ở header query giờ có chiều rộng cố định kèm
elide và tooltip, không thể đẩy nút Run/Builder ra ngoài màn hình; dialog giới hạn chiều cao và
cuộn được khi nội dung dài, nên các nút hành động luôn với tới được. Sidebar và scroll ownership của
grid/output được xác nhận đã đạt. Toàn bộ gate pass (486 test UI, clippy workspace, build release,
clean-code scan). Runtime screenshot ở các kích thước yêu cầu vẫn bị chặn do không có quyền Screen
Recording; ER dense label và chính sách wrap/truncate cho cell dài còn lại cho lát sau.
