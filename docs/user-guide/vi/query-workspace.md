# Workspace truy vấn (Query)

Workspace **Query** là nơi viết, chạy và kiểm tra SQL. Nó gồm trình soạn thảo
SQL, bộ chọn kết nối/schema, vùng output (kết quả, biểu đồ, thông báo, explain
plan, lịch sử) và thanh điều khiển transaction.

Mở bằng tab **Query** trong workspace, hoặc nhấn `Ctrl/Cmd+T` để tạo document
truy vấn mới. Nếu chưa có kết nối nào, `Ctrl/Cmd+N` sẽ mở hộp thoại New
Connection.

## Bố cục màn hình

Từ trên xuống dưới:

1. **Breadcrumb đường dẫn file** — hiện khi document được mở từ một file `.sql`.
2. **Thanh context** — chip kết nối/schema, nút Run/Stop, Explain, Format, bộ
   chọn giới hạn số dòng và menu "More actions" (⋯).
3. **Visual query builder** — panel tùy chọn, bật/tắt từ menu actions.
4. **Thanh transaction** — xuất hiện khi có transaction đang mở hoặc khi bạn
   bật nó.
5. **Panel tham số** — xuất hiện khi câu SQL có bind parameter.
6. **Trình soạn thảo** — vùng soạn SQL, có status bar riêng ở cuối.
7. **Dock output** — các tab Results / Chart / Messages / Explain / History.
   Có thể kéo đường chia để đổi kích thước, dock xuống dưới hoặc sang phải,
   và phóng to.

## Thanh context

| Điều khiển | Tác dụng |
|---|---|
| Chip kết nối/schema | Hiển thị `kết nối / schema`. Bấm để mở bộ chọn và đổi kết nối hoặc schema. Chấm xanh = đã kết nối, chấm đỏ cho môi trường `production`/`prod`, biểu tượng cảnh báo khi chưa kết nối. |
| **Run** (▶) | Chạy câu lệnh tại vị trí con trỏ, hoặc đoạn được bôi đen. Khi truy vấn đang chạy sẽ đổi thành **Stop** (■) nếu driver hỗ trợ hủy; nếu không sẽ hiện **Running**. |
| **Explain** | Chạy `EXPLAIN` cho câu lệnh hiện tại và mở tab Explain. |
| **Format** | Định dạng lại SQL (tương đương `Ctrl/Cmd+Shift+F`). |
| Giới hạn dòng | Dropdown (`100 / 500 / 1.000 / 100.000 rows`) giới hạn số dòng lấy về ở lần chạy tiếp theo. |
| **⋯** (More query actions) | Mở menu Query actions bên dưới. |

### Menu Query actions (⋯)

- **Run query** / **Run selection** — giống nút Run.
- **Run script** — chạy lần lượt toàn bộ câu lệnh trong document.
- **Format SQL** — giống nút Format.
- **Explain** — giống nút Explain.
- **Ask agent** — chuyển truy vấn sang Agent surface.
- **Save query** (`Ctrl/Cmd+S`) / **Save query as** (`Ctrl/Cmd+Shift+S`) — lưu
  vào query library; "Save as" sẽ hỏi tên và thư mục (tùy chọn).
- **Visual builder / Hide visual builder** — bật/tắt trình dựng SELECT trực quan.
- **Find in SQL** — mở overlay tìm kiếm trong editor (`Ctrl/Cmd+F`).
- **Show/Hide transaction controls** — bật/tắt thanh transaction.
- **Decrease / Increase font size** — đổi cỡ chữ editor (hiển thị theo px).
- **Generate SQL Prediction** — yêu cầu gợi ý AI ngay lập tức.
- **AI prediction** — chế độ `Off`, `Subtle` (mặc định) hoặc `Eager`. Ghi chú
  bên dưới nêu đúng luồng dữ liệu: gửi đoạn SQL quanh con trỏ cùng schema
  context tới AI provider bạn đã cấu hình. Chưa cấu hình key thì không có gì
  được gửi đi.
- **SQL snippets** — mở panel snippet (xem bên dưới).
- **New folder** — tạo thư mục trong query library theo tên trong ô nhập cạnh nút.

## Trình soạn thảo SQL

- **Tô màu cú pháp** theo dialect của driver đang dùng (PostgreSQL hoặc SQLite).
- **Tự động đóng ngoặc/nháy**; `Backspace` xóa cả cặp.
- **`Ctrl/Cmd+/`** bật/tắt comment `--` cho các dòng được chọn.
- **Overlay tìm kiếm** (`Ctrl/Cmd+F`): hộp "Search…" nổi trên editor. Gõ là nhảy
  tới kết quả đầu tiên; `Enter` xuống kết quả tiếp, `Shift+Enter` quay lại,
  `Esc` hoặc nút ✕ để đóng. Hiển thị `n/tổng`, hoặc `0/0` màu đỏ khi không có
  kết quả.
