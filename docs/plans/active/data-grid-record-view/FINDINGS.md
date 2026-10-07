# Research: dải nút dọc và Single Record View

Ngày nghiên cứu: 2026-10-06. Source baseline: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, branch `main`; source files inspected had no local modifications. Phần §1–8 là research snapshot trước triển khai; cập nhật implementation ở §9.

## 1. Kết luận

Đề xuất thêm dải điều khiển rộng khoảng 36–40 logical px, ngay bên trái cột số thứ tự `#`. Đây là UI chrome, không phải một cột dữ liệu; không tham gia sort, resize, copy hay export. Nút Record nằm ở đáy dải, có tooltip và trạng thái active. Có nút Grid để quay lại.

Chọn một row → Record → toàn bộ vùng bảng chuyển thành danh sách `Field | Value` của đúng row đó. Không mở tab Query hay dialog mới. Không giữ grid bên cạnh như inspector hiện tại. Độ rộng là đề xuất, cần xác nhận qua capture native.

Ví dụ với row 4 trong ảnh người dùng:

| Field | Value |
|---|---|
| id · INTEGER · PK | 4485 |
| first_name · TEXT | Alex |
| last_name · TEXT | Morgan |
| email · TEXT | user_1278@example.com |
| created_at · DATETIME | Hiển thị đầy đủ giá trị từ result; ảnh đang cắt nên không suy đoán phần bị khuất |

Type/PK là metadata phụ cùng ô Field, tránh thêm một cột Type riêng khi không cần. Header nhỏ: `Record · row 4`. Số row phải dùng offset và vị trí đang hiển thị, không dùng raw array index.

## 2. DBeaver thực sự làm gì

