# UI Consistency Audit — báo cáo hợp nhất (vòng 1 + vòng 2)

**Ngày:** 2026-10-22
**Mốc code:** `89095cc7d4b056a9fcc19dd9368f7c2863c8759f` (main)
**Phạm vi:** foundation, font, spacing, layout, presentation, states (vòng 1: 25 scenario /
35 ảnh) và trace chuyên sâu Select/dropdown (vòng 2: 21 call site, WID geometry dump).
**Bằng chứng:** `/tmp/ui-audit-round/` (PNG + audit JSON), `/tmp/select-audit/` (crop +
WID log). Skill tái lập: `.skills/ui-consistency-audit/`.
**Kết quả:** 25 finding — **3 P1, 22 P2**; 4 false positive đã loại; 3 evidence gap.

---

## 1. Tóm tắt điều hành

Bố cục tổng thể dùng đúng token và component chung, nhưng **4 cụm lỗi hệ thống** tạo ra
gần như toàn bộ vấn đề:

1. **Select tự phá layout cha** — `Select::show` bọc nội dung trong
   `Layout::top_down(Align::Min)` (`select.rs:170`). Hệ quả: top-align trong hàng
   ngang (lệch 11 px ở toolbar query), left-anchor trong vùng RTL (filter history
   chiếm 88/164 px, trống 76 px bên phải), và mọi call site thừa hưởng lỗi.
2. **Default ngầm của component** — theme mặc định là light (`select.rs:83`), không có
   state disabled, label không theo `Label` contract. Call site quên một prop là ra lỗi
   im lặng (engine select trắng trên dialog tối).
3. **Container tự viết layout** — explorer toolbar lồng RTL/LTR làm Refresh bị Filter
   đè; composer agent dùng `bottom_up` nhưng nội dung author top-down; query toolbar
   chip 50 px cạnh nút 28 px.
4. **Chuỗi và state không đồng bộ** — row cap tiếng Anh hardcode, ~417 chuỗi chưa i18n,
   required field thiếu inline error, menu option không có semantic name.

## 2. Ba ca điển hình (đo runtime)

### 2.1 Query editor — schema chip + row-limit select

| Quan sát | Số đo |
|---|---|
| Chip `postgres / public` cao | **50 px** (card padding 12 px) |
| Run / Explain / Format | **28 px** |
| Select `500 rows` | `y82..110` (28 px) so với Run `y93..121` → **lệch 11 px** |
| Schema picker | bespoke popup 280 px; connection list cuộn 180 px, **schema list không cuộn** |

Nguyên nhân: chip tự dựng card; Select bị `top_down` đẩy lên đỉnh hàng. Schema picker
(`query_context_picker_view.rs:20-100`) không dùng Select chung, hardcode
`"Connection"`/`"Schema"`, không keyboard/Escape. Trên connection nhiều schema (tenant),
các schema cuối không thể chọn.

### 2.2 Sidebar — filter loại history

`sidebar_history_view.rs:63-93` đặt Select 88 px trong `right_to_left`; đo được
`x132..220` trong slot `x132..296` → **76 px trống bên phải**. Cùng gốc `select.rs:170`.
Ba filter sidebar dùng ba width khác nhau: 88 / 96 / 120.

### 2.3 New Connection — engine select

`connection/view.rs:247-252` là call site duy nhất trong 21 chỗ thiếu `.theme(...)`;
default light khiến trigger **nền trắng trên dialog dark #2f2f2f**. Đây là hệ quả của
default ngầm, không phải lỗi riêng lẻ.

## 3. Ma trận Select (21 call site)

| Nhóm | Size | Variant | Width | Label | Ghi chú |
|---|---|---|---|---|---|
| Query row-limit | 28 | Ghost | 108 | không | English hardcode |
| Query chart ×5 | 28 | Ghost | 88–120 | `.small()` | hàng không wrap |
| Sidebar history | 28 | Ghost | 88 | không | lệch trái 76 px |
| Sidebar problems ×2 | 28 | Ghost | 96/120 | không | |
| Table page-limit | 28 | Ghost | 104 | không | đã i18n |
| Agent mode | 28 | Ghost | 88 | không | không có disabled visual |
| Schema form ×6 | 28 | Ghost | 120–180 | `.small()` | cạnh Input 38 px |
| Visual query builder | 28 | Ghost | 160 | không | |
| Transfer activity ×2 | 28 | Ghost | avail/8 | không | |
| Cell editor | 28 | mặc định | avail | không | |
| Files environment | 28 | **Outline** | avail | **có** | chuẩn nhất |
| Connection advanced ×2 | 28 | **Outline** | 220/260 | không | |
| New Connection engine | **38** | Outline | avail | có | **thiếu theme** |

Tổng: 10 width literal, 28/38 trộn trong cùng form, Ghost/Outline không quy tắc,
2/21 có label chuẩn.

## 4. Bảng finding hợp nhất