- **Popup gợi ý (completion)**: tự mở sau ~150 ms khi gõ ký tự identifier (ngoài
  chuỗi/comment), khi gõ `.`, hoặc thủ công bằng `Ctrl/Cmd+Space`. Di chuyển bằng
  ↑/↓ hoặc PageUp/PageDown, chấp nhận bằng `Enter` hoặc `Tab`, đóng bằng `Esc`.
  Mỗi mục có nhãn loại (table, column, keyword, snippet…).
- **Hover** lên tên bảng hoặc cột sẽ hiện card thông tin schema.
  `Ctrl/Cmd+click` lên identifier để mở bảng hoặc object tương ứng.
- **Signature help** hiện popup tham số khi con trỏ nằm trong lời gọi hàm.
- **Snippets**: gõ trigger như `sel*`, `upd*`, `ins*`, `del*`, `exp*`, `idx*`
  rồi chấp nhận gợi ý; `Tab`/`Shift+Tab` di chuyển giữa các placeholder
  `${n:…}`, `Esc` kết thúc phiên snippet.
- **Ghost text AI**: ở chế độ Subtle hoặc Eager, gợi ý hiện sau ~300 ms ngừng
  gõ. `Tab` chấp nhận toàn bộ, `Alt+→` chấp nhận một từ, `Alt+↓` một dòng,
  `Esc` bỏ qua. `Ctrl/Cmd+Alt+Space` yêu cầu thủ công.
- **Undo/redo**: `Ctrl/Cmd+Z` / `Ctrl/Cmd+Shift+Z`. `Ctrl/Cmd+A` chọn hết;
  `Tab`/`Shift+Tab` thụt/vượt đầu dòng cho vùng chọn.

### Status bar của editor

`Ln n, Col m`, vùng dòng đang chọn, tên driver, schema hiện tại, trạng thái
transaction (`Auto-commit` / `Manual` / `Transaction · n pending`), `n
parameters` (bấm để nhập giá trị bind), `n diagnostics` (bấm để xem cảnh báo
SQL), và nút "Show output" khi dock output đang đóng.

## Chạy truy vấn

- **Run** (mặc định `F5`; tooltip của nút Run hiển thị `Ctrl/Cmd+Enter`) chạy
  câu lệnh tại con trỏ hoặc vùng chọn. Ứng dụng tự xác định câu lệnh chứa con
  trỏ — không cần bôi đen.
- **Run script** (`Ctrl/Cmd+Shift+Enter`) chạy toàn bộ câu lệnh; mỗi câu tạo
  một tab kết quả (`Result 1`, `Result 2`, …).
- Khi truy vấn đang chạy, nút chuyển thành **Stop** (bấm hoặc nhấn `Esc` để
  hủy). Với driver không hỗ trợ hủy, nút hiển thị "Running" và tooltip giải
  thích lý do.
- Chạy khi chưa kết nối sẽ báo "Connect to a database before running"; chạy SQL
  rỗng báo "Query is empty".
- Giới hạn dòng chỉ áp dụng cho các lần chạy từ editor — saved task và các
  nguồn gọi khác giữ nguyên SQL gốc.

### Cổng chặn câu lệnh phá hủy

`DROP`, `TRUNCATE`, `DELETE` không có `WHERE`, `DO`, `CALL`, `EXECUTE` và các
nội dung script tương đương bị giữ lại trước khi gửi tới server. Hộp thoại "Run
Destructive Statement?" hiển thị đúng đoạn SQL (cắt ở 600 ký tự) với hai nút
**Run Destructive Statement** và **Cancel**. Không có gì chạy cho tới khi bạn
xác nhận.

## Bind parameter

Khi SQL dùng placeholder (`$1`, `$2`, … với driver kiểu PostgreSQL; `?` ở driver
hỗ trợ — `?` không được coi là placeholder trên PostgreSQL hay SQL Server),
panel tham số liệt kê từng tham số kèm ô nhập và nút "secret". Status bar hiện
`n parameters`. Giá trị chỉ giữ trong bộ nhớ của document; giá trị secret không
bao giờ được lưu cùng draft.

## Transaction

Bật thanh transaction từ nhãn transaction trên status bar hoặc mục "Show
transaction controls" trong menu actions.

- Mặc định là **Auto-commit active**. Chuyển sang chế độ manual để có nút
  **Begin Transaction**.
