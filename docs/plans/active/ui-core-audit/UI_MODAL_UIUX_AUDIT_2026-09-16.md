# UI Core Modal & UX Deep Audit

**Sha**: bae4765e chore: ignore local agent tooling state directories (.omh, .hermes, .kilo, .jules)
**Crate audited**: `crates/ui` (db-pro-ui)
**Date**: 2026-09-16

## Tổng quan

Modal component trong DB Pro được thực hiện qua cơ chế egui inset + overlay overlay tự vẽ, không phải window modal native. Điều này tạo ra nhiều vấn đề UX và accessibility so với tiêu chí migration plan (native rendering, accessibility, multi-window, lifecycle).

## Modal mechanism

### Taint modal thực tế

Modal trong DB Pro không phải window modal native. Cơ chế:

- `Dialog` component tạo inset area thông qua `show_inset` egui, không phải fullscreen modal
- Overlay faded tự vẽ qua `faded_overlay` trong `overlay.rs`
- Hiệu ứng translate animation cho dialog position
- Toast system độc lập với modal, render riêng

Trade-off chính: inset không capture toàn bộ screen, không_disable interaction bên ngoài, không focus trap, không backdrop native.

### Kiến trúc overlay

1. `Dialog` proxy (`dialog.rs`): wrapper struct với title, description, width, height, content closure, id_salt. `show` proxy tới `show_dialog_inset` callback.

2. `faded_overlay` tự compute screen_rect, paint hiệu ứng, và dùng `small_translate` animate position dialog.

3. `ToastManager` quản lý toast đa stack, 6 quadrant, animation_AUTO.

4. `show_context_menu` dùng egui::Menu native, `show_popup_overlay` builder pattern với focus trap cơ bản.

### Các modal state trong app

Từ `app.rs` và `app_state.rs`:

- `welcome_open` — welcome screen
- `connection_dialog_open` — connection dialog
- `delete_confirmation_id` — delete confirmation
- `folder_delete_confirmation` — folder delete confirmation
- `insert_row_open` — insert row dialog
- `palette_mode` — quick open / command palette
- `agent_open` — agent panel (slide-in sidebar)
- `query_tools_open` — query tools panel
- `completion_open` — completion popup
- `snippets_open` — snippets popup
- `visual_query_builder_open` — visual query builder
- `agent_settings_open` — agent settings
- `save_as_open` — save as dialog
- `record_inspector_open` — record inspector
- `data_edit_open` (implied) — data edit
- `export_overwrite_pending` — export overwrite confirmation
- `pending_destructive_run` — destructive run confirmation
- `ddl_execute_confirmation` — DDL execute confirmation
- `conflict_dialog_open` — conflict dialog
- `pending_changes_open` — pending changes
- `staged_apply_request` — staged apply
- `query_txn_bar_open` — transaction bar
- `agent_pending_prompt` — agent pending prompt

Nhiều state boolean độc lập — có thể multiple dialog open cùng lúc,导致 overlap.

## UX findings

### Strengths

1. **Gallery confirmations pattern**: Các dialog (connection, delete, folder delete, insert row) đều managed trong 1 điểm中央 (app.rs:704-718) — dễ follow và maintain.

2. **Keyboard navigation cho palette**: ↑↓, Enter, Esc rõ ràng (palette_view.rs:932). Keyboard accessibility cơ bản có cho palette.

3. **Toast UX tốt**: Multi-stack, 6 position, animation, dismiss/clear.

4. **Menu native**: Sử dụng egui::Menu để context menu, focus trap cơ bản thông qua `should_close_on_click_outside`.

5. **Destructive UX confirmation**: Có confirm cho destructive run, DDL execute, export overwrite — bảo vệ user khỏi action không cập nhật.

6. **Theme toggle**: Light/dark mode toggle có, reduce motion support.

### Weaknesses

1. **Modal không capture focus**: Inset không trap keyboard focus. Người dùng có thể tab ra ngoài dialog. Không có keyboard modal behavior chuẩn.

2. **Không có backdrop native**: Widget phía sau dialog vẫn visible và potential interaction. Overlay tự vẽ faded nhưng không disable interaction vùng phía sau.

3. **Accessibility tree thiếu**: Result grid không expose grid role với cell coordinates cho screen reader. Tree keyboard navigation chỉ palette, không cho sidebar tree.

4. **Dialog không trap Esc modal**: Esc có thể close palette nhưng không nhất thiết close mọi dialog. Hành vi close không consistent across dialogs.

