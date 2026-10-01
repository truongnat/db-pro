# Agent evidence — Linux window controls

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native UI implementation |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `0f1b34dbb147ce130a6ae5899c2523f816adb0b1` |
| Branch / PR | `main` / no PR; changes are uncommitted |
| Scope interpretation | Replace Linux window-manager control glyphs with app-owned Lucide controls while preserving window actions and resize behavior. |
| Out of scope | Changing macOS/Windows decorations, launching the app, and the separate pending Explorer scroll change. |

## 2. Progress checkpoint

- Current HEAD: `0f1b34dbb147ce130a6ae5899c2523f816adb0b1`; implementation is in the uncommitted working tree.
- Completed acceptance rows: Linux minimize/maximize/restore/close controls; accessible labels and focus ring; close hover treatment; drag and double-click maximize; edge/corner resize; Windows and macOS keep native decorations.
- Remaining acceptance rows: owner runtime verification; full all-surface review under the existing plan.
- Findings / risks: P0 0, P1 0, P2 1 — actual Linux compositor behavior and visual appearance are not runtime-verified.
- Tests already run: `cargo test -p db-pro-ui --lib` → 938 passed / 0 failed / 0 ignored, exit 0; focused control-click and resize regressions also passed.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `0f1b34dbb147ce130a6ae5899c2523f816adb0b1` (baseline; implementation remains uncommitted) |
| Commit list | none |
| File / surface inventory | `crates/native-app/src/main.rs` — Linux frameless viewport; `crates/ui/src/app_lifecycle.rs` — always show controls in Linux Settings and draw resize hit zones; `crates/ui/src/shell_topbar_view.rs` — Lucide controls, drag/maximize, resize requests and headless test; `PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md` — Wave 16 record. |
| Acceptance mapping | OS frame diagnosis → baseline/source inspection; control actions and accessible names → `draw_window_controls`; buttons through resize overlays → `linux_window_buttons_emit_viewport_commands_through_resize_overlay`; resize boundary → `frameless_window_resize_handles_only_start_native_resize_at_the_edge`; runtime appearance and compositor behavior → pending owner verification. |
| Commands and counts | `cargo fmt --all -- --check` exit 0; `cargo check -p db-pro-ui -p db-pro-native` exit 0; `cargo test -p db-pro-ui --lib` 938/0/0 exit 0; `cargo clippy -p db-pro-ui --all-targets -- -D warnings` exit 0; `cargo build --release --locked -p db-pro-native` exit 0, binary not launched; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` 15 pass / 1 inherited warning / 0 fail; `git diff --check` exit 0. |
| CI run IDs / status | not run |
| Known limitations | No native runtime verification by request; no commit created. The owner should verify hover/focus appearance and drag/minimize/maximize/restore/close/edge resize on the target Linux compositor. |
| Migrations / config implications | Linux window decorations are disabled; application-level controls replace the OS titlebar controls. No persisted data changes. |
| Out-of-scope changes | Pending Explorer mouse-wheel fix remains in the working tree and is not included in this window-controls finding. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent review yet |
| Verdict | n/a |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 1 pending runtime verification; inherited: n/a |
| Findings | P2 runtime appearance/compositor behavior pending owner check |
| CI disposition | not run |
| Next task(s) unblocked | Owner runtime verification |

## 5. Research / audit handoff

- Source date: 2026-10-01
- Source references: `crates/native-app/src/main.rs`, `crates/ui/src/shell_topbar_view.rs`, egui 0.29 `ViewportBuilder` and `ViewportCommand` API in the local Cargo registry.
- Factual findings: the baseline configured the native viewport without changing OS decorations; egui provides decoration, drag, resize, minimize, maximize and close commands.
- Inference: the unattractive Linux control icons came from the window manager because DB Pro did not draw these controls.
- Decision / recommendation: use custom Lucide controls on Linux only; retain native controls on macOS and Windows.
- Unresolved questions: how the controls and native resize requests behave on the owner's Linux compositor at runtime.
- Downstream tasks activated: owner runtime verification under Wave 16.

## 6. Tổng kết (Vietnamese summary)

Đã thêm bộ nút cửa sổ Lucide riêng cho Linux, giữ thao tác kéo, phóng to/khôi phục và resize; test UI, Clippy và release build đều qua. Chưa mở app theo yêu cầu, nên giao diện và hành vi trên compositor Linux vẫn cần chủ ứng dụng xác minh.