- Trong transaction, thanh hiển thị `Uncommitted Transaction` hoặc
  `Transaction Active`, mức cô lập (`[READ COMMITTED]`), số thay đổi đang chờ,
  và nút **Commit** / **Rollback**.
- Transaction đang mở sẽ chặn việc ngắt kết nối: hiện cảnh báo "Open
  transaction blocks disconnect — Commit or Rollback first" kèm nút Dismiss.

## Dock output

Năm tab — **Results**, **Chart**, **Messages**, **Explain**, **History** — với
badge số dòng trên Results và số thông báo trên Messages. Khi dải tab hẹp (dock
bên phải) chỉ còn icon. Các nút chrome: **Close output** (✕),
**Maximize/Restore output**, **Dock to bottom / Dock to right** (khi đang
dock), và tay nắm kéo để đổi kích thước. `Ctrl/Cmd+Shift+R` mở dock ở tab
Results và chuyển focus bàn phím vào grid.

### Results

Mỗi lần chạy hiển thị `{n} rows · {ms} ms`, nút ghim, nút **Export**, và nút
"Open results in a workspace tab". Script nhiều câu lệnh tạo một dãy tab kết
quả; chuột phải lên tab để có **Pin/Unpin Tab**, **Close**, **Close Other
Tabs**.

Trạng thái rỗng: "Statement completed" cho lệnh không trả về dòng nào, và
hướng dẫn "No results" khi chưa chạy gì.

**Toolbar của grid** (kết quả truy vấn): ô "Filter visible rows…" kèm bộ đếm
`n rows` cập nhật trực tiếp và nút xóa (✕); các nút copy; menu **Copy as ▾**
(các dòng đang hiển thị dưới dạng CSV, JSON, Markdown hoặc SQL INSERT); nút
bật/tắt record/value inspector; nút cell inspector nâng cao (hiện khi đã chọn
ô); và tooltip trợ giúp (ⓘ). Kết quả chỉ-đọc (ví dụ view, hoặc câu lệnh không
có khóa chính dùng được) hiện cảnh báo thay cho các điều khiển sửa.

**Tương tác trong grid:**

| Thao tác | Hiệu ứng |
|---|---|
| Click ô | Chọn ô. Click gutter `#` để chọn cả dòng. |
| Chuột phải ô | Menu: Copy Cell Value `Ctrl/Cmd+C`, Copy Row (TSV) `Ctrl/Cmd+Shift+C`, Copy Selected Rows (/with Headers/as JSON/as Markdown/as INSERT SQL), Copy Row as JSON/CSV; khi được phép sửa: Edit Cell `Enter`/`F2`, Set to NULL, Revert Cell, Revert Row / Undo Delete, Duplicate Row, Delete Row `Delete`/`Backspace`; ngoài ra Use column as row id, Open referenced row (khóa ngoại), Filter by this value, Sort Ascending/Descending. |
| Chuột phải header cột | Sort Ascending (A → Z) / Descending (Z → A) / Clear Sort, Copy Column Name, Copy Column Values, Add Filter, Hide Column / Show Columns, Move Column Left/Right, Reset Column Order, Reset Column Widths, Auto Size, Reset Layout. |
| Click header cột | Xoay vòng sắp xếp tăng → giảm → bỏ. (Ở surface table-data, `Shift+click` thêm sắp xếp nhiều cột.) |
| Kéo đường chia cột | Đổi độ rộng cột. |
| `Ctrl/Cmd+A` | Chọn toàn bộ dòng. `Esc` bỏ chọn. |
| `Ctrl/Cmd+S` / `Ctrl/Cmd+Z` trong grid | Áp dụng / hủy các thay đổi đã stage. Dán (`Ctrl/Cmd+V`) vào ô đã chọn sẽ stage một sửa đổi nếu kết quả cho phép sửa. |
| `Tab` khi đang sửa | Commit ô và nhảy sang ô kế tiếp. |

Ô sửa được mở editor inline phù hợp kiểu dữ liệu: ô text, dropdown cho boolean
và cột dạng enum, hoặc date picker cho cột date/datetime. Thay đổi đã stage
được tô sáng dòng cho tới khi bạn áp dụng (`Ctrl/Cmd+S`) hoặc hủy
(`Ctrl/Cmd+Z`).

**Record inspector** (nút bật panel trên toolbar) hiển thị dòng đang chọn dưới
dạng bảng Field/Value; click giá trị để copy. **Cell inspector** (nút 🔍) mở
trình xem lớn hơn cho giá trị dài.

### Chart