- Record view chuyển tên cột thành các hàng và hiển thị giá trị của record hiện tại; chọn cell rồi chuyển Record. Có điều hướng record. [Data view and format](https://dbeaver.com/docs/dbeaver/Data-View-and-Format/).
- Data Editor có left sidebar chứa Grid/Record và các presentation khác; Save/Cancel nằm ở toolbar dưới. [Data Editor](https://dbeaver.com/docs/dbeaver/Data-Editor/).
- Hai trang chưa thống nhất vị trí Record: trang Data Editor ghi left sidebar, trang Data view and format ghi bottom toolbar. Chưa chạy DBeaver để đối chiếu phiên bản; không khẳng định layout pixel hiện tại. Nút neo đáy rail là đề xuất theo yêu cầu người dùng.
- DBeaver có hỗ trợ layout nhiều record, nhưng phạm vi đề xuất DB Pro chỉ là một record.

## 3. Source evidence DB Pro

Mọi đường dẫn/line dưới đây thuộc SHA baseline ở đầu báo cáo.

| Bằng chứng | Hiện trạng và hệ quả |
|---|---|
| `crates/ui/src/result_grid_view.rs:58-74` | Table Data không render result toolbar; toolbar khác có nút Record. Giải thích vì sao ảnh không có nút này. |
| `crates/ui/src/result_grid_toolbar_view.rs:129-138` | Record toggle `record_inspector_open`, tooltip mô tả inspector panel. |
| `crates/ui/src/result_grid_view.rs:74-105` | Inspector chia vùng ngang với grid, bề rộng tối đa 320px/45%. Đây là xem song song, chưa đáp ứng single-record presentation. |
| `crates/ui/src/result_grid_edit.rs:255-289` | Inspector lấy selected_row/selected_cell và row từ UiQueryResult; có thể reuse selection và Inspect action. |
| `crates/ui/src/result_grid_record_surface_view.rs:24-85` | Inspector giới hạn scroll 220px, rút gọn giá trị hơn 80 ký tự, đọc raw row; chưa đủ cho full Record view. |
| `crates/ui/src/result_grid_cell.rs:32-33` | Grid ưu tiên staged_cell_value; Record mới phải dùng cùng effective value để tránh hiển thị khác dữ liệu chưa Save. |
| `crates/ui/src/table_data_state.rs:353-405` | Row/cell selection đã có; selected_rows hỗ trợ nhiều row. Không tạo selection state thứ hai. |
| `crates/ui/src/result_grid_body_view.rs:52-83` | Scroll ngang bao header/body, row virtualization nằm trong scroll dọc. Rail phải nằm ngoài cả hai để luôn cố định. |
| `crates/ui/src/table_data_view.rs:44-104` | Table Data có toolbar/grid/footer và placeholder path riêng. Rail cần có vị trí ổn định cả loading/error/empty. |
| `crates/ui/src/table_events.rs:174-184`, `crates/ui/src/query_result_events.rs:27-40` | Load result có đường reset selection; Table load giữ selection khi staged changes tồn tại. Không được coi index cũ luôn là cùng record sau reload. |
| `docs/architecture/system-overview.md:5-14` | UI native egui, command/event bridge; presentation switch có thể chỉ xử lý local result. |

Đã kiểm tra plan Core Component Style System: không tìm thấy scope dành riêng cho single-record presentation. Chưa kiểm tra remote PR inventory; phải kiểm tra trước khi triển khai/publish nếu cần tránh trùng work.

## 4. Hợp đồng tương tác đề xuất

1. Grid mặc định. Chọn row qua gutter hoặc cell đều hợp lệ; cần đúng một selected row. Khi chưa chọn hoặc chọn nhiều row, Record disabled kèm lý do; không tự chọn một record tùy ý.
2. Record hiển thị đúng row trong result hiện có; không phát LoadTableData mới chỉ để đổi presentation.
3. Quay Grid giữ row/cell selection, column widths, sort/filter và scroll. State dùng chung; không clone toàn bộ result vào Record state.
4. Record bản đầu để xem/copy/Inspect, không thêm inline edit. Giá trị hiển thị lấy staged override như Grid, giữ NULL khác chuỗi rỗng; long text wrap hoặc mở Inspect để xem hết, không cắt im lặng.
5. Nếu đang gõ cell khi bấm Record, dùng đường validation/staging hiện có trước khi chuyển; lỗi validation giữ Grid để người dùng sửa. Không gọi Save hay gửi mutation do chuyển mode. Cần kiểm tra implementation của staging trước khi viết patch.
6. Loading/error/empty và result replacement: không hiển thị row cũ từ index cũ. Nếu selection hết hợp lệ thì quay Grid; chỉ giữ Record khi xác minh được identity của record trong result mới.
7. Hiển thị row number theo vị trí visible sau sort/filter cộng page offset. Không dùng raw zero-based index của inspector hiện tại.
8. Chưa thêm Prev/Next giữa record hay shortcut Tab trong scope đầu; đây là khả năng DBeaver có, không phải yêu cầu hiện tại. Tab vẫn dùng cho focus accessibility.
9. Footer Save/Discard/Refresh và pagination giữ flow hiện tại. Rail có accessible label, tooltip, focus và active state; semantic colors từ DbProTheme.

## 5. Phương án và phạm vi sửa

| Phương án | Đánh giá |
|---|---|
| Chỉ expose nút inspector hiện có | Ít code nhưng vẫn giữ grid và chỉ có preview; không đạt yêu cầu. |
| Dialog record | Làm được nhưng mất tương tác chuyển mode tại grid; không phù hợp yêu cầu dải dọc. |
| Rail + Grid/Record presentation dùng chung state | Khuyến nghị; đúng hành vi và reuse dữ liệu/selection/value formatting hiện có. |

Điểm sửa dự kiến: composition `table_data_surface_view.rs`/`table_data_view.rs`, presentation state và routing trong `result_grid_view.rs`, record renderer hiện có hoặc renderer tách đúng vai trò; reuse staged_cell_value và cell inspector. Không biến `record_inspector_open` thành hai nghĩa. Chưa chốt số file hoặc tạo abstraction mới trước khi implementation review.

Severity: yêu cầu presentation thiếu là P2 UX. Các điểm raw/staged mismatch và selection sau reload là rủi ro cần chặn trong implementation; chưa chứng minh wrong database mutation nên không gán P1 cho pattern chưa kiểm chứng.

## 6. Provider và proof boundary

| Provider | Đề xuất support | Source/automated | Live UI/provider |
|---|---|---|---|
| SQLite | Có: đọc result đã load, không cần PK để xem | Source path chung; implementation chưa có | Pending |
| PostgreSQL | Có: đọc result đã load, không cần PK để xem | Source path chung; implementation chưa có | Pending |

Tests/build chưa chạy vì chưa thay đổi code. Ảnh người dùng là visual reference, không chứng minh native interaction của feature mới. Sau triển khai cần kiểm tra sort/filter/offset, một/nhiều/không selected row, giá trị dài/NULL/JSON/BLOB, staged changes và result replacement. Capture native ở 1280×800, 1440×900, 1920×1080, normal/loading/error/empty. Provider evidence phải riêng cho SQLite/PostgreSQL.

## 7. Learning pass

Bài học reusable: Record presentation và side inspector là hai vai trò khác nhau; chuyển presentation phải dùng cùng selection và effective staged value như Grid. Ghi tại report dự án này; không ghi global memory vì chưa có yêu cầu cập nhật memory.

## 8. Tổng kết bằng tiếng Việt

Đã xác minh DBeaver và code DB Pro. Đề xuất rail bên trái cột `#`, Record neo đáy và thay vùng grid bằng một record. Có nền tảng selection/inspector để reuse nhưng không thể chỉ bật inspector hiện có. Scope này sau đó được owner cho triển khai; xem cập nhật §9 và VERIFICATION.md.


## 9. Implementation and owner simplification

Baseline SHA: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`; implementation is an uncommitted main patch. Exact patch and artifact hashes are recorded in evidence/manifest.json.

- Fixed vertical rail beside the index column; Record at the bottom, enabled for one selected row.
- Owner requested a simple shared Table. Final Record uses exactly two columns, Field and Value. No custom heading, type sublabel, or Inspect column. Earlier research recommendations above are historical, superseded by this instruction.
- Click field name copies the name; click value copies the full effective value, including staged overrides. NULL copies NULL; empty text reports an intentional no-op because the existing egui-winit clipboard bridge ignores empty text.
- Mode switch stages a valid active edit locally; an invalid editor stays open. It sends no SQL and does not save changes. Grid selection is preserved; hidden or reloaded selections exit Record.
- Both PostgreSQL and SQLite share this presentation over UiQueryResult. Native captures use deterministic fixtures; live provider verification remains pending.
- Full-height rail uses horizontal_top; a normal horizontal container initially clipped content. Reuse Table rather than custom compound record rows for this simple field/value presentation.
- Focused headless click tests establish egui clipboard output, not physical OS clipboard behavior.
- Two workspace PostgreSQL connection-form tests fail on both the patch and exported baseline: new_postgresql_connection_defaults_to_tls_require and explicit_disable_selection_is_preserved_on_submit. Baseline run: 182 pass / 2 fail.
- Native perf invocation also has an inherited libtest --quick incompatibility; quick scan runs its supported checks. No performance improvement claimed.

Learning pass: reusable local lessons recorded here. Global memory not modified, as no memory write was authorized.

## 10. Footer balance follow-up

Owner supplied active Save/Discard/Refresh screenshot. Baseline SHA remains `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, plus uncommitted implementation patch. Refresh used a legacy 24px button; Save/Discard used shared Sm buttons (28px) with different icon/text metrics. Refresh now uses the same shared Sm tokens; Discard uses Secondary and Save remains primary. Footer action spacing is SPACE_XS (4px). Action dispatch, enabled conditions and database mutation policies are unchanged. Added only a deterministic changed-data capture state. No new tests for this reversible styling change; existing behavior tests retained. Reusable lesson: toolbar siblings should use the same component/size rather than mixing legacy and shared buttons. Global memory not modified.
