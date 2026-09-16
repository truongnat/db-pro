# UI Core Components — Quality Audit

**Sha**: bae4765e chore: ignore local agent tooling state directories (.omh, .hermes, .kilo, .jules)
**Crate audited**: `crates/ui` (db-pro-ui)
**Date**: 2026-09-16
**Auditor**: Hermes Agent session
**Scope**: UI/UX quality + performance audit của core components, theo `docs/10-egui-native-migration-plan.md` và `docs/architecture/performance-baseline.md`.

---

## Goal

Audit chất lượng core components UI trong crate `crates/ui`, tập trung vào:
- Native egui rendering & dark/light theme token integrity
- Accessibility basics, keyboard interaction, focus
- Performance critical path: result grid, table view, query editor, query runner UI update
- Maintainability: component state size, hard-coded constants, dead-code allowances

---

## Findings

### P0 (chặt)

**P0-1 — Keyboard/accessibility first-class chưa đầy đủ trên tree/gallery explorer**
- Tree explorer trong `component_gallery_surfaces.rs` và sidebar flow (xem qua `sidebar_view.rs`) chưa thấy lộ trình keyboard navigation rõ ràng (không có centralized focus ring policy, không thấy accessible-name propagation cho tree items).
- Grid result có `handle_grid_keyboard` trong `result_grid_view.rs`, tốt — nhưng các view phụ (tree, sidebar, gallery surface) chưa đồng bộ.
- Rủi ro: nếu product claim là desktop-professional / keyboard-first thì hiện tại còn gap.

**P0-2 — Release-readiness visual acceptance evidence chưa có trong repo**
- Migration plan §Visual acceptance gate yêu cầu screenshot ở 1280×800, 1440×900, 1920×1080 với 3 trạng thái (normal/loading/error/empty) cho mỗi milestone.
- Trong repo hiện tại không thấy artifact screenshot đi kèm với commit UI gần nhất (checked `docs/plans/`, `crates/ui/`, not found image artifacts).

### P1 (chặt)

**P1-1 — `DbProApp` quá lớn (god-struct UI state)**
- `app.rs` + `app_state.rs` chứa rất nhiều field trực tiếp trong struct (connection, schema, audit, security, FDW, replication, masking, migration, diagram, result grids, editor state, query view state, gallery state...).
- Hậu quả: mỗi view module trung tâm mutable trên `self.*`, khó isolate test, dễ tạo side-effect ngầm giữa các flow.
- Không phải bug run-time, nhưng là debt kiến trúc lớn cho product UI tiếp tục mở rộng.

**P1-2 — Component gallery demo state quá lớn & hard-coded**
- `ComponentGalleryState` trong `component_gallery_view.rs` đếm ~90 field, mặc định hardcode cả demo data (connection form, SQL textarea, selected table rows, tree expand flags, alert open flags...).
- Fit cho gallery, nhưng nếu pattern này lây vào product flow sẽ khó maintain. Nên tách demo state khỏi product state.

**P1-3 — Toolbar result grid tạo allocation mỗi frame**
- `draw_grid_toolbar` trong `result_grid_view.rs` tạo nhiều `Button::new(...)` mỗi frame cho các action (copy/CSV/JSON/record/inspect...), chưa thấy caching/lazy creation.
- Grid body được virtualization, nhưng toolbar本身 là garbage mỗi frame.

**P1-4 — Magic constant phân tán**
- Một số value như `row_height = 28.0`, toolbar height `38.0`, spacing `4.0`, `180.0`, `28.0`, `34.0` xuất hiện trực tiếp trong grid/view code. Một số đã vào `tokens.rs`, một số chưa.
- Nên theo dõi: không có central spacing token cho toolbar/grid row height, dễ lệch khi theme scale thay đổi.

**P1-5 — Nhiều `#[allow(dead_code)]` trên entity public-ish**
- `src/ide_workspace.rs:60-67` (`MigrationStatus`), `change_set.rs:86-89`, `change_set.rs:207-210`, `app.rs:743`.
- Nếu planned-unused thì ok; nếu leftover thì nên dọn hoặc feature-flag rõ ràng.

