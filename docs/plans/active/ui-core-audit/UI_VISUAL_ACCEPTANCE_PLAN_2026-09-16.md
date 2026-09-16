# UI Visual Acceptance Evidence Plan

**SHA**: bae4765e chore: ignore local agent tooling state directories (.omh, .hermes, .kilo, .jules)
**Crate**: `crates/ui`
**Date**: 2026-09-16
**Status**: DRAFT — pending runtime environment availability

---

## Goal

Sinh screenshot visual acceptance evidence theo `docs/10-egui-native-migration-plan.md` §Visual acceptance gate:
- 3 độ phân giải: 1280×800, 1440×900, 1920×1080
- 3 trạng thái: normal, loading, error/empty
- Áp dụng cho milestone UI gần nhất

---

## Constraint (ghi nhận)

- Build `db-pro-native` thất bại trong môi trường CLI này (lỗi bindgen sqlite, thiếu GUI).
- Không thể chạy app để chụp screenshot ở đây.
- **Alternative**: chụp trên machine khác có GUI, hoặc dùng CI GPU runner, hoặc skip nếu không có runtime.

---

## Nếu có runtime — screenshot plan

### Độ phân giải cần chụp

| # | Resolution | Mục đích |
|---|-----------|----------|
| 1 | 1280×800 | Laptop phổ biến, kiểm tra density |

| 2 | 1440×900 | Desktop phổ thông |
| 3 | 1920×1080 | Full HD, kiểm tra spacing scale |

### Trạng thái cần chụp (cho mỗi độ phân giải)

| State | Mô tả |
|-------|-------|
| Normal | App chạy, connected, query result grid visible, table metadata visible, query editor visible |
| Loading | Kết nối database đang loading, query-executing indicator visible |
| Error/Empty | Query result empty (no rows), error state (query lỗi), empty schema state |

### Component cần coverage

- **Query workspace**: connection/schema combo, editor, run/clear buttons, result grid (header, rows, sort, selection, clipboard toolbar)
- **Table metadata view**: structure tab (columns table, PK/FK badges), indexes tab, foreign keys tab (jump action), constraints tab (filter/segment), dependencies tab (direction badge, jump)
- **Result grid**: header freeze, row selection, keyboard navigation (nếu có), resize divider
- **Editor**: caret, selection, syntax highlight, squiggle, ghost text (nếu có)
- **Sidebar/Explorer tree**: indent, icon, chevron, selected state, hover state
- **Dialog/Sheet**: overlay, animation transition, content scroll
- **Theme toggle**: dark mode / light mode (nếu có toggle live)

### Ảnh artifact đặt ở đâu

- `docs/plans/active/ui-core-audit/screenshots/<resolution>/<state>/<component>.png`
- Hoặc `docs/plans/completed/native-ui-foundation/screenshots/...` nếu milestone đã hoàn thành.

---

## Kiểm tra consistency từ screenshot

- Row height consistency: grid 28.0 vs table 44.0 → kiểm tra visual density
- Input field vertical padding: 4/5/6px → kiểm tra visual gap
- Tree indent: 14.0+8.0 vs 10.0 → kiểm tra indent consistency
- Resize divider hit target: 6px → kiểm tra feel
- PK/FK badge position: kiểm tra vertical align với text

---

## Tóm tắt bằng tiếng Việt

Đã tạo visual acceptance evidence plan theo migration plan §Visual acceptance gate. Build db-pro-native thất bại trong môi trường CLI này. Plan ghi nhận constraint, đề xuất alternative (chụp trên machine có GUI, CI GPU runner). Nếu có runtime, chụp 3 độ phân giải × 3 trạng thái cho query workspace, table metadata, result grid, editor, sidebar tree, dialog/sheet, theme toggle. Coverage consistency check từ screenshot cho row height, input padding, tree indent, resize hit target, badge position.

---

## Checklist

- [ ] Fix build environment (bindgen, cmake, pkg-config, sqlite3)
- [ ] Build `cargo build --release --locked -p db-pro-native`
- [ ] Chạy db-pro-native trên desktop thật / remote desktop / CI GPU runner
- [ ] Chụp screenshot 1280×800, 1440×900, 1920×1080 (normal/loading/error-empty)
- [ ] Chụp component: query workspace, table metadata, result grid, editor, sidebar tree, dialog/sheet, theme toggle
- [ ] Kiểm tra consistency từ screenshot (row height, input padding, tree indent, resize hit target, badge position)
- [ ] Lưu artifact vào `docs/plans/active/ui-core-audit/screenshots/` hoặc milestone completed dir

---

*Hermes Agent session — visual acceptance evidence plan drafted (build constraint noted).*