5. **Multi-dialog constraint**: Không có constraint rằng chỉ 1 dialog mở tại 1 thời điểm. Nhiều dialog overlay có thể overlap, gây confuse.

6. **Animation position dialog**: `small_translate` animation dùng timer-based decay — tốt cho perf nhưng position dialog không tùy biến theo screen size.

7. **Dialog width fixed**: Nhiều dialog có width fixed (ví dụ 540.0 cho shortcuts dialog) — không responsive với screen nhỏ.

8. **Overlay effect đơn giản**: faded_overlay paint hiệu ứng đơn giản, không có animation fade-in/out cho overlay — dialog xuất hiện đột ngột.

9. **No modal z-order management**: Không có z-order quản lý cho multiple overlay — dialog overlay có thể overlap không rõ ràng.

10. **No modal dismissal consistency**: Mỗi dialog có cơ chế dismiss riêng (click outside, button close, Esc) — không consistent.

## Accessibility gaps

1. **Grid accessibility**: Grid không expose role="grid" với row/cell role, không có aria-label cho cell content. Screen reader không thể navigate grid.

2. **Tree accessibility**: Sidebar tree không có keyboard navigation và aria tree role. Chỉ palette có keyboard nav.

3. **Dialog accessibility**: Dialog không announce title cho screen reader tự động, không focus trap, không modal ARIA attributes.

4. **Color contrast**: Theme tokens có contrast ratio nhưng chưa tính cho nghiệp vụ cụ thể (tùy state light/dark). Badge/PK/FK vertical position lệch so với text center.

5. **Focus indicator**: Không có visible focus indicator rõ ràng cho dialog focus — người dùng keyboard khó track focus.

## Performance risk

1. **Overlay paint trong main frame**: faded_overlay paint cùng frame với main render — không async/offscreen. Nếu overlay phức tạp, có thể ảnh hưởng frame budget.

2. **Animation timer-based**: Decay animation dùng timer-based, không Tween stack — tốt cho perf. Nhưng animation state persist trong context, cần invalidate khi theme change.

3. **Grid render**: Virtualization có (GridProjectionCache), nhưng alignment logic phức tạp trong draw_grid_body. Layout compute nhiều pass.

4. **Layout compute inline**: Nhiều inline constraint compute (row height, column width) trong render pass — không cache ngoài render.

5. **Thưa toast render**: Toast render ctx mỗi frame khi có toasts, request repaint 50ms — acceptable nhưng có thể optimize.

6. **No GPU overlay layer**: Overlay không tách layer GPU — paint trực tiếp trong main frame. Nếu có nhiều overlay stacked, có thể gây perf issue.

## Recommendation

### P0 (nên làm trước)

1. **Focus trap cho dialog**: Thêm focus trap cho dialog (capture keyboard focus trong dialog, không cho phép tab ra ngoài). Đây là modal behavior cơ bản.

2. **Backdrop interaction disable**: Overlay phải disable interaction vùng phía sau dialog. Hiện tại chỉ faded visual, không disable interaction.

3. **Consistent close behavior**: Esc close mọi dialog, consistent modal behavior. Click outside close dialog (nếu phù hợp).

4. **Single dialog constraint**: Constraint 1 dialog mở tại 1 thời điểm, hoặc stackable với z-order rõ ràng. Tránh multiple dialog overlap.

### P1 (nên làm sau)

5. **Accessibility tree**: Thêm accessibility node cho grid (role grid với cell coordinates), tree, dialog. Đảm bảo screen reader có thể navigate.

6. **Dialog width responsive**: Responsive width cho dialog, sử dụng available width thay fixed width.

7. **Modal z-order management**: Quản lý z-order cho multiple overlay, đảm bảo dialog hiển thị đúng层级.

8. **Animation overlay fade**: Thêm animation fade-in/out cho overlay, dialog xuất hiện mượt mà.

9. **Focus indicator**: Visible focus indicator rõ ràng cho dialog focus.

### P2 (polish)

10. **Performance optimization**: Cache layout compute ngoài render, dùng GPU-friendly paint cho overlay.

11. **Toast animation polish**: Animation toast mượt mà, position toast hợp lý.

12. **Dialog content adaptive**: Content dialog adaptive với width/height, scroll khi nécessaire.

## So sánh với migration plan quality gates