**P1-6 — UI update path dùng pha trộn UiCommand và mutable trực tiếp**
- Nhiều view mutate trực tiếp `self.*` (gallery, query view header combo), bên cạnh `dispatch_command(UiCommand::...)`.
- Cần documentation rõ ràng: view nào chỉ được mutable local, view nào phải qua command/event.

### P2 (mềm)

**P2-1 — Cố định chiều rộng combo header query**
- `query_view.rs` hardcode `HEADER_CONN_COMBO_WIDTH = 170.0`, `HEADER_SCHEMA_COMBO_WIDTH = 130.0`. OK cho boundedness, nhưng nên đảm bảo tooltip/overflow xử lý đọc đúng trên DPI cao/Retina.

**P2-2 — Thử quest accessibility cho table relation/dependency jump**
- `draw_table_relations_view`, `draw_table_dependencies_view` có jump action (open table) qua button/click, nhưng không thấy keyboard shortcut tương ứng. Nếu user phụ thuộc keyboard sẽ khó dùng.

**P2-3 — Truyền màu trực tiếp từ theme nhưng chưa uniform cho icon color**
- Một số icon color dùng trực tiếp field theme như `self.theme.warning`, `self.theme.accent`, `self.theme.text_muted` — ổn, nhưng nên kiểm tra consistency tương tự cho grid/editor component.

**P2-4 — SQL formatter & parameter discovery conservative, good — nhưng test boundary cần duy trì**
- `sql_format.rs`, `sql_parameters.rs` đã bảo vệ literals/comments/dollar-quoted, skip `'` trong parameter discovery. Giữ coverage này.

---

## Strengths (lành)

- Token system rõ ràng: màu/surface/typography/radius/spacing gom vào `DbProTheme` + `tokens.rs`. Widget không hard-code RGB.
- Dark/light mode hoạt động qua `apply()` mapping vào `egui::Visuals`.
- Virtualization + cache cho result grid (`GridProjectionCache`, `GridSelectionCache`, comment giải thích độ đo ~36.5ms → được giải quyết).
- Editor native (`editor/renderer.rs`) có đủ::{caret, selection, syntax highlight, bracket matching, AI ghost text, diagnostic squiggle, search highlight, IME output rect, pair bracket guard trong string/comment}.
- Destructive UX có confirmation flags riêng (pending DDL, data delete, discard changes).
- SQL formatter bảo vệ literals/comments/dollar-quoted, test coverage đủ.
- Parameter discovery skip `'`/comment, phát `$N`, `:name`, `?`.
- Accessibility basics có trong component library (`Button::accessible_name`, form field width contract).

---

## Performance notes (dựa trên code path & baseline docs)

- Hiển thị result grid: draw path dùng `ScrollArea::vertical().show_rows(...)` đúng kiểu egui virtual scroll.
- Grid cache: `GridProjectionCache` & `GridSelectionCache` có tồn tại, giải thích độ đo trước đây ~36.5ms → đã được xử lý. Cần runtime verification thực tế với dataset lớn để confirm.
- Editor: highlight token hóa từng dòng, có cache token — tranh thủ tốt, nhưng cần profile nếu file SQL rất dài (>10k lines).
- Query runner UI update: không thấy file `query_runner.rs`/`query_execution.rs`/`query_output_view.rs`/`query_session.rs` tồn tại trong crate — query execution flow có thể nằm ở runtime crate, không nằm trong scope UI crate này.

---

## UI/UX notes (focus)

- Gallery 3-col layout rõ ràng: icon/text/overlay, hover tooltip, focus ring, button feedback — GOOD.
- Form component (`input.rs`, `overlay.rs`, `button.rs`) có field width contract, error state, search input variant — đủ cơ bản cho desktop app.
- Table component (`table_view.rs`, `table_metadata_view.rs`) có search filter, badge variant, monospace type column, relation jump action — usable.
- Query view: connection/schema combo, statement counter, run/clear buttons, SQL formatting line decorators — hoàn chỉnh.
- Result grid: header freeze, search box, sort indicators, column menu, row selection, clipboard actions, keyboard navigation — đầy đủ.

