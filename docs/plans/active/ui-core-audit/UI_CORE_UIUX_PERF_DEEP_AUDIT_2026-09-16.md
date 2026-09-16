# UI Core Components — UI/UX + Performance Deep Audit

**SHA**: bae4765e chore: ignore local agent tooling state directories (.omh, .hermes, .kilo, .jules)
**Crate audited**: `crates/ui` + `crates/infrastructure/benches/sqlite_benchmarks.rs` + `crates/ui/benches/result_grid_benchmarks.rs`
**Date**: 2026-09-16
**Auditor**: Hermes Agent session
**Scope**: Deep audit focusing on UI/UX quality và performance path (render, state-update, scroll, selection lookup, projection/rebuild).

---

## Goal

Mở rộng audit trước đó, tập trung vào:
- UI/UX: keyboard navigation, focus ring, focus trap, button feedback, form state, toolbar flow, collapsed/expanded state consistency.
- Performance: result grid projection rebuild, selection lookup rebuild, scroll window materialisation, editor tokenisation, tool/tooltip allocation path, metric-budget compliance.

References:
- `docs/10-egui-native-migration-plan.md`
- `docs/architecture/performance-baseline.md`
- `docs/plans/performance-baseline-audit-2026-08-13.md`
- `docs/plans/performance-optimization-report-2026-08-13.md`
- `crates/ui/benches/result_grid_benchmarks.rs`
- `crates/infrastructure/benches/sqlite_benchmarks.rs`

---

## UI/UX findings

### P0 (chặt)

**P0-1 — Keyboard navigation & accessible name cho tree/sidebar/gallery chưa full**
- Grid result (`result_grid_view.rs`) có `handle_grid_keyboard`, hàng tốt.
- Tree explorer (`component_gallery_surfaces.rs`),sidebar view, gallery surface tree items: chưa thấy centralized keyboard navigation policy, focus ring policy, hay accessible-name propagation cho tree item. Nếu product target là desktop-professional / keyboard-first thì đây là gap.
- Nên có: `egui::Response` focus ring visible, tab order đơn giản, aria-name/thông báo toast cho tree jump.

**P0-2 — Visual acceptance evidence chưa có trong repo**
- Migration plan §Visual acceptance gate yêu cầu screenshot ở 1280×800, 1440×900, 1920×1080 với 3 trạng thái (normal/loading/error/empty) cho mỗi milestone.
- Trong repo không thấy artifact screenshot đi kèm commit UI gần nhất.

### P1 (chặt)

**P1-1 — `DbProApp` god-struct UI state**
- `app.rs`/`app_state.rs` chứa một struct khổng lồ với hàng trăm field (connection, schema, result grid, editor, query view, gallery, sidebar, FDW, replication, masking, migration, diagram...).
- View modules mutable trực tiếp `self.*`, khó isolate test, dễ side-effect ngầm giữa flows.
- Khuyến nghị: tách nhỏ state struct theo domain (AppShell, EditorState, QueryState, GridState, GalleryState, ...) hoặc ít nhất document boundary mutation rule rõ ràng.

**P1-2 — `ComponentGalleryState` quá lớn + demo data hardcoded**
- ~90 field, default chứa connection form, SQL textarea, selected rows, tree expand flags, alert flags...
- Fit cho gallery demo, nhưng pattern này dễ lây sang product flow → khó maintain. Nên tách demo state khỏi product state.

**P1-3 — Toolbar result grid allocation mỗi frame**
- `draw_grid_toolbar` tạo nhiều `Button::new(...)` mỗi frame cho copy/CSV/JSON/record/inspect...
- Grid body đã virtualization, nhưng toolbar vẫn là garbage mỗi frame. Nên lazy-create hoặc cache instance nếu profiler cho thấy bottleneck.

**P1-4 — Magic constant phân tán**
- `row_height = 28.0`, toolbar height `38.0`, spacing `4.0`, `180.0`, `28.0`, `34.0`, `HEADER_CONN_COMBO_WIDTH = 170.0`, `HEADER_SCHEMA_COMBO_WIDTH = 130.0`...
- Một số đã vào `tokens.rs`, một số chưa. Nên centralize: row height, toolbar height, standard spacing, header combo width vào tokens/theme.

**P1-5 — Nhiều `#[allow(dead_code)]` trên entity public-ish**
- `ide_workspace.rs:60-67` (`MigrationStatus`), `change_set.rs:86-89`, `change_set.rs:207-210`, `app.rs:743`.
- Nếu planned-unused thì ok; nếu leftover thì nên dọn hoặc feature-flag rõ ràng.

**P1-6 — Mutation path pha trộn UiCommand và direct mutable `self.*`**
- View gallery mutate trực tiếp `self.gallery_state.db_card_status = ...`, view query mutate combo trực tiếp `self.selected_connection_id`.
- Cần boundary documentation rõ: view nào chỉ mutable local, view nào phải qua command/event.

### P2 (mềm)

**P2-1 — Combo header cố định chiều rộng**
- `query_view.rs` hardcode combo width. OK cho boundedness, nhưng cần đảm bảo tooltip/overflow xử lý đúng trên DPI cao/Retina.