- **Native rendering**: Modal không native, dùng inset + overlay self-paint. Không đạt gate.
- **Accessibility**: Accessibility tree thiếu, không expose grid/tree/dialog cho screen reader. Không đạt gate.
- **Performance**: Overlay paint trong main frame, animation timer-based — acceptable nhưng chưa tối ưu.
- **Animation**: Animation có nhưng đơn giản, không có fade-in/out cho overlay.
- **Multi-window**: Không áp dụng cho modal, nhưng multiple dialog constraint chưa có.
- **Lifecycle**: Modal state management 분산 trong app.rs, không có lifecycle manager tập trung.

## References

- Migration plan: `docs/10-egui-native-migration-plan.md`
- Dialog component: `crates/ui/src/components/dialog.rs`
- Overlay component: `crates/ui/src/components/overlay.rs`
- Modal/gallery overlays: `crates/ui/src/component_gallery_view.rs` (gallery_state, shortcuts_dialog_open, show_dialog calls)
- Component gallery feedback: `crates/ui/src/component_gallery_feedback.rs` (show_dialog macro usage, confirmation dialog drawing)
- Component gallery surfaces: `crates/ui/src/component_gallery_surfaces.rs` (dialog surfaces in gallery)
- Component gallery inputs: `crates/ui/src/component_gallery_inputs.rs`
- Component gallery agent: `crates/ui/src/component_gallery_agent.rs`
- App render loop: `crates/ui/src/app.rs:704-720`
- Palette keyboard: `crates/ui/src/palette_view.rs:932`
- Grid render: `crates/ui/src/result_grid_view.rs`
- Grid projection cache: `crates/ui/src/grid_layout.rs`
- Result grid cell: `crates/ui/src/result_grid_cell.rs`
- Result grid selection: `crates/ui/src/result_grid_selection.rs`
- Result grid clipboard: `crates/ui/src/result_grid_clipboard.rs`
- Result grid edit: `crates/ui/src/result_grid_edit.rs`
- Result grid header: `crates/ui/src/result_grid_header.rs`
- Table metadata view: `crates/ui/src/table_metadata_view.rs`
- Query output view: `crates/ui/src/query_output_view.rs`
- Query runner: `crates/ui/src/query_runner.rs`
- Query execution: `crates/ui/src/query_execution.rs`
- Query session: `crates/ui/src/query_session.rs`
- Editor renderer: `crates/ui/src/editor/renderer.rs`
- Editor mod: `crates/ui/src/editor/mod.rs`
- Query SQL format: `crates/ui/src/query/sql_format.rs`
- Query SQL parameters: `crates/ui/src/query/sql_parameters.rs`
- Query visual builder: `crates/ui/src/query/visual_builder.rs`
- Query document: `crates/ui/src/query/query_document.rs`
- Query mod: `crates/ui/src/query/mod.rs`
- Theme: `crates/ui/src/theme.rs`
- Tokens: `crates/ui/src/tokens.rs`
- Component library: `crates/ui/src/components/library.rs`
- Button: `crates/ui/src/components/button.rs`
- Input: `crates/ui/src/components/input.rs`
- Badge: `crates/ui/src/components/badge.rs`
- Card: `crates/ui/src/components/card.rs`
- Table: `crates/ui/src/components/table.rs`
- Tabs: `crates/ui/src/components/tabs.rs`
- Chrome: `crates/ui/src/components/chrome.rs`
- Alert: `crates/ui/src/components/alert.rs`
- Tree: `crates/ui/src/components/tree.rs`
- Code: `crates/ui/src/components/code.rs`
- Progress: `crates/ui/src/components/progress.rs`
- Spacer: `crates/ui/src/components/spacer.rs`
- Toast: `crates/ui/src/components/toast.rs`
- Divider: `crates/ui/src/components/divider.rs`
- Form field: `crates/ui/src/components/form_field.rs`
- Dropdown: `crates/ui/src/components/dropdown.rs`
- Slider: `crates/ui/src/components/slider.rs`
- Search: `crates/ui/src/components/search.rs`
- Toolbar: `crates/ui/src/components/toolbar.rs`
- Animation: `crates/ui/src/components/animation.rs`
- Sidebar view: `crates/ui/src/sidebar_view.rs`
- Query view: `crates/ui/src/query_view.rs`
- Component gallery view: `crates/ui/src/component_gallery_view.rs`
- Component gallery surfaces: `crates/ui/src/component_gallery_surfaces.rs`
- Component gallery feedback: `crates/ui/src/component_gallery_feedback.rs`
- Component gallery inputs: `crates/ui/src/component_gallery_inputs.rs`
- Component gallery overlays: `crates/ui/src/component_gallery_overlays.rs`
- Component gallery agent: `crates/ui/src/component_gallery_agent.rs`
- Explorer tree: `crates/ui/src/explorer_tree.rs`
- Explorer view: `crates/ui/src/explorer_view.rs`
- Explorer connections: `crates/ui/src/explorer_connections.rs`
- Explorer details: `crates/ui/src/explorer_details.rs`
- Explorer folders: `crates/ui/src/explorer_folders.rs`
- Navigation view: `crates/ui/src/navigation_view.rs`
- Connection view: `crates/ui/src/connection_view.rs`
- Settings view: `crates/ui/src/settings_view.rs`
- Settings model: `crates/ui/src/settings_model.rs`
- Problems view: `crates/ui/src/problems_view.rs`
- Activity bar view: `crates/ui/src/activity_bar_view.rs`
- Schema object view: `crates/ui/src/schema_object_view.rs`
- Schema workbench: `crates/ui/src/schema_workbench.rs`
- Schema workbench form: `crates/ui/src/schema_workbench_form.rs`
- Schema compare: `crates/ui/src/schema_compare.rs`
- Table DDL view: `crates/ui/src/table_ddl_view.rs`
- Table editor view: `crates/ui/src/table_editor_view.rs`
- Diagram view: `crates/ui/src/diagram_view.rs`
- Visual query builder view: `crates/ui/src/visual_query_builder_view.rs`
- Files activity view: `crates/ui/src/files_activity_view.rs`
- Git workspace: `crates/ui/src/git_workspace.rs`
- IDE workspace: `crates/ui/src/ide_workspace.rs`
- Workspace view: `crates/ui/src/workspace_view.rs`
- Workspace actions: `crates/ui/src/workspace_actions.rs`
- Workspace session: `crates/ui/src/workspace_session.rs`
- Tasks view: `crates/ui/src/tasks_view.rs`
- Palette view: `crates/ui/src/palette_view.rs`
- Search service: `crates/ui/src/search_service.rs`
- Capability lookup: `crates/ui/src/capability_lookup.rs`
- Agent view: `crates/ui/src/agent_view.rs`
- Agent state: `crates/ui/src/agent_state.rs`
- Agent workflow state: `crates/ui/src/agent_workflow_state.rs`
- Events: `crates/ui/src/events.rs`
- Events query: `crates/ui/src/events_query.rs`
- Events query dispatch: `crates/ui/src/events_query_dispatch.rs`
- Agent message frame: `crates/ui/src/agent_message_frame.rs`
- Change set: `crates/ui/src/change_set.rs`
- Cell inspector: `crates/ui/src/cell_inspector.rs`
- Agent provider: `crates/ui/src/agent_provider.rs`
- Agent context: `crates/ui/src/agent_context.rs`
- Offline agent provider: `crates/ui/src/offline_agent_provider.rs`
- Task bridge: `crates/ui/src/task_bridge.rs`
- Grid projection cache: `crates/ui/src/grid_projection_cache.rs`
- Grid projection key: `crates/ui/src/grid_projection_key.rs`
- Grid layout: `crates/ui/src/grid_layout.rs`
- Result grid view: `crates/ui/src/result_grid_view.rs`
- Result grid cell: `crates/ui/src/result_grid_cell.rs`
- Result grid selection: `crates/ui/src/result_grid_selection.rs`
- Result grid clipboard: `crates/ui/src/result_grid_clipboard.rs`
- Result grid edit: `crates/ui/src/result_grid_edit.rs`
- Result grid header: `crates/ui/src/result_grid_header.rs`
- Table metadata view: `crates/ui/src/table_metadata_view.rs`
- Query output view: `crates/ui/src/query_output_view.rs`
- Query runner: `crates/ui/src/query_runner.rs`
- Query execution: `crates/ui/src/query_execution.rs`
- Query session: `crates/ui/src/query_session.rs`
- Query SQL format: `crates/ui/src/query/sql_format.rs`
- Query SQL parameters: `crates/ui/src/query/sql_parameters.rs`
- Query visual builder: `crates/ui/src/query/visual_builder.rs`
- Query document: `crates/ui/src/query/query_document.rs`
- Query mod: `crates/ui/src/query/mod.rs`
- Editor renderer: `crates/ui/src/editor/renderer.rs`
- Editor mod: `crates/ui/src/editor/mod.rs`
- App state: `crates/ui/src/app_state.rs`
- App types: `crates/ui/src/app_types.rs`
- App: `crates/ui/src/app.rs`