---

## Next steps

1. **P0-1**: thêm keyboard navigation & accessible-name cho tree/gallery/sidebar — priority cao nếu target là keyboard-first product.
2. **P0-2**: Sinh screenshot visual acceptance evidence cho milestone UI gần nhất (1280×800, 1440×900, 1920×1080; normal/loading/error/empty).
3. **P1-1**: cân nhắc tách `DbProApp` thành nhiều state struct nhỏ hơn (AppShell, EditorState, QueryState, GridState, ...) — hoặc ít nhất document boundary mutation rule.
4. **P1-2**: tách ComponentGallery demo state khỏi sản phẩm; revised gallery state struct nhỏ hơn.
5. **P1-3**: lazy-create hoặc cache toolbar button instance; hoặc accept allocation nếu profiler cho thấy không phải bottleneck.
6. **P1-4**:chiếu magic constant vào `tokens.rs` / `theme.rs` cho row height, toolbar height, standard spacing.

---

## Verification

- Code paths read: `app.rs`, `app_state.rs`, `result_grid_view.rs`, `table_view.rs`, `query_view.rs`, `editor/renderer.rs`, `table_metadata_view.rs`, `component_gallery_view.rs`, `component_gallery_surfaces.rs`, `sidebar_view.rs`, `theme.rs`, `tokens.rs`, `components/button.rs`, `components/overlay.rs`, `components/input.rs`, `result_grid_header.rs`, `result_grid_clipboard.rs`, `result_grid_edit.rs`, `result_grid_cell.rs`, `grid_layout.rs`, `feedback.rs`.
- Documentation read: `docs/10-egui-native-migration-plan.md`, `docs/architecture/performance-baseline.md`, `docs/plans/performance-baseline-audit-2026-08-13.md`, `docs/plans/performance-optimization-report-2026-08-13.md`.
- Runtime verification: **chưa làm** — cần build `cargo build --release --locked -p db-pro-native` và chạy thực tế để thu thập performance evidence và screenshot.

---

## Tóm tắt bằng tiếng Việt

Đã audit xong chất lượng UI core components (`crates/ui`), tập trung vào visual/token integrity, virtualization, editor, form/table component, và performance path. Kết quả: hệ thống token & dark/light theme tốt, grid/editor/table component khá và có cache, nhưng còn debt kiến trúc lớn (god-struct `DbProApp`), gallery demo state quá lớn, toolbar allocation mỗi frame, magic constant phân tán, và thiếu evidence visual acceptance + keyboard-first cho tree/sidebar/gallery. P0/P1/P2 đã ghi nhận. Cần runtime verification thực tế và screenshot acceptance gate.

---

## Plan references

- `docs/10-egui-native-migration-plan.md`
- `docs/architecture/performance-baseline.md`
- `docs/plans/performance-baseline-audit-2026-08-13.md`
- `docs/plans/performance-optimization-report-2026-08-13.md`

---

## Checklist

- [x] Audit core UI components
- [x] UI/UX quality findings
- [x] Performance code-path findings
- [ ] Runtime verification (build + run db-pro-native)
- [ ] Screenshot visual acceptance evidence
- [ ] Keyboard focus/accessibility audit for tree/sidebar/gallery (P0-1)
- [ ] God-struct boundaries documentation (P1-1)

---

## P0/P1/P2 summary

- P0: 2 (accessibility/tree keyboard, visual acceptance evidence)
- P1: 6 (god-struct, gallery state, toolbar allocation, magic constant, dead_code, mutation path)
- P2: 4 (combo width, table jump keyboard, icon color consistency, formatter test boundary)

---

*Hermes Agent session — actionable findings drafted; pending runtime verification.*