| ID | Mức | Finding | Anchor | Trạng thái |
|---|---|---|---|---|
| UI-01 | **P1** | Refresh Schema bị Filter đè cùng slot 26×26 | `explorer_toolbar_view.rs:21-53` | fixed |
| UI-02 | **P1** | Agent composer đảo thứ tự (label dưới, Send trên) | `agent_surface_view.rs:141-153` | fixed |
| UI-03 | **P1** | Schema picker không cuộn → không chọn được schema cuối | `query_context_picker_view.rs:70-85` | fixed |
| UI-04 | P2 | Select top-down wrapper phá center/RTL | `select.rs:170` | fixed |
| UI-05 | P2 | Select default light + engine select trắng trên dark | `select.rs:83`, `connection/view.rs:247` | fixed |
| UI-06 | P2 | Không có disabled visual | `agent_header_view.rs:45-63` | fixed |
| UI-07 | P2 | Menu row 28 px nhưng text wrap/tràn | `option.rs:29-32,75-83` | fixed |
| UI-08 | P2 | Option không có widget_info; Escape không hoàn tác arrow selection | `select.rs:332-342` | fixed |
| UI-09 | P2 | Label Select lệch contract Label form | `select.rs:174-183`, `form/ui.rs:37-68` | fixed |
| UI-10 | P2 | Form/sidebar trộn height, 10 width literal | `schema_workbench_form.rs`, `sidebar_constants.rs:56` | fixed |
| UI-11 | P2 | Row cap hardcode English, table đã i18n | `query_context_view.rs:154-158` | fixed |
| UI-12 | P2 | Toolbar chip 50 px cạnh nút 28 px; select lệch 11 px | `query_context_view.rs:56-104` | fixed |
| UI-13 | P2 | Chart select hàng không wrap | `query_output_panes_view.rs:106-188` | fixed |
| UI-14 | P2 | New Connection tràn 800 px, scrollbar ẩn | `dialog/frame.rs:20` | fixed |
| UI-15 | P2 | Required host/username thiếu inline error | `form_fields.rs:191,248`, `workspace_actions.rs:343` | fixed |
| UI-16 | P2 | Settings lệch phải 80 px | `settings_surface_view.rs:98` | fixed |
| UI-17 | P2 | Results toolbar trộn 38/28/26/18 px | `result_grid_toolbar_view.rs:58-78` | fixed |
| UI-18 | P2 | History rail 96 px dead space, nút Copy bị cắt | `sidebar_history_view.rs:209-240` | fixed |
| UI-19 | P2 | Schema Compare 6 cặp control dính 0 px | surface compare | fixed |
| UI-20 | P2 | 13 width dialog one-off; destructive sai footer/thứ tự | `query_dialog_surface_view.rs:21-92` | deferred |
| UI-21 | P2 | Status bar cắt chữ, không tooltip | `shell_statusbar_view.rs:118-123` | fixed |
| UI-22 | P2 | Quick Open hardcode English; ~417 chuỗi chưa i18n | `palette_surface_view.rs:80-84,111` | deferred |
| UI-23 | P2 | Type scale lệch (24 unused, 16/18/hardcode) | `primitive.rs:67-114` | fixed |
| UI-24 | P2 | Dark `text_muted` 4.09:1 < AA | `semantic.rs:182-290` | fixed |
| UI-25 | P2 | Diagram control thiếu accessible name | `diagram_view.rs:1171` | fixed |

### 4.1 Kết quả sửa (commit `0e370498`)

- **Select component** — `show` chỉ bọc `vertical` khi có label; theme `Option<DbProTheme>`
  tự suy từ visuals; disabled visual; menu row truncate 1 dòng; `WidgetInfo` cho option;
  `MenuKeyAction` (Enter commit, Escape revert); label dùng `Label` chung.
- **P1 surfaces** — explorer toolbar reserve slot trước khi vẽ search; composer allocate
  rect rồi author top-down; schema picker bọc `ScrollArea` 180 px + Escape.
- **Runtime evidence** — 42 PNG tại `/tmp/ui-audit/wave2/png` (8 surface × 3 size dark,
  4 surface × 3 size light, explorer/agent × 3 size), tất cả `exit=0`; menu row-limit
  được capture ở trạng thái mở. Chi tiết từng finding: `VERIFICATION.md`.
- **Gate** — fmt/check/clippy/test/release build/UI-architecture pass; clean-code diff
  ratchet 0 fail. `cc-scan --fail-on error` toàn repo vẫn fail vì 21 lỗi legacy có
  baseline rỗng — không do diff này, không regenerate baseline.
- **Hoãn** — UI-20 (13 dialog width + destructive footer) và UI-22 (~417 chuỗi i18n);
  lý do và residual width literal ghi trong `VERIFICATION.md`.

## 5. False positive đã loại

- Welcome cards không keyboard accessible — **sai**: `Sense::click()` gồm FOCUSABLE,
  có Enter/Space + focus ring.
- Zero-size node trong dialog — hidden focus anchor cố ý (`dialog/ui.rs:247-272`).
- Fixture xấu — dùng để test scanner.
- Select chung không có keyboard — **sai**: có Arrow/Enter/Escape; chỉ picker bespoke
  mới thiếu.

## 6. Evidence gap

1. Không capture được menu mở: hook dùng `egui::Popup::open_id` trong khi Select giữ
   state ở temp data. Fix hook nằm trong wave này.
2. Table workspace và một số fixture trả `unsupported`/Welcome — chưa xác nhận visual.
3. Chưa chạy locale vi/ja, 800×600, scale factor 125/150%.

## 7. Thứ tự sửa

1. **P1:** explorer toolbar, composer order, schema picker scroll.
2. **Select component:** layout-neutral, theme bắt buộc, disabled, draft selection +
   Escape, menu row truncate, option semantics, label contract.
3. **Call site:** engine theme, toolbar chip/row-cap i18n, sidebar width, form size,
   chart wrap.
4. **Layout còn lại:** results toolbar, history rail, new-connection scroll, settings
   offset, status bar, dark contrast, diagram names.
5. **Hoãn có ghi chú:** sweep ~417 chuỗi i18n và refactor 13 dialog width (rộng, không
   cùng bản chất Select); ghi rõ trong VERIFICATION nếu chưa làm.
6. Gates + capture lại đúng command, đúng số đo; cập nhật ledger.