Biến kết quả thành biểu đồ với các bộ chọn **Type** (Bar, Line, Area, Scatter,
Pie), cột **X**, cột **Y** (chỉ cột số), **Agg** (None, Count, Sum, Average,
Min, Max) và cột **Series** tùy chọn. Dòng cuối báo số điểm đã vẽ, giới hạn
điểm, và số giá trị Y null/không-phải-số bị bỏ qua. Không có dòng nào thì hiện
"No data to chart".

### Messages

Các notice, warning và chi tiết thực thi do server trả về, đã khử trùng lặp.
Khi rỗng: "Query notices and execution details will appear here."

### Explain

Nút **Explain** và **Explain analyze**, toggle **Raw JSON**, và **Copy plan**
khi đã có plan. Plan PostgreSQL parse được sẽ vẽ thành cây plan node kèm
planning time, runtime stats (với ANALYZE) và tối đa 8 phát hiện của advisor
(hotspot/warning); plan không parse được sẽ hiện dạng text thô.

`EXPLAIN ANALYZE` thực sự chạy câu lệnh. Với driver mà ANALYZE có thực thi,
panel hiện cảnh báo và bắt buộc tick "I understand this will execute the
query" trước khi nút **Run EXPLAIN ANALYZE** bấm được.

### History

Các lần chạy gần đây của phiên, mới nhất lên đầu, lọc bằng ô tìm kiếm và giới
hạn hiển thị 50 dòng (`n total` ở góc phải). Mỗi mục có badge ✓ OK / ✕ FAIL,
thời gian chạy (ms), số dòng trả về hoặc số dòng ảnh hưởng, và preview SQL hai
dòng. Click preview để mở trong editor mới; **Replay** để chạy lại.

Mục **History** trong sidebar hiển thị nhật ký thực thi đã lưu (`{connection} ·
{schema} · {outcome} · {time} · {ms}`), có bộ lọc theo kết quả (ok / error /
cancelled), nhóm theo ngày (Today / Yesterday / ngày cụ thể) và thao tác "Open
as new query".

### Export

Nút **Export** ở pane Results mở hộp thoại: chọn định dạng (CSV, TSV, JSON,
Markdown, INSERT, COPY, Excel), nhập đường dẫn file đầu ra rồi xác nhận. Export
chỉ ghi các dòng mà kết quả thực sự nắm giữ — nếu kết quả bị cắt bởi giới hạn
dòng, thông báo sẽ nói rõ bao nhiêu dòng khớp so với bao nhiêu dòng đã ghi. Nếu
file đã tồn tại bạn sẽ thấy **Overwrite** / **Keep existing file**; export lỗi
không để lại file ghi dở.

## Visual query builder

Mở từ menu actions. Dựng câu `SELECT` từ bảng/cột được chọn theo dialect đang
dùng, hiện preview SQL, và cung cấp **Apply to editor** (chép SQL sinh ra vào
document mà không chạy), **Import from editor** (parse một `SELECT` được hỗ trợ
ngược về builder — SQL không hỗ trợ sẽ bị từ chối kèm lỗi và text của bạn giữ
nguyên), và **Clear**.

## Phím tắt

Tất cả đều đổi được trong **Settings → Keybindings**; `mod` nghĩa là `Ctrl`
trên Windows/Linux và `Cmd` trên macOS.

| Lệnh | Mặc định |
|---|---|
| New query | `mod+T` |
| Run query | `F5` |
| Run script (toàn bộ câu lệnh) | `mod+Shift+Enter` |
| Format query | `mod+Shift+F` |
| Save query | `mod+S` |
| Save query as | `mod+Shift+S` |
| Find in editor | `mod+F` |
| Toggle comment | `mod+/` |
| Focus results grid | `mod+Shift+R` |
| Quick open | `mod+P` hoặc `mod+K` |
| Command palette | `mod+Shift+P` |
| Dừng truy vấn đang chạy | `Esc` (nếu driver hỗ trợ) |

## Các trạng thái và giới hạn đáng chú ý

- **Chưa kết nối**: chip chuyển thành cảnh báo "No connection" và Run bị vô
  hiệu, tooltip yêu cầu kết nối trước.
- **Giới hạn dòng**: các lần chạy từ editor bị giới hạn theo mức đã chọn (tối
  đa 100.000); `EXPLAIN`, export và history không bị ảnh hưởng — export chỉ ghi
  các dòng đã lấy về.
- **Completion** bị tắt bên trong chuỗi và comment.
- **AI prediction** cần provider đã cấu hình; ghi chú egress trong menu actions
  nêu chính xác dữ liệu nào rời khỏi máy.