**P2-2 — Table relation/dependency jump thiếu keyboard shortcut**
- `draw_table_relations_view`, `draw_table_dependencies_view` có jump qua button click, không thấy keyboard shortcut. Nếu user phụ thuộc keyboard sẽ khó dùng.

**P2-3 — Icon color dùng trực tiếp theme field**
- Một số icon color dùng `self.theme.warning`, `self.theme.accent`, `self.theme.text_muted` — ổn, nhưng nên kiểm tra consistency tương tự cho grid/editor component.

**P2-4 — SQL formatter & parameter discovery boundary test cần duy trì**
- `sql_format.rs`, `sql_parameters.rs` đã bảo vệ literals/comments/dollar-quoted, skip `'/comment`. Giữ coverage này.

---

## UI/UX strengths (lành)

- Token system rõ ràng, không hard-code màu trong component.
- Dark/light mode hoạt động qua `apply()` mapping egui::Visuals.
- Component library có focus ring, tooltip, accessible name cho button.
- Form component (`input.rs`, `overlay.rs`, `button.rs`) có field width contract, error state, search variant.
- Table component có search filter, badge variant, monospace type column, relation jump action.
- Query view hoàn chỉnh: connection/schema combo, statement counter, run/clear, SQL formatting decorator.
- Result grid đầy đủ: header freeze, search box, sort indicators, column menu, row selection, clipboard action, keyboard navigation.
- Gallery 3-col layout rõ ràng, hover tooltip, button feedback, overlay.

---

## Performance findings

### Benchmark evidence (đã có trong repo)

**Result grid benchmarks** (`crates/ui/benches/result_grid_benchmarks.rs`):
- `bench_million_row_metadata`: 1M row projection (không sort, không filter). Measure `filtered_sorted_indexes` bản chất.
- `bench_sorted_projection`: 200k row x 4 column — bao gồm temporal column sort (parse date/time cho mỗi comparison). Đây là case đo lường tại sao draw path không được rebuild projection mỗi frame → dẫn đến `GridProjectionCache`.
- `bench_visible_scroll_window`: materialize 100 visible rows từ index.
- `bench_requested_grid_sizes`: `GridSelectionLookup::new` với 1k / 10k rows x 50 columns.
- `bench_selection_lookup`: 200k rows x 4 columns — đo `GridSelectionLookup::new`. Comment ghi rõ: trước cache, draw path rebuild mỗi frame đo được 36.5ms của ~40ms steady-state frame at 200k rows x 4 columns in debug.

**SQLite benchmarks** (`crates/infrastructure/benches/sqlite_benchmarks.rs`):
- Budget target: connect <100ms, metadata small DB <300ms, serialize 10k rows <150ms, cancel ack <200ms.
- Benchmark: connect/disconnect, introspect small/large schema (50 tables x 20 columns), query 10k/100k rows, JSON/BLOB query, serialize large text (1k rows x ~4-5KB), explain.

### Performance path analysis (dựa trên code)

**Result grid draw path:**
- `result_grid_view.rs`: `draw_grid_body` dùng `ScrollArea::vertical().show_rows(...)` — đúng egui virtual scroll.
- `GridProjectionCache` và `GridSelectionCache` tồn tại, giải thích độ đo ~36.5ms → đã giải quyết.
- Tuy nhiên: `draw_grid_toolbar` allocation mỗi frame vẫn còn. Header freeze, search box, sort indicators, column menu, row selection, clipboard action, keyboard navigation — đầy đủ.
- Cần runtime verification thực tế: dataset lớn (1M row), scroll wheel fps, sort click latency, selection lookup rebuild cost khi column order thay đổi.

**Editor performance path:**
- `editor/renderer.rs`: highlight token hóa từng dòng, có cache token.
- Cần profile nếu SQL file rất dài (>10k lines) — có thể tách highlight path thành lazy tokenisation per visible range.
- Caret/selection/squiggle/ghost text/search highlight hoạt động, nhưng cần đo lường draw time khi có nhiều diagnostic squiggle + search highlight + IME composition.

**Query runner UI update path:**
- Không thấy file `query_runner.rs`/`query_execution.rs`/`query_output_view.rs`/`query_session.rs` trong crate UI → query execution flow có thể nằm ở runtime crate, không nằm scope UI crate này.
- Nếu UI cập nhật khi result stream-in, cần đảm bảo không rebuild projection/selection lookup mỗi chunk.

**Tool/tooltip allocation path:**
- Một số tooltip tạo string mỗi frame (format! trong on_hover_text). Nếu nhiều row, nhiều column → nhiều allocation nhỏ. Acceptable cho desktop app thông thường, nhưng nên profile khi grid có 10k+ rows visible.

---

## UI/UX mở rộng — chi tiết từng component

### Result grid
- Header freeze, sort indicators, column menu (resize/reorder/hide), search box, row selection (checkbox + keyboard), clipboard actions (copy/CSV/JSON/record/inspect), keyboard nav (arrow keys, page up/down, home/end), selected row highlight.
- UX strength: đầy đủ cho desktop app.
- UX gap: toolbar allocation mỗi frame; magic constant phân tán.

