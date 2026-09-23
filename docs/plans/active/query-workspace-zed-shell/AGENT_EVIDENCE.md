# Agent evidence — query-workspace-zed-shell

## 1. Task

| Field | Value |
|---|---|
| Feature | Query Workspace Zed Shell (UI05) |
| Branch | `feature/query-workspace-zed-shell` |
| State | In Progress |
| Goal | Editor-first Query chrome: chips, status Run, dock closed until result |

## 2. Source evidence

Exact SHAs pending commit. Touched presentation surfaces:

- `crates/ui/src/query_view.rs` — composition rewrite
- `crates/ui/src/query_editor_panel.rs` — fill height + editor rect
- `crates/ui/src/events_query.rs` — open dock on result/error/explain
- `crates/ui/src/app.rs` / `app_state.rs` — picker/params/dock/max state
- `crates/ui/src/editor/renderer.rs` — `SqlEditorResponse.rect` for overlays

## 3. Gates

| Command | Result |
|---|---|
| `cargo check -p db-pro-ui` | PASS |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS |
| `cargo test -p db-pro-ui --lib` | PASS · 523/0 |
| `cargo fmt` (touched files) | PASS |
| Runtime screenshots | pending |

## 4. Review

n/a (not yet in Review)

## 5. Risks / follow-ups

- Visual builder still collapsing-header (side split deferred)
- Full workspace fmt/check/clippy/test not re-run end-to-end this pass
- Capture harness still wired to other scenarios — Query shots pending

## 6. Tổng kết bằng tiếng Việt

Đã đảo composition Query: editor chiếm chiều cao khi chưa có result; ComboBox header → chip context; Run nằm status bar; output thành dock mở khi có kết quả/lỗi; find overlay; bỏ inline completion card. Còn runtime screenshot mới đóng được RUNTIME_VERIFY.
