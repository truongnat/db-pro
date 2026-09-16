# Runtime Verification Plan — Modal & Overlay

**Associated audit**: `UI_MODAL_UIUX_AUDIT_2026-09-16.md`
**Sha**: bae4765e
**Date**: 2026-09-16
**Status**: Draft — không thể thực thi trong môi trường CLI này (build db-pro-native thất bại). Plan ghi nhận constraint.

## Mục tiêu

Thu thập runtime evidence cho claims trong UI_MODAL_UIUX_AUDIT:

1. Modal không capture focus — test keyboard focus không trap trong dialog.
2. Backdrop interaction không disable — test click outside dialog vẫn trigger widget phía sau.
3. Esc không close mọi dialog — test Esc trong dialog không close.
4. Multiple dialog overlap — test mở 2 dialog cùng lúc.
5. Animation/transition — đo frame budget khi overlay hiển thị.

## Environment constraint

- Build `db-pro-native` thất bại trong môi trường CLI này: lỗi bindgen sqlite, thiếu GUI context, không có display.
- Không thể run db-pro-native để đo réalité.
- Plan ghi nhận: cần machine có GUI (macOS desktop, Linux X11/Wayland) và fix build dependency (bindgen, cmake, libsqlite3-dev, pkg-config, display server).

### Alternative nếu build không khả thi ngay

1. **Unit test component behavior**: viết test cho Dialog/Sheet logic trong crate ui — nhưng egui widget test cần egui test utils, không dễ.
2. **Mock overlay behavior**: viết integration test cho overlay state machine (ToastManager, dialog state) — test logic không GUI.
3. **Visual acceptance**: chụp screenshot trên machine có GUI, hoặc dùng CI GPU runner.

## Test cases (nếu chạy được)

### TC-1: Focus trap test

Mục tiêu: verify dialog không capture focus.

Cách làm:
- Mở dialog (ví dụ destructive run dialog).
- Focus vào input/nút trong dialog.
- Nhấn Tab — kiểm tra focus có ra ngoài dialog không.
- Nếu focus ra ngoài → confirm weakness: không focus trap.

Evidence thu thập: ghi log focus widget ID khi Tab.

### TC-2: Backdrop interaction test

Mục tiêu: verify click outside dialog vẫn trigger widget phía sau.

Cách làm:
- Mở dialog.
- Click vào vùng phía sau dialog (ví dụ button trong workspace).
- Nếu button phía sau触发 → confirm weakness: backdrop không disable interaction.

Evidence: log event khi click.

### TC-3: Esc close behavior test

Mục tiêu: verify Esc không close mọi dialog.

Cách làm:
- Mở dialog (ví dụ shortcuts dialog).
- Nhấn Esc.
- Kiểm tra dialog có close không.
- Nếu không close → confirm weakness: Esc không trap.

Test trên palette: palette dùng Esc close — verify palette close behavior.

### TC-4: Multiple dialog overlap test

Mục tiêu: verify multiple dialog có thể overlap.

Cách làm:
- Mở dialog A (ví dụ shortcuts dialog).
- Mở dialog B (ví dụ drop table dialog) trong gallery.
- Quan sát overlay — có overlap không.
- Kiểm tra z-order: dialog nào trên?

### TC-5: Frame budget measurement

Mục tiêu: đo overhead overlay.

Cách làm:
- Instrumentation: thêm `Instant::now()` quanh render path có overlay.
- Đo frame time khi có overlay vs không overlay.
- Nếu overlay significant overhead → perf risk.

Cần thêm instrumentation code vào app.rs/overlay.rs trước khi đo.

## Runtime instrumentation cần thêm (nếu làm)

1. **Dialog state machine**:追踪 dialog open/close events, времени mở/đóng.
2. **Focus tracking**: ghi log focus widget khi dialog mở.
3. **Interaction blocking test**: instrumentation click event, check có blocked khi dialog open không.
4. **Frame timing**: ghi log frame time khi overlay active.

## Coverage modal state

Test các modal state trong app_state.rs:

- `connection_dialog_open`
- `delete_confirmation_id`
- `folder_delete_confirmation`
- `insert_row_open`
- `palette_mode`
- `agent_open` (sidebar, không phải dialog)
- `query_tools_open` (panel)
- `completion_open` (popup)
- `snippets_open` (popup)
- `visual_query_builder_open`
- `agent_settings_open`
- `save_as_open`
- `record_inspector_open`
- `export_overwrite_pending`
- `pending_destructive_run`
- `ddl_execute_confirmation`
- `conflict_dialog_open`
- `pending_changes_open`
- `staged_apply_request`
- `query_txn_bar_open`
- `agent_pending_prompt`

Mỗi modal state cần test focus trap, backdrop disable, Esc behavior.

## Verification acceptance criteria

1. **Focus trap**: Dialog capture focus, không cho Tab ra ngoài.
2. **Backdrop disable**: Click outside dialog không trigger widget phía sau.
3. **Esc close**: Esc close mọi dialog (hoặc popup phù hợp).
4. **Single dialog**: Chỉ 1 dialog mở, hoặc z-order rõ ràng.
5. **Animation**: Overlay fade-in/out, dialog xuất hiện mượt.
6. **Frame budget**: Overlay không làm tăng frame time đáng kể.

## References

- Audit: `docs/plans/active/ui-core-audit/UI_MODAL_UIUX_AUDIT_2026-09-16.md`
- Overlay code: `crates/ui/src/components/overlay.rs`
- Dialog code: `crates/ui/src/components/dialog.rs`
- Gallery overlays: `crates/ui/src/component_gallery_overlays.rs`
- Gallery feedback: `crates/ui/src/component_gallery_feedback.rs`
- Query dialogs: `crates/ui/src/query_dialogs_view.rs`
- App render loop: `crates/ui/src/app.rs:704-720`
- Migration plan quality gates: `docs/10-egui-native-migration-plan.md`