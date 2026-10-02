# Audit spacing / padding toàn UI DB Pro

Ngày: 2026-10-02. Source SHA: `52cdf982e748a0f57a1dae3b4493316268e22318`. Phạm vi: UI native egui; spacing, padding, sizing, alignment và khả năng co giãn theo container. Đơn vị dưới đây là **logical points của egui**; ảnh native macOS thu ở scale 2×.

## Kết luận

**7 phát hiện P2**, không tìm thấy P0/P1 trong phạm vi này. Ba nguyên nhân chính: spacing của layout cha truyền vào component con, công thức chiều rộng bỏ sót khoảng cách ngầm của egui, và chiều cao control chưa theo cùng một hợp đồng.

Báo cáo là kết quả audit; các phát hiện chưa được sửa. Viền xanh trên workspace tab đã bỏ ở SHA trên và đã push.

## Bằng chứng và phạm vi kiểm tra

- Inventory quét 559 file Rust UI production; bỏ file test và phần `#[cfg(test)]`.
- 976 dòng spacing trong 162 file; 431 dòng chứa giá trị số trực tiếp theo pattern của scanner. Đây là danh mục để review, **không phải 431 lỗi**.
- 307 dòng sizing/allocation và 251 khai báo contract padding/gap/margin/height.
- Probe egui đo 5 control trong 3 context: spacing theme `[8,4]`, Gallery `[0,4]`, ResponsiveGrid gap 16 `[16,0]`.
- 24 ảnh native: 13 Gallery section; Query, Table, Settings, Diagram, New Connection, Connection Error, Loading; thêm Buttons/Form Error ở hai kích thước lớn.
- 20 ảnh đầu tại 1280×800 logical → 2560×1600 physical. Các ảnh 1440/1920 có chiều cao bị màn hình clamp: chi tiết trong VERIFICATION.md.
- Hai overview và toàn bộ ảnh gốc nằm trong `evidence/`. Ảnh trên vùng đang nhìn thấy không chứng minh phần cuộn chưa mở hoặc mọi trạng thái popup.

## Phát hiện đã xác nhận

### SP01 — [P2] ResponsiveGrid làm thay đổi spacing bên trong control

**Vị trí:** `crates/ui/src/components/responsive_layout/ui.rs:153-179`; caller `component_gallery_inputs.rs:108`.

Grid đặt `item_spacing.y = 0`, rồi `item_spacing.x = metrics.gap` ở row. Child cell thừa hưởng cả hai thay đổi, không được trả lại spacing nội dung ban đầu.

**Số đo:** label `Host` đến dấu `*` bình thường **12**, trong Gallery **4**, trong Grid gap 16 **20**. Grid cell spacing là `[16,0]`, thay vì theme `[8,4]`. Form và metric card dùng cùng primitive đều bị ảnh hưởng.

**Ảnh hưởng:** tăng gap giữa cột cũng làm dấu required và icon trong field xa hơn; label/helper bên trong lại mất khoảng cách dọc ngầm. Layout thay đổi mật độ control dù không đổi component.

**Sửa đề xuất:** lưu spacing nội dung của parent trước khi điều chỉnh grid; grid giữ gap ở row/column allocation, cell khôi phục spacing nội dung. Test gap giữa cell và gap bên trong cell độc lập. Không sửa từng form bằng `add_space` bù trừ.

### SP02 — [P2] Input có clear button nở rộng vượt width yêu cầu

**Vị trí:** `crates/ui/src/components/input/text.rs:146-169`.

`extra_width` trừ 24 cho clear button nhưng không trừ `item_spacing.x` giữa TextEdit và clear target. PasswordInput đã có phép trừ toggle + gap tại `password.rs:129` và không bị lỗi này trong probe.

**Số đo, width yêu cầu 300:** Input thường **300**; clearable Input có nội dung **308** ở theme thường, **316** trong Grid gap 16, **300** trong Gallery x-gap 0.

**Ảnh hưởng:** field lệch mép với sibling và có thể bị clip mép phải/nút clear. Gallery x-gap 0 che lỗi mà parent bình thường vẫn gặp.

**Sửa đề xuất:** reserve đúng clear target + gap thực tế, theo điều kiện nút thực sự hiển thị; giữ tổng frame width bằng width được resolve. Test empty/nonempty, enabled/disabled và spacing 0/8/16.

### SP03 — [P2] Select cũng vượt width do reservation không đủ

**Vị trí:** `crates/ui/src/components/select/handler.rs:68-77`, `select/ui/select.rs:103-132`, `select/config.rs:18`.

