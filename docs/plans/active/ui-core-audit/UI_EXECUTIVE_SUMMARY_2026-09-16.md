# UI Core Audit — Executive Summary

**SHA**: bae4765e
**Crate audited**: `crates/ui` (db-pro-ui)
**Date**: 2026-09-16
**Audit lead**: agent (deterministic)
**Date range**: 2026-09-16 (single session)

## Audit scope

- Core component UI quality (migration plan `docs/10-egui-native-migration-plan.md` quality gates)
- UI/UX deep audit for modal, overlay, toast, dialog, palette, grid, tree, input, button, card, badge, tabs, alert, code, progress, animation
- Layout computation audit (distance, padding, margin, width, height)
- Runtime verification plan (modal-specific)
- Visual acceptance plan

## Reports generated

```
docs/plans/active/ui-core-audit/
├── UI_CORE_QUALITY_AUDIT_2026-09-16.md            # baseline quality audit (164 lines)
├── UI_CORE_UIUX_PERF_DEEP_AUDIT_2026-09-16.md    # UI/UX + performance deep audit (211 lines)
├── UI_LAYOUT_COMPUTATION_AUDIT_2026-09-16.md     # layout computation audit (234 lines)
├── UI_MODAL_UIUX_AUDIT_2026-09-16.md             # modal & UX deep audit (296 lines)
├── UI_MODAL_RUNTIME_VERIFICATION_PLAN_2026-09-16.md  # runtime verification plan (modal) (92 lines)
├── UI_RUNTIME_VERIFICATION_PLAN_2026-09-16.md    # runtime verification plan (general) (102 lines)
└── UI_VISUAL_ACCEPTANCE_PLAN_2026-09-16.md       # visual acceptance evidence plan (92 lines)
```

## Key findings (total)

### P0 (critical)

1. **Modal không focus trap** — inset dialog không capture keyboard focus. Tab có thể ra ngoài dialog.
2. **Modal không backdrop interaction disable** — overlay chỉ faded visual, không disable click-through.
3. **Modal Esc behavior không consistent** — Esc không close mọi dialog. Palette có Esc close, dialog không.
4. **Multi-dialog không constraint** — nhiều dialog overlay có thể overlap, không z-order management.

### P1 (high)

5. **Accessibility tree thiếu** — grid không expose role grid/cell, tree không keyboard nav, dialog không announce title.
6. **Dialog width fixed** — không responsive với screen nhỏ.
7. **Modal z-order management thiếu** — multiple overlay overlap không rõ ràng.
8. **Animation fade-in/out thiếu** — overlay dialog xuất hiện đột ngột.
9. **Focus indicator không rõ ràng** — dialog focus không visible indicator.

### P2 (polish)

10. **Performance: overlay paint trong main frame** — không async/offscreen, có thể ảnh hưởng frame budget với nhiều overlay.
11. **Layout compute inline trong render pass** — không cache layout compute ngoài render.
12. **Toast render mỗi frame** — request repaint 50ms, acceptable nhưng có thể optimize.
13. **Row height inconsistency** — result grid 28.0 vs table component 44.0 (chênh 16px).
14. **Input vertical padding inconsistency** — 4px (Input/Password), 5px (Search), 6px (Textarea).
15. **Spacing token violation** — `add_space(3.0)`, `add_space(6.0)`, `add_space(10.0)` xuất hiện nhiều lugar tidak terdefinisi dalam token system.
16. **Grid header padding (8,4) khác data cell (12,0)**.
17. **Tree indent inconsistent** — 14.0/level + 8.0 base vs 10.0/level.
18. **Resize divider hit target 6.0px** — hẹp (desktop HIG khuyến nghị 8-10px).
19. **PK/FK badge vertical position lệch** so với text center.

## Strengths

1. **Token ownership** — DbProTheme tập trung token, widget đọc semantic tokens từ theme.
2. **Virtualization** — GridProjectionCache cho result grid, hiệu quả cho dataset lớn.
3. **Editor native** — SQL editor render native, syntax highlighting, overlay terminal/minimap.
4. **Destructive UX confirmation** — confirm cho destructive run, DDL execute, export overwrite.
5. **Keyboard navigation cho palette** — ↑↓, Enter, Esc rõ ràng.
6. **Toast UX tốt** — multi-stack, 6 position, animation, dismiss/clear.
7. **Form validation** — real-time validation, touched/dirty tracking.
8. **Theme toggle** — light/dark mode, reduce motion support.
9. **Gallery confirmations pattern** — modal state tập trung management.

## Component coverage

Đã audit:

