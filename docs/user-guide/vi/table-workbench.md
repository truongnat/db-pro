# Table Workbench (Không gian làm việc bảng)

Table Workbench là màn hình làm việc mở ra khi bạn chọn một bảng trong DB Pro. Nó gồm lưới dữ liệu với cơ chế sửa an toàn theo kiểu "staging", các tab xem cấu trúc và siêu dữ liệu, phân tích cột, xem DDL, cùng ba công cụ cấp schema đi kèm: sơ đồ ER, Schema Compare và Schema Workbench (lập kế hoạch → xem trước → áp dụng thay đổi đối tượng).

## Mở một bảng

- Nhấp vào bảng trong Explorer để chọn; workspace của bảng mở thành một tab.
- Nhấp chuột phải vào bảng trong Explorer để xem menu đầy đủ:

| Mục menu | Tác dụng |
|---|---|
| View Data | Mở workspace ở tab **Data** |
| View Structure | Mở workspace ở tab **Structure** |
| Modify Table (Alter)... | Mở Schema Workbench với kế hoạch alter cho bảng |
| Drop Table... | Mở luồng xác nhận/kế hoạch xóa bảng |
| Generate SQL: SELECT * / INSERT / UPDATE / DELETE | Tạo tab SQL editor điền sẵn câu lệnh |
| View DDL / CREATE Script | Mở workspace ở tab **DDL** |
| Copy Qualified Name / Copy Table Name | Sao chép `schema.table` hoặc `table` vào clipboard |
| Ask Agent about table | Mở trợ lý AI với ngữ cảnh là bảng này |
| Refresh Schema (F5) | Nạp lại schema của bảng |

## Thanh tiêu đề và các tab

Dải trên cùng hiển thị breadcrumb — `kết nối › schema › bảng` — và badge số dòng (`N rows`) nếu đã biết. Ba nút nằm bên phải:

| Nút | Tooltip | Tác dụng |
|---|---|---|
| Ask Agent | Open AI Assistant with table context | Mở trợ lý AI với ngữ cảnh bảng này |
| New Query | Open SQL Editor for this table | Tạo tab truy vấn cho bảng |
| Refresh | Reload table metadata and rows | Tải lại cả cấu trúc lẫn trang dữ liệu |

Bên dưới là tám tab: **Data · Structure · Profile · Indexes · Foreign Keys · Constraints · Dependencies · DDL**.

## Tab Data

### Thanh công cụ

Thanh công cụ trên gồm các nút sửa dữ liệu, xuất file và trình soạn WHERE inline:

- **Add Row** — mở hộp thoại Insert Row. Bị vô hiệu (tooltip giải thích lý do) khi không thể insert cho kết quả hoặc kết nối hiện tại.
- **Export** — xuất các dòng *đang hiển thị* (xem bên dưới).
- Ô **WHERE** — trình soạn điều kiện `SELECT`. Gõ phần thân của mệnh đề `WHERE` (ví dụ `amount > 100`); gợi ý tên cột tự bật lên (dùng ↑/↓, Enter hoặc Tab để chọn, Esc để đóng). Nhấn **Run** hoặc Enter để chạy `SELECT … WHERE <điều kiện>`; để trống sẽ chọn toàn bộ dòng.
- **Run** bị chặn khi còn thay đổi đang staged — thông báo yêu cầu áp dụng hoặc hủy trước. Câu lệnh phá hủy (destructive) cần xác nhận trước khi chạy.

### Các trạng thái hiển thị

| Chỉ báo | Ý nghĩa |
|---|---|
| Chip `N pending · +i ~u -d` | Có thay đổi staged; nhấp để mở hộp thoại Pending Changes |
| `Read-only` | Kết nối hoặc kết quả không cho phép ghi |
| `Table has no primary key; safe row editing is unavailable.` | Không sửa/xóa được; vẫn insert được. Dùng **Use column as row id** trong menu ô để chọn cột định danh thay thế |
| `N rows selected` | Đang chọn nhiều dòng |
| `CONFLICT · Mutation #n failed` | Một thay đổi thất bại; hover để xem lỗi. Xuất hiện các nút **Reload Row**, **Discard Local Change**, **Resolve Conflict**, **Retry** |