Text width trừ icon + explicit gap, nhưng row còn thêm `item_spacing.x` của egui trước icon. Công thức padding cũng dùng giá trị tổng 20 trong khi frame dùng button padding từ theme; thay đổi padding theme có thể làm sai reservation tiếp.

**Số đo, width yêu cầu 300:** Select **308** ở theme thường, **316** trong Grid gap 16, **300** trong Gallery x-gap 0.

**Ảnh hưởng:** combobox không thẳng mép với Input/Password, có nguy cơ clip border/chevron.

**Sửa đề xuất:** tính nội dung từ width trừ đúng frame margins và toàn bộ trailing reservation, hoặc đặt spacing row rõ ràng rồi reserve cùng giá trị. Test tổng bounds của trigger, không chỉ helper trả về một con số.

### SP04 — [P2] Chiều cao control không theo input contract

**Vị trí:** `crates/ui/src/tokens/component/input.rs:22-24`; `input/text.rs:123`, `input/password.rs:112`; `select/handler.rs:76`.

Contract có Input Default **38**, Sm **30**, nhưng Input renderer hiện không áp dụng hai height này. Input lấy chiều cao từ TextEdit/horizontal layout; Password từ eye button; Select từ content và margin riêng.

| Control | Cao thực tế trong probe | Width bình thường |
| --- | ---: | ---: |
| Input | 32 | 300 |
| Input + clear | 32 | 308 |
| Password | 34 | 300 |
| Select | 36 | 308 |

**Ảnh hưởng:** top/bottom cạnh và content center không thống nhất khi xếp các control thành hàng/cột. Token khai báo không phản ánh geometry đang chạy.

**Sửa đề xuất:** thống nhất một contract theo density/role rồi áp dụng thực sự ở shared controls. Giữ toolbar compact khác form nếu có chủ đích. Dùng 38 cho form standard hiện có trong token là một phương án cần đối chiếu ảnh trước khi áp dụng; không tăng mọi control lên cùng một size.

### SP05 — [P2] Gallery shell truyền horizontal gap = 0 vào toàn nội dung

**Vị trí:** `crates/ui/src/component_gallery_view.rs:228-254`; ví dụ caller `component_gallery_surfaces.rs:22-74`.

`ui.spacing_mut().item_spacing.x = 0` phục vụ navigation/detail sát nhau, nhưng content cũng thừa hưởng zero gap. Các demo bù bằng `add_space(6/10/32)`; Alert section đã tự phục hồi gutter giữa card, là dấu hiệu ownership spacing phân tán.

**Bằng chứng:** probe context Gallery cho required gap 4 và width Select/clearable 300, trong khi theme thường cho gap 12 và width 308. Ảnh `gallery-buttons.png`, `gallery-feedback.png` cho thấy nhiều control row phải dựa vào gap cục bộ hoặc dính sát nhau.

**Ảnh hưởng:** cùng component được preview với spacing khác môi trường production; Gallery khó dùng làm chuẩn xác nhận UI và có thể che lỗi width reservation.

**Sửa đề xuất:** giới hạn zero gap ở shell navigation/divider/detail allocation. Trả spacing nội dung về spacing parent trước khi render header, card, row và form. Sau đó xử lý reservation ở SP02/SP03 để tránh lộ overflow do đổi context.

### SP06 — [P2] Feedback có hàng fixed width làm cột spinner bị ép/clipped

**Vị trí:** `crates/ui/src/component_gallery_feedback.rs:113-181`.

Hàng progress dùng cột 320 + gap 32 + cột 280 + gap 32, rồi cột spinner trong cùng `ui.horizontal`. Hai cột đầu/gap đã cần **664 points**, chưa kể spinner/card/container padding. Không có chuyển layout theo available width.

**Ảnh native:** `evidence/gallery-feedback.png` tại 1280×800. “Animated Vector Spinner” xuống dòng gần từng chữ ở mép phải; phần “Sync…” và các hàng shortcut/toast cũng có content ra ngoài vùng nhìn thấy. Trạng thái này không chỉ là khác biệt token.

**Ảnh hưởng:** demo bị cắt và chiếm chiều cao bất thường, không thể đánh giá component trong pane thông thường.

**Sửa đề xuất:** dùng layout theo chiều rộng local container, chuyển 3→2→1 cột với min content width; giữ gap bằng semantic token. Áp dụng cùng quy tắc cho hàng shortcut/toast có nội dung dài; kiểm tra chiều rộng pane thay vì chỉ window.

