# UI Layout Computation Audit — Distance / Padding / Margin / Width / Height

**SHA**: bae4765e chore: ignore local agent tooling state directories (.omh, .hermes, .kilo, .jules)
**Crate audited**: `crates/ui` (db-pro-ui)
**Date**: 2026-09-16
**Auditor**: Hermes Agent session
**Scope**: Tính toán layout thủ công từ khoảng cách, padding, margin, width/height — kiểm tra consistency, 4px grid adherence, hard-coded magic numbers.

---

## Constants centralize (tốt)

Từ `tokens.rs`:
```
SPACE_XXS=2.0, SPACE_XS=4.0, SPACE_SM=8.0, SPACE_MD=12.0, SPACE_LG=16.0,
SPACE_XL=20.0, SPACE_2XL=24.0, SPACE_3XL=32.0, SPACE_4XL=40.0,
SPACE_5XL=48.0, SPACE_6XL=64.0
ICON_TEXT_GAP=8.0, LABEL_HELPER_GAP=4.0, FORM_FIELD_GAP=14.0
```

Từ `app_types.rs`:
```
SIDEBAR_MIN_WIDTH=220.0, SIDEBAR_MAX_WIDTH=380.0,
AGENT_MIN_WIDTH=300.0, AGENT_MAX_WIDTH=480.0,
OUTPUT_MIN_HEIGHT=120.0, OUTPUT_MAX_HEIGHT=420.0,
GRID_ROW_NUMBER_WIDTH=48.0, TABLE_PAGE_SIZE=100
```

Các giá trị này được dùng nhất quán ở một số chỗ. Nhưng nhiều component tính trực tiếp inline.

---

## Row height inconsistency (P1)

| Component | Header height | Row height | File |
|-----------|--------------|------------|------|
| Result grid header | 34.0 | — | result_grid_header.rs:22 |
| Result grid data row | — | 28.0 | result_grid_view.rs:698 |
| Table component (data) | 36.0 | 44.0 (default) | table.rs:89,177 |
| Tree node | — | 26.0 | tree.rs:74, explorer_tree.rs:13 |

- Grid row 28.0 vs Table row 44.0 → chênh lệch 16px. Nếu query result dùng grid, table metadata dùng table component, user sẽ cảm nhận density khác nhau.
- Grid header 34.0 vs Table header 36.0 → 2px, nhỏ nhưng không hoàn toàn consistent.
- Tree node 26.0 — consistent giữa hai tree implementation (tree.rs và explorer_tree.rs).

**Khuyến nghị**: thống nhất row height cho data grid. Nếu giữ grid 28.0 cho compact, table component nên dùng 28.0-32.0 nữa, hoặc ngược lại increase grid lên 32.0-36.0 để match table.

---

## Input field vertical padding inconsistency (P1)

| Field type | inner_margin vertical | File |
|------------|----------------------|------|
| Input | 4.0 (Margin::symmetric(8.0, 4.0)) | input.rs:115 |
| PasswordInput | 4.0 (Margin::symmetric(8.0, 4.0)) | input.rs:293 |
| SearchInput | 5.0 (Margin::symmetric(8.0, 5.0)) | input.rs:409 |
| Textarea | 6.0 (Margin::symmetric(8.0, 6.0)) | input.rs:551 |

3 giá trị khác nhau: 4, 5, 6. Nên thống nhất: 4.0 cho input/password/search, 6.0 cho textarea (nếu muốn multiline cao hơn), hoặc unify tất cả về 4.0 hoặc 5.0.

---

## Spacing token not fully adopted (P1)

`add_space(X.0)` calls tìm thấy trong code nhưng không match tokens:

| Value | File(s) | Note |
|-------|---------|------|
| 3.0 | input.rs:96, 280 (label → field gap) | Không có trong tokens. Giữa XXS=2.0 và XS=4.0. |
| 6.0 | navigation_view.rs:434, 3490 | Không có trong tokens. Giữa XS=4.0 và SM=8.0. |
| 10.0 | card.rs:43, 100 (card header gap) | Không có trong tokens. Giữa SM=8.0 và MD=12.0. |