### Bộ lọc đang áp dụng

Các bộ lọc phía server hiển thị thành chip dưới toolbar (`column = value`, `column contains x`, `IS NULL`, …). Mỗi chip có nút **×** để gỡ; **Clear all** xóa toàn bộ. Tạo bộ lọc qua menu tiêu đề cột (**Add Filter** — chuẩn bị sẵn một draft) hoặc menu chuột phải của ô (**Filter by this value**).

### Lưới dữ liệu

- Nhấp tiêu đề cột để sắp xếp; chuột phải để mở menu: **Sort Ascending/Descending, Clear Sort, Add Filter, Copy Column Name, Copy Column Values, Move Column Left/Right, Reset Column Order, Reset Column Widths, Hide Column, Show Columns, Reset Layout, Auto Size**.
- Kéo mép cột để đổi độ rộng; thứ tự/ẩn cột là layout theo từng bảng, reset được từ menu tiêu đề.
- Chọn nhiều dòng qua gutter bên trái; badge số dòng đang chọn hiện trên toolbar.

### Sửa dữ liệu

Mọi thay đổi được *stage* trước — chưa ghi vào database cho tới khi bạn nhấn **Save**.

- Nhấp đúp vào ô (hoặc **Enter/F2**, hoặc chọn **Edit Cell** trong menu) để sửa inline. Trình sửa đổi theo kiểu cột: ô nhập thường, dropdown cho boolean/enum, và bộ chọn ngày cho date/time. **Enter** chốt ô, **Tab** chốt và sang ô phải, **Esc** hủy.
- Cột chỉ đọc (ví dụ generated column) hiển thị **Read-only Column** trong menu kèm lý do.
- **Set to NULL**, **Revert Cell**, **Revert Row / Undo Delete**, **Duplicate Row**, **Delete Row** là các thao tác còn lại trong menu ô.
- Thay đổi gom vào ChangeSet. Lưu bằng nút **Save** hoặc `Ctrl+S` (`⌘S` trên macOS); hủy bằng **Discard** hoặc `Ctrl+Z`. Hủy khi có nhiều hơn một thay đổi sẽ hỏi xác nhận.

### Menu chuột phải của ô

| Nhóm | Mục |
|---|---|
| Sao chép | Copy Cell Value (`Ctrl+C`), Copy Row (TSV) (`Ctrl+Shift+C`), Copy Selected Rows, Copy Selected Rows with Headers, Copy Selected Rows as JSON / Markdown / INSERT SQL, Copy Row as JSON, Copy Row as CSV |
| Sửa (kết quả writable) | Edit Cell, Set to NULL, Revert Cell, Revert Row / Undo Delete, Duplicate Row, Delete Row (Delete/Backspace) |
| Điều hướng & lọc | Use column as row id, Open referenced row (đi theo FK tới bảng được tham chiếu, lọc đúng dòng), Filter by this value, Sort Ascending, Sort Descending |

### Chân trang

Footer chứa **Save** (`Ctrl+S`), **Discard** (`Ctrl+Z`), **Refresh** (`F5`), nhãn khoảng dòng, điều hướng trang (**First · ‹ · › · Last**) và bộ chọn số dòng/trang (50/100/250/500/1000 — đổi sẽ quay về trang đầu). Điều hướng bị khóa khi còn thay đổi staged.

### Các hộp thoại