### Table view (table_metadata_view.rs)
- Structure tab: metrics chips, columns table (search filter, type badge, nullability, PK/FK flags, default expression), column detail window.
- Indexes tab: index metadata table, unique badge, index detail window.
- Foreign keys tab: relations table, target jump button, copy action.
- Constraints tab: categorized PK/FK/Unique/Check/NotNull, filter, segment selector.
- Dependencies tab: direction badge (DEPENDS ON / DEPENDED BY), kind badge, jump target button.
- UX strength: đầy đủ, có filter, search, detail popup, jump action.
- UX gap: relation/dependency jump thiếu keyboard shortcut; jump action chỉ qua button click.

### Query view
- Connection/schema combo cố định chiều rộng (170.0 / 130.0).
- Statement counter, run/clear buttons, SQL formatting decorator.
- UX strength: hoàn chỉnh.
- UX gap: combo width hardcode, cần tooltip/overflow xử lý DPI cao.

### Editor (editor/renderer.rs)
- Caret, selection, syntax highlight, bracket matching, AI ghost text, diagnostic squiggle, search highlight, IME output rect, pair bracket guard trong string/comment.
- UX strength: native editor đủ tính năng.
- UX gap: cần profile khi file dài; highlight path có thể lazy per visible range.

### Component gallery
- 3-col layout (icon/text/overlay), hover tooltip, focus ring, button feedback, overlay.
- UX strength: component library demo rõ ràng, live theme toggle.
- UX gap: gallery state quá lớn, demo data hardcode.

### Sidebar
- Tree explorer có expand/collapse, icon, label.
- UX strength: cơ bản.
- UX gap: keyboard navigation, focus ring policy, accessible name cho tree item.

---

## Next steps

1. **P0-1**: thêm keyboard navigation & accessible name cho tree/sidebar/gallery.
2. **P0-2**: sinh screenshot visual acceptance evidence (1280×800, 1440×900, 1920×1080; normal/loading/error/empty).
3. **P1-1**: cân nhắc tách `DbProApp` thành nhiều state struct nhỏ hơn, hoặc document boundary mutation rule.
4. **P1-2**: tách ComponentGallery demo state khỏi sản phẩm.
5. **P1-3**: lazy-create hoặc cache toolbar button instance; hoặc accept allocation nếu profiler cho thấy không phải bottleneck.
6. **P1-4**:chiếu magic constant vào `tokens.rs`/`theme.rs`.
7. **P2-2**: thêm keyboard shortcut cho table relation/dependency jump.
8. **Runtime verification**: build `cargo build --release --locked -p db-pro-native`, run thực tế, đo lường fps/scroll latency/sort latency/selection lookup rebuild.

---

## Verification status

- Code paths read: `app.rs`, `app_state.rs`, `result_grid_view.rs`, `table_view.rs`, `query_view.rs`, `editor/renderer.rs`, `table_metadata_view.rs`, `component_gallery_view.rs`, `component_gallery_surfaces.rs`, `sidebar_view.rs`, `theme.rs`, `tokens.rs`, `components/button.rs`, `components/overlay.rs`, `components/input.rs`, `result_grid_header.rs`, `result_grid_clipboard.rs`, `result_grid_edit.rs`, `result_grid_cell.rs`, `grid_layout.rs`, `feedback.rs`, `result_grid_benchmarks.rs`, `sqlite_benchmarks.rs`.
- Documentation read: `docs/10-egui-native-migration-plan.md`, `docs/architecture/performance-baseline.md`, `docs/plans/performance-baseline-audit-2026-08-13.md`, `docs/plans/performance-optimization-report-2026-08-13.md`.
- Runtime verification: **chưa làm** — cần build + run db-pro-native để thu thập performance evidence và screenshot.

---

## Tóm tắt bằng tiếng Việt

Đã mở rộng audit focus vào UI/UX và performance. Kết quả:
- UI/UX: component library, grid, table, query view, editor, gallery đều khá và đầy đủ, nhưng còn gap keyboard navigation cho tree/sidebar/gallery, toolbar allocation mỗi frame, magic constant phân tán, mutation path pha trộn command/direct mutable.
- Performance: có benchmark evidence trong repo (result grid projection, selection lookup, scroll window; SQLite connect/introspect/query/serialize), grid draw path đã có cache giải quyết 36.5ms issue, editor highlight có cache token. Cần runtime verification thực tế và profile khi file dài/nhiều diagnostic.
- P0/P1/P2 đã ghi nhận. Cần screenshot acceptance gate + build/run db-pro-native để đo lường thực tế.

---

## P0/P1/P2 summary

- P0: 2 (keyboard/accessibility tree/sidebar/gallery, visual acceptance evidence)
- P1: 6 (god-struct, gallery state, toolbar allocation, magic constant, dead_code, mutation path)
- P2: 4 (combo width, table jump keyboard, icon color consistency, formatter test boundary)

---

*Hermes Agent session — UI/UX + performance deep audit drafted; pending runtime verification.*