→ Violated 4px grid. Nên dùng existing tokens hoặc add token mới (ví dụ SPACE_XS_PLUS=6.0, SPACE_SM_PLUS=10.0) nhưng tốt hơn là stick với existing.

---

## Grid header vs data cell padding mismatch (P2)

| Cell type | Padding (shrink2) | File |
|-----------|-------------------|------|
| Grid header text | shrink2(8.0, 4.0) | result_grid_header.rs:125 |
| Grid data cell | shrink2(12.0, 0.0) | result_grid_view.rs:473 |
| Table cell (header & data) | shrink2(12.0, 0.0) | table.rs:302, 473 |

- Grid header: horizontal 8, vertical 4
- Grid data cell: horizontal 12, vertical 0
- Table cell: horizontal 12, vertical 0

Grid header padding khác grid data cell. Khoảng cách horizontal: 8 vs 12 (4px chênh). Vertical: 4 vs 0 (4px chênh). Nên unify: grid header cũng dùng shrink2(12.0, 0.0) hoặc grid data dùng shrink2(8.0, 4.0) — nhưng vertical 0 cho data cell là để text auto-center qua layout, có thể accept.

---

## Tree indent inconsistency (P2)

| Tree impl | Indent per level | Base indent | File |
|-----------|-----------------|-------------|------|
| DatabaseTreeNode (tree.rs) | 14.0 | 8.0 | tree.rs:94 |
| Codexen tree (explorer_tree.rs) | 10.0 (CODEX_ROW_INDENT) | — | explorer_tree.rs:15 |

Hai tree implementation dùng indent khác nhau: 14.0/level + 8.0 base vs 10.0/level. Nếu cả hai tree hiển thị trong cùng app (sidebar + explorer), user sẽ thấy indent khác nhau giữa các tree.

Chevron slot: tree.rs 14.0, explorer_tree.rs CODEX_CHEVRON_SLOT 14.0 — consistent.

---

## Resize divider hit target small (P2)

- `result_grid_header.rs:62-65`: divider_rect = col_rect.right() - 3.0 to col_rect.right() + 3.0 → 6.0px wide hit target.
- 6px là hẹp cho mouse resize. Apple HIG desktopแนะนำ 8-10px minimum cho resize handle. Nên tăng xuống 8.0 hoặc 10.0 (tức ±4.0 hoặc ±5.0).

---

## PK/FK badge vertical positioning (P2)

- `result_grid_header.rs:138`: badge_rect center.y = header_text_rect.center().y - 7.0, badge height 14.0.
- → Badge top = center.y - 14.0, bottom = center.y. Badge nằm ở nửa trên của header text rect.
- Column name và type text được căn giữa (center.y - galley.size().y * 0.5).
- → Badge và text không đồng bộ vertically: badge trên cùng, text giữa. Có thể nhìn lệch.

Khuyến nghị: vị trí badge nên căn theo text baseline hoặc dùng cùng vertical center.

---

## Column width constraints review (P2)

| Context | Min | Max | Default | File |
|---------|-----|-----|---------|------|
| Grid column (user resize) | 60.0 | 1000.0 | 180.0 (clamped 180-520) | result_grid_header.rs:365, result_grid_view.rs:783 |
| Table column (flex) | 80.0 (min flex) | — | — | table.rs:138 |
| Input field | 120.0 (INPUT_MIN_WIDTH) | available | — | input.rs:11 |

- Grid min 60.0, table flex min 80.0, input min 120.0. Context khác nhau → accept được.
- Grid default 180.0 clamp 180-520: nếu nhiều column, total width = 48.0 (gutter) + N * 180.0. Với 10 column: 48 + 1800 = 1848px. Cần horizontal scroll. Acceptable cho desktop.

---

## Dialog / Card / Sheet inner margin (P2)

| Container | inner_margin | File |
|-----------|--------------|------|
| Card | 16.0 (Margin::same(16.0)) | card.rs:18 |
| Dialog | 20.0 (Margin::same(20.0)) | dialog.rs:167 |
| Sheet | 16.0 (Margin::same(16.0)) | dialog.rs:277 |

Dialog cao hơn card/sheet 4px. Có thể intentional (modal cần breathing room nhiều hơn). Nếu consistent, nên unify 16.0 hoặc dokumentation rõ ràng.

---

## Badge vs Button icon gap (P2)

| Component | Gap value | File |
|-----------|-----------|------|
| Badge (dot/icon → text) | 4.0 (BADGE_GAP) | badge.rs:10 |
| Button (icon → text) | 8.0 (ICON_TEXT_GAP) | button.rs:12 |

Badge compact hơn button, gap 4 vs 8 — acceptable. Không cần unify.

---

## Spacing token adoption summary (P1)

Tổng `add_space` calls tìm thấy: 215+ (search bị truncated, chỉ thấy 50+ trong kết quả).

Giá trị không match tokens: 3.0, 6.0, 10.0. Các giá trị match tokens: 2.0, 4.0, 8.0, 12.0, 16.0.

**Khuyến nghị**: audit toàn bộ add_space calls, replace 3.0→SPACE_XXS hoặc SPACE_XS, 6.0→新 token hoặc SPACE_XS, 10.0→SPACE_SM hoặc新 token.

---

## Positive findings (lành)

- `tokens.rs` có 4px grid system rõ ràng, nhiều component dùng.
- Button size tokens nhất quán: Sm (26px), Default (32px), Lg (38px), Icon (32px), IconSm (24px) — internal consistency tốt.
- Badge computation transparent: width = pad_x*2 + leading + gap + text_width, height = (text_height + pad_y*2).max(min_height).
- Table column width algorithm rõ ràng: fixed + flex distribution, min 80.0 flex.
- Dialog/Sheet animation translate distances consistent: 8.0 / 16.0.
- Tree row height consistent giữa hai impl (26.0).

---

## Next steps

1. **P1**: Thống nhất row height giữa grid và table (28.0 vs 44.0).
2. **P1**: Thống nhất input field vertical padding (4/5/6 → 4 hoặc 5).
3. **P1**: Audit add_space calls, replace giá trị không match tokens (3.0, 6.0, 10.0).
4. **P2**: Unify grid header padding với data cell (8,4) vs (12,0).
5. **P2**: Thống nhất tree indent giữa hai implementation (14.0+8.0 vs 10.0).
6. **P2**: Tăng resize divider hit target từ 6.0px → 8.0-10.0px.
7. **P2**: Căn chỉnh PK/FK badge vertical position với text.

---

## Tóm tắt bằng tiếng Việt

Đã audit tính toán layout từ khoảng cách, padding, margin, width/height. Kết quả:
- Row height inconsistency lớn: grid 28.0 vs table 44.0 (chênh 16px).
- Input field vertical padding inconsistency: 4/5/6px.
- Spacing token không được dùng hết: add_space(3.0), (6.0), (10.0) vi phá 4px grid.
- Grid header padding khác data cell (8,4) vs (12,0).
- Tree indent khác nhau giữa hai implementation (14.0+8.0 vs 10.0).
- Resize divider hit target 6.0px hẹp.
- PK/FK badge vertical position có thể lệch với text.

P0 không có. P1: 3 (row height, input padding, spacing token). P2: 4 (grid padding, tree indent, resize hit target, badge position).

---

## Checklist

- [x] Grid header/row height audit
- [x] Table component height audit
- [x] Input field padding audit
- [x] Spacing token adoption audit
- [x] Grid header vs data cell padding audit
- [x] Tree indent consistency audit
- [x] Resize divider hit target audit
- [x] PK/FK badge position audit
- [ ] Replace hard-coded spacing values (3.0, 6.0, 10.0)
- [ ] Thống nhất row height grid vs table
- [ ] Thống nhất input vertical padding
- [ ] Tăng resize hit target

---

## P0/P1/P2 summary

- P0: 0
- P1: 3 (row height inconsistency, input padding inconsistency, spacing token violation)
- P2: 4 (grid header vs data padding, tree indent, resize hit target, badge position)

---

*Hermes Agent session — layout computation audit finished.*
