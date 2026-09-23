# Checklist — query-workspace-zed-shell

## Plan

- [x] PLAN.md written
- [x] Branch `feature/query-workspace-zed-shell`

## Implement

- [x] Editor-first `draw_query` composition
- [x] Context chips replace ComboBoxes
- [x] Query status strip with Run/Stop
- [x] Output dock gated; open on result/error
- [x] Hide permanent diagnostics / params / inline completion
- [x] Find as floating overlay
- [x] Skip duplicate shell output panel on Query tab
- [x] Builder moved to More menu

## Verify

- [x] `cargo fmt` (touched files)
- [x] `cargo check -p db-pro-ui`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] `cargo test -p db-pro-ui --lib` (523 passed)
- [x] Runtime screenshots A–J recorded in `screenshots/` (1280x800 and 1440x900 light and dark)

## Review

- [x] Self-review P0/P1/P2
- [x] PR referencing PLAN / CHECKLIST / VERIFICATION