- **Insert Row** — trường nhập theo từng cột với badge PK/FK, kiểu dữ liệu, dấu `*required`/`nullable` và biểu thức mặc định. Thao tác nhanh: **Clear All**, **Fill Required**, **Generate All** (sinh dữ liệu mẫu), và theo trường: **NULL**, **UUID/Now/Gen**, nút xóa. **Stage Insert** thêm dòng vào ChangeSet; chỉ thực sự insert khi Save. Cột bị hạn chế ghi hiển thị `Read-only — <lý do>`.
- **Pending changes** — liệt kê toàn bộ thay đổi staged, gom theo dòng (`Row PK: […]` hoặc `New row · temporary identity #n`), với **Revert Cell**, **Revert Row**, **Remove Insert**, **Discard All**.
- **Unapplied Changes** — hiện khi điều hướng sẽ làm mất thay đổi staged: **Apply Changes**, **Discard Changes**, hoặc **Cancel**.
- **Row Conflict Resolution** — xuất hiện khi dòng trong database bị đổi sau khi bạn staged. Hiển thị khóa chính và bảng diff 3 chiều cho từng cột đổi (Original Baseline / Local Staged (Mine) / Database Current / Status). Xử lý bằng **Keep Mine (Overwrite DB)**, **Use Database (Discard Mine)**, **Reload & Retry**, hoặc **Discard Local Change**. Xung đột xóa có **Discard Staged Delete** và **Reload & Retry Delete**.
- **Export Results** — chọn định dạng (CSV, TSV, JSON, Markdown, INSERT, COPY, Excel), nhập đường dẫn file, nhấn Export. Chỉ trang đang tải được xuất. File đã tồn tại sẽ hỏi **Overwrite / Keep existing file**.
- **Value inspector** — mở qua nút **Inspect** từng trường trong record view. Chế độ: Raw / Pretty / Tree cho JSON, Raw / Hex / Base64 cho bytea, Raw / Pretty cho còn lại. Ô writable có **Apply to ChangeSet**; bytes lưu được qua **Export bytes…**.
- **Record view** — đổi chế độ hiển thị để xem một dòng được chọn dưới dạng danh sách trường (`Record · row N`), mỗi trường có nút **Inspect**.

### Trạng thái trống và lỗi

`Loading data for <table>…` khi đang tải; `<table> has no rows` (gợi ý Add Row) khi trống; `Data for <table> could not be loaded` kèm lỗi và nút **Retry** khi thất bại.

## Tab Structure

Hiển thị các badge số liệu (số cột, `PK: …`, số index, số foreign key, check constraint, dependency, số dòng) và bảng **Columns** có tìm kiếm (`Search columns or types…` lọc theo tên, kiểu, default):

| Cột | Ghi chú |
|---|---|
| # | Vị trí thứ tự |
| Column Name | Icon phân biệt PK (chìa khóa), FK (liên kết) hay cột thường; hover xem `IDENTITY`/`GENERATED`/`UNIQUE`/collation |
| Data Type | Font monospace; hover xem kiểu đầy đủ |
| Nullable | Badge `NULL` / `NOT NULL` |
| Key | Badge `PK` / `FK` / `UNIQUE` |
| Default Expression | Giá trị mặc định; hover khi bị cắt |
| Actions | **Alter** — mở planner ALTER COLUMN trong Schema Workbench; bị tắt kèm tooltip lý do khi driver không hỗ trợ |

Đang tải hiển thị `Loading table structure…`; lỗi hiển thị `Table structure could not be loaded` kèm chi tiết.

## Tab Profile

Phân tích **trang dữ liệu đang tải** (badge `Page sample`) — đây là mẫu giới hạn, không phải quét toàn bảng. Nếu chưa có dữ liệu, tab yêu cầu mở Data trước. Mỗi cột hiển thị: **Completeness** (% đầy + tỷ lệ NULL), **Distinct** (số lượng · %), **Pattern** (badge phân loại), **Range · Min / Max**, và **Avg · Sum** khi cột toàn số.

## Tab Indexes

Danh sách index có tìm kiếm (`Filter indexes…`): **Index Name** (index unique có icon dấu tích; hover xem định nghĩa DDL đầy đủ), **Indexed Columns**, **Method**, **INCLUDE**, **Predicate**, và **Status** (`PRIMARY KEY` / `Enforces uniqueness` / `Active index`).

## Tab Foreign Keys

Danh sách khóa ngoại đi ra, có tìm kiếm: **Constraint Name**, **Source Columns**, **Target Table**, **Target Columns**, và **Action** (`UPDATE <rule> · DELETE <rule>` cùng `DEFERRABLE`/`DEFERRABLE INITIALLY DEFERRED`; hover xem tùy chọn `MATCH`). Icon liên kết **mở bảng được tham chiếu** trong workspace riêng.