### SP07 — [P2, systemic] Spacing relationship và density còn phân tán

**Vị trí:** `tokens/primitive.rs:14-37`, `theme.rs:460-466`; inventory `evidence/spacing-inventory.csv`. Hotspot: `schema_workbench_form.rs`, Gallery surfaces/feedback, Query support và Connection advanced panels.

Scale chung đã có 2/4/8/12/16/20/24/32… và các role icon/label/form/section. Tuy nhiên nhiều view còn dùng số trực tiếp; implicit `item_spacing` + explicit `add_space` chưa có hợp đồng chung. Ví dụ `Label` dùng `LABEL_HELPER_GAP = 4` cho dấu `*` nhưng gap thực tế ngoài grid là 8 + 4 = 12.

**Ảnh hưởng:** đổi một token chưa đủ để chỉnh toàn bộ mật độ; cùng giá trị 8/12/16 có thể vừa là padding vừa là gap cộng thêm. Các contributor phải đoán parent style trước khi chỉnh layout.

**Sửa đề xuất:** xác định ownership theo quan hệ, chuyển từng nhóm đã xác nhận sang semantic token và đo actual bounds. Các số component-owned có lý do quang học/geometry được giữ; không coi mọi 6/10/14 là lỗi và không replace tất cả số bằng token máy móc.

## Những phần nên giữ

- Primitive → Semantic → Component đã tồn tại, không cần framework spacing mới.
- Container clamp gutter/width theo parent; ResponsiveGrid chọn số cột theo local width. Cần sửa inheritance, không thay thuật toán sizing đúng.
- PasswordInput reserve toggle kèm actual gap; probe cho width ổn định 300 trong cả ba context.
- Input chrome đã paint inset trong allocation; sửa width reservation sẽ giúp nguyên tắc này có hiệu lực ở field clearable.
- Table compact/default, toolbar, dialog, sheet và card có vai trò density khác nhau. Chênh padding không tự động là lỗi.
- Alignment header/gutter của Gallery phần chính nhìn nhất quán trong ảnh; tab đang chọn vẫn nhận biết sau khi bỏ đường xanh trên cùng.

## Density / spacing contract đề xuất để sửa tiếp

| Quan hệ / role | Quy tắc đề xuất |
| --- | --- |
| Spacing scale | Giữ scale hiện có; 2 points cho optical micro gap khi cần |
| Icon ↔ text | Gap thực tế 8; component sở hữu, không cộng thêm ngầm từ parent |
| Label ↔ control | Gap thực tế 4 hoặc một token được chọn thống nhất |
| Helper/error ↔ field | Gap thực tế 4; không phụ thuộc grid gap |
| Sibling controls | 8–12 theo toolbar/form role |
| Form columns/rows | 16; độc lập với spacing nội dung cell |
| Card content | 12 hiện tại; header/body/footer theo role |
| Page/pane gutter | 16 cho workbench; 24/32 cho reading/settings khi có chủ đích |
| Dialog / sheet | Giữ role 24×20 và 16, đối chiếu header/footer alignment |
| Heights | Một contract cho Input/Password/Select cùng density; toolbar và table giữ role riêng |

Đây là đề xuất audit, chưa thay token hay layout production.

## Thứ tự sửa

1. `/layout`: sửa ownership spacing của Grid/Gallery và reservation Input/Select cùng một pass; test parent gap 0/8/16, narrow container, empty/nonempty.
2. `/layout`: áp dụng actual height contract cho field cùng density; đối chiếu top/bottom edges và baseline.
3. `/adapt`: Feedback chuyển cột theo width local và wrap hàng dài; capture lại 1280/1440/1920 cùng pane hẹp.
4. Chuẩn hóa semantic relationships từng nhóm có bằng chứng; giữ exception đã được giải thích.
5. `/polish`: đối chiếu ảnh light/dark, normal/error/disabled/loading; audit lại spacing sau sửa.

## Giới hạn của audit

Inventory là lexical scan, không phải AST proof về mọi numeric literal hay mọi path gọi. Probe xác nhận geometry của case đã chạy; chưa đo mọi i18n/font scaling/state. Native ảnh bao phủ các viewport đang nhìn thấy; popover/dialog của Gallery chưa mở tương tác và nội dung bên dưới scroll không được đánh giá bằng ảnh. Query/Table/Settings/Diagram dùng fixture native, không phải xác nhận live provider. Không chấm tổng accessibility, typography hay frame performance vì phạm vi này là spacing.