- **Modal/Overlay**: Dialog, Sheet, Toast, Tooltip, Popover, DropdownMenu, context menu, faded_overlay, small_translate animation.
- **Input**: Input, PasswordInput, Textarea, SearchInput, FormField.
- **Button**: Button variants/sizes/loading/disabled, icon button.
- **Feedback**: Alert, Toast, Progress, Spinner, Skeleton, EmptyState.
- **Data display**: ResultGrid (view, cell, selection, clipboard, edit, header), Table, Badge, Card, Code, Tree.
- **Layout**: Spacer, Divider, Toolbar, Tabs, Chrome, Panel.
- **Navigation**: Palette (quick open), Sidebar tree, Explorer, Breadcrumb, SegmentedTabs, UnderlineTabs.
- **Editor**: SqlEditor, renderer, terminal overlay, minimap overlay, prediction.
- **Query**: QueryDocument, QueryOutputView, QueryRunner, QueryExecution, QuerySession, SQL format, SQL parameters, VisualBuilder.

## Quality gates vs migration plan

| Gate | Status | Notes |
|------|--------|-------|
| Native rendering | ⚠️ Partial | Modal inset-based, không window modal native. Overlay self-paint. |
| Accessibility | ❌ Fail | Accessibility tree thiếu cho grid/tree/dialog. Không screen reader support. |
| Performance | ⚠️ Partial | Overlay paint trong main frame. Animation timer-based acceptable. Grid virtualized. |
| Animation | ⚠️ Partial | Animation có (toast, spinner, progress, small_translate) nhưng đơn giản, không fade-in/out overlay. |
| Multi-window | N/A | Single window app, modal không áp dụng. Multiple dialog constraint chưa có. |
| Lifecycle | ⚠️ Partial | Modal state management phân tán trong app.rs, không lifecycle manager tập trung. |

## Runtime verification constraint

- Build `db-pro-native` thất bại trong môi trường CLI này: lỗi bindgen sqlite, thiếu GUI context.
- Không thể run app để đo réalité (fps, scroll latency, edit latency, modal behavior).
- Runtime verification plan ghi nhận constraint và đề xuất alternative:
  1. Unit test component behavior (Dialog/Sheet logic) — cần egui test utils.
  2. Mock overlay behavior (ToastManager, dialog state machine) — test logic không GUI.
  3. Visual acceptance: chụp screenshot trên machine có GUI, hoặc CI GPU runner.

## Visual acceptance constraint

- Build thất bại → không chụp được screenshot ở đây.
- Plan visual acceptance evidence đề xuất: machine có GUI, CI GPU runner, 3 resolutions × 3 states (normal/loading/error/empty).

## Recommendation summary

### Prioritize P0 first

1. Focus trap cho dialog (capture keyboard focus).
2. Backdrop interaction disable (disable click-through).
3. Consistent Esc close behavior.
4. Single dialog constraint + z-order management.

### P1 sau

5. Accessibility tree cho grid/tree/dialog.
6. Responsive dialog width.
7. Animation fade-in/out overlay.
8. Visible focus indicator.

### P2 polish

9. Performance optimization (cache layout compute, GPU overlay layer).
10. Toast animation polish.
11. Dialog content adaptive.

## References

- Migration plan: `docs/10-egui-native-migration-plan.md`
- Quality baseline: `docs/plans/active/ui-core-audit/UI_CORE_QUALITY_AUDIT_2026-09-16.md`
- UI/UX perf deep: `docs/plans/active/ui-core-audit/UI_CORE_UIUX_PERF_DEEP_AUDIT_2026-09-16.md`
- Layout computation: `docs/plans/active/ui-core-audit/UI_LAYOUT_COMPUTATION_AUDIT_2026-09-16.md`
- Modal UX deep: `docs/plans/active/ui-core-audit/UI_MODAL_UIUX_AUDIT_2026-09-16.md`
- Modal runtime verification: `docs/plans/active/ui-core-audit/UI_MODAL_RUNTIME_VERIFICATION_PLAN_2026-09-16.md`
- Runtime verification: `docs/plans/active/ui-core-audit/UI_RUNTIME_VERIFICATION_PLAN_2026-09-16.md`
- Visual acceptance: `docs/plans/active/ui-core-audit/UI_VISUAL_ACCEPTANCE_PLAN_2026-09-16.md`

## Tổng kết bằng tiếng Việt

Đã audit đầy đủ core components UI của db-pro (crate `crates/ui`). Tìm thấy nhiều vấn đề UX mô-đun: modal không focus trap, không backdrop interaction disable, Esc behavior không consistent, multi-dialog không constraint. Accessibility tree thiếu cho grid/tree/dialog. Layout computation inconsistency: row height, input padding, spacing token violation, tree indent, resize divider hit target, badge position. Performance: overlay paint trong main frame, layout compute inline. Strengths: token ownership, virtualization, editor native, destructive confirm, palette keyboard nav, toast UX tốt, form validation, theme toggle. Build thất bại trong CLI nên runtime verification và visual acceptance chưa thực thi được — cần machine có GUI hoặc CI GPU runner. Reports đã lưu trong `docs/plans/active/ui-core-audit/`.