## Tab Constraints

Các dòng ràng buộc lọc được theo loại — nút tab **All / Primary Key / Unique / Check / Not Null** cùng ô lọc văn bản. Cột của bảng: **Type** (badge), **Constraint Name**, **Target Columns / Expression**, **Details**. Bao gồm PRIMARY KEY, UNIQUE index, CHECK constraint (kèm biểu thức SQL) và NOT NULL từng cột.

## Tab Dependencies

Quan hệ phụ thuộc hai chiều với tab lọc hướng (**All / Depends On (Outgoing) / Depended By (Incoming)**), ô tìm kiếm và bộ đếm `matching of total`. Cột: **Direction** (`DEPENDS ON` / `DEPENDED BY`), **Kind** (`TABLE`, `VIEW`, `FK`, `TRIGGER`, `FUNCTION`, `SEQUENCE`), **Object Name** (nút link mở dependency kiểu table trong workspace), **Relationship Details**.

## Tab DDL

- Toolbar: badge `DDL SCRIPT · CREATE TABLE` với **Refresh** (tạo lại từ schema mới nhất), **Open in Query** (mở script trong SQL editor để chạy), và **Copy** (sao chép vào clipboard).
- Ô script là bản xem trước có thể sửa, nhưng **Apply DDL** cố ý bị tắt (`Not enabled in v0.1 — run DDL with Open in Query`) — để thực thi DDL đã chỉnh, hãy dùng Open in Query.
- Trạng thái tải/lỗi giống các tab khác (`Loading DDL for <table>…`, `DDL for <table> could not be loaded`).

## Sơ đồ ER

Mở activity/tab **Diagram** để xem bản đồ quan hệ của schema đã tải.

- **Canvas**: lăn chuột để zoom về phía con trỏ (hỗ trợ pinch), kéo vùng trống để pan, kéo thẻ bảng để đổi vị trí (vị trí lưu theo kết nối). Nhấp vào node để làm nổi bật quan hệ của nó, các phần khác mờ đi.
- **Toolbar**: badge `N tables` / `N relationships`, badge `Arranging…` khi đang tính layout.
- **Schema lớn**: ô tìm kiếm (`Find table or column…`) dựng bản đồ vùng lân cận thay vì vẽ tất cả. Chọn **1 hop / 2 hops** cho độ sâu lân cận, hoặc nhấn **Show all N tables** / **Focus search** để đổi chế độ. Khi không khớp, hiện nút **Clear search**.
- **Thanh zoom nổi** (góc phải trên): `−`, % zoom (nhấp để về 100%), `+`, reset view, **Fit** (fit toàn đồ thị hoặc tập đang lọc).
- **Minimap** (góc phải dưới): tổng quan toàn canvas; nhấp hoặc kéo để đổi tâm nhìn.
- **Icon toolbar**: **Design Mode** (pencil-ruler) bật/tắt panel draft; icon reset-layout xóa các vị trí thủ công đã lưu và dựng lại lưới.
- Trạng thái trống: `No map yet` khi chưa tải schema; `Arranging N tables…` khi đang tính; `No matching tables` khi tìm không khớp.

### Design Mode

Trình thiết kế schema dạng draft (tiêu đề `DESIGN DRAFT · ObjectMutationService plan · fingerprint-gated apply`):

1. Nhập `schema` + `table` rồi nhấn **Add draft table**.
2. Với mỗi bảng draft, thêm cặp `col`/`type` bằng **Add col**.
3. Thêm khóa ngoại bằng `fk name`, `from schema.table.col`, `to schema.table.col`, rồi **Add FK**.
4. **Preview mutation plan** hiển thị fingerprint, các hiệu ứng dự kiến và SQL sinh ra. **Apply (confirm)** thực thi; **Undo**/**Redo**/**Discard** quản lý draft.

Nếu schema thật đổi sau khi bạn mở Design Mode, cảnh báo `Live schema changed — re-open Design Mode or discard before apply` sẽ chặn việc apply.

## Schema Compare

Truy cập từ phần sidebar và dưới dạng workspace view.

- **Take Snapshot** — ghi lại schema đã tải. **Diff vs snapshot** / **Diff now** introspect lại và so sánh.
- Kết quả diff tóm tắt `N total difference(s) · Added · Changed · Removed`, rồi nhóm: bảng/view/routine chỉ có ở snapshot hoặc chỉ có ở hiện tại, và thay đổi cột/kiểu. `Schemas are identical. No drift detected.` khi giống hệt nhau.
- **Migration plan**: **Generate migration plan** sinh danh sách thao tác gắn với target `kết nối · schema` đang hoạt động, kèm badge rủi ro từng thao tác (`SAFE`, `MUTATING`, `DESTRUCTIVE`) và cảnh báo. DDL migration sinh ra có thể sao chép (**Copy SQL**). Nếu plan có thao tác phá hủy, phải tick **Confirm destructive operations (never auto-applied)** trước khi **Apply migration SQL**; nếu target ghi lại không còn khớp phiên hiện tại, thẻ `SAFETY LOCK — plan target mismatch` sẽ khóa nút apply.
- **Data compare**: nhập `target connection id`, `schema`, `table`, `key columns (comma)` rồi nhấn **Compare rows** để so mẫu theo khóa giữa hai kết nối. Kết quả hiển thị số liệu (`src/tgt · +added -removed ~changed =equal · truncated`), lọc theo `all / added / removed / changed`, kèm `DATA SYNC SQL PREVIEW` chỉ để xem trước.

## Schema Workbench

Mở từ menu chuột phải của bảng (**Modify Table (Alter)…**, **Drop Table…**) hoặc sidebar. Luồng làm việc **lập kế hoạch → xem trước → áp dụng**. Sidebar có 15 chế độ:

`Table / columns · Column alter · Views · Indexes · Constraints · Triggers · Sequences · Types / enums · Schema / database · Extensions · Comments · Partitions · Dependencies · Docs export · Execution history`.

- Mỗi chế độ có form **DEFINITION** (schema, tên đối tượng, designer riêng: lưới cột với `+ Add Column`/`+ UUID PK`/`+ Timestamps`, preset trigger, mặc định sequence, preset enum, preset query view, designer index/constraint).
- Các nút như **Plan create**, **Plan drop**, **Plan rename**, **Plan alter**, **Plan refresh**, **Plan comment** sinh DDL preview trực tiếp (`-- Click 'Plan create', 'Plan drop' or 'Plan rename' above to generate DDL` khi trống). **Apply DDL to Database** thực thi sau hộp thoại xác nhận; **Open in SQL Editor** đưa script sang tab truy vấn.
- **Dependencies** liệt kê các cạnh phụ thuộc đối tượng (From · Relation · To). **Docs export** sinh tài liệu schema (**Generate Markdown**, **Generate HTML**, **Open as query**). **Execution history** liệt kê các lần chạy DDL với **Clear history**, **Open in SQL editor** và **Copy SQL** theo mục.

## Phím tắt

| Phím | Ngữ cảnh | Tác dụng |
|---|---|---|
| Enter / F2 | Lưới (editable) | Bắt đầu sửa ô; Enter chốt |
| Tab | Đang sửa ô | Chốt và sang ô kế tiếp |
| Esc | Sửa ô / lưới | Hủy sửa / bỏ chọn |
| Delete / Backspace | Lưới (editable) | Stage xóa dòng |
| Ctrl/⌘+C | Lưới | Sao chép ô |
| Ctrl/⌘+Shift+C | Lưới | Sao chép các dòng đã chọn |
| Ctrl/⌘+A | Lưới | Chọn tất cả |
| Ctrl/⌘+S | Table data | Áp dụng thay đổi staged |
| Ctrl/⌘+Z | Table data | Hủy thay đổi staged |
| Ctrl/⌘+V | Lưới | Dán vào ô editable |
| F5 | Table data / explorer | Làm mới dữ liệu bảng / schema |
