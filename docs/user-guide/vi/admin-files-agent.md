# Quản trị, Files & AI Agent

Tài liệu này hướng dẫn các màn hình quản trị của DB Pro: **Settings** (Cài đặt), **Server Monitor** (kèm audit, replication, FDW, event trigger và server settings), **Security & Roles**, **Saved Tasks**, **Data Transfer**, không gian làm việc **Files**, và panel **Agent** (AI copilot).

Hầu hết các màn hình quản trị nằm trong sidebar. Để mở, bấm biểu tượng **Tools & Management Hub** (hình các khối) trên activity bar bên trái rồi chọn thẻ công cụ:

| Nhóm | Thẻ | Mở |
|---|---|---|
| Schema & Migration | Schema & Data Compare, Schema Workbench, ER Diagram | Xem hướng dẫn Table Workbench |
| Administration & Server | **Server Monitor**, **Security & Roles**, **Execution History & Audit** | Các activity Monitor / Security / History trong sidebar |
| Data & Automation | **Data Transfer & Migration**, **Saved Tasks & Backups**, **Diagnostics & Problems** | Transfers / Tasks / Problems |

Ô tìm kiếm đầu hub lọc các thẻ. Các activity này cũng mở được từ **Command Palette** (`Ctrl/Cmd+Shift+P`).

Workspace **Files** dùng chung biểu tượng rail **SQL Scripts & Projects**; nó xuất hiện trong sidebar sau khi bạn mở một thư mục (xem bên dưới). Panel **Agent** bật/tắt bằng biểu tượng tia lửa (sparkles) trên activity bar.

---

## 1. Settings (Cài đặt)

Mở **Settings** bằng biểu tượng bánh răng ở cuối activity bar. Màn hình gồm hai phần: cột điều hướng **SETTINGS** bên trái và nội dung cuộn được bên phải. Cuối mỗi mục luôn có thẻ **DIAGNOSTICS** (xem §1.11).

Các nhóm trong cột điều hướng:

- **Workspace** — General, Appearance, Editor, Data Grid
- **Connections** — Connections, AI Providers
- **System** — Keybindings, Backup / Restore, Security, Advanced

Toàn bộ cài đặt được lưu cục bộ dưới dạng JSON có phiên bản (schema `settings-v1`); giá trị vượt biên sẽ bị kẹp lại khi nạp (cỡ chữ 10–24, tab 2–8, page size lưới 25–1000). Mật khẩu không bao giờ ghi vào file này — chúng nằm trong secret store.

### 1.1 General

- **Language** — hàng nút đổi ngôn ngữ giao diện.
- **Confirm destructive queries** — hỏi xác nhận trước khi chạy SQL phá huỷ dữ liệu.
- **Restore query tabs on startup** — mở lại các tab query của phiên trước khi khởi động.

**Workspace sessions**: session có tên lưu *bố cục và tham chiếu tab* — không lưu nội dung SQL, secret hay lưới kết quả. Nhập tên vào **Session name**, bấm **Save**, rồi quản lý bằng **Restore**, **Duplicate**, **Delete**. Nếu việc restore có điểm cần lưu ý (ví dụ thiếu tab), ghi chú sẽ hiện bên dưới.

### 1.2 Appearance

- **Light / Dark** — chuyển giao diện sáng/tối, áp dụng ngay.
- **Reduce motion** — thay spinner bằng biểu tượng trạng thái tĩnh khi đang tải.

### 1.3 Editor

- **Font size** — nút `−` / `+`, 10–24 px.
- **Tab width** — kéo giá trị, 2–8.
- **Schema completion** — bật/tắt popup gợi ý.
- **Format SQL on save** — tự format khi lưu.
- **AI prediction** — `Off` / `Subtle` / `Eager`. Gửi đoạn SQL quanh con trỏ cùng ngữ cảnh schema tới provider AI đã cấu hình.

**SQL Lint** — công tắc tổng **Enable SQL lint warnings**, sau đó từng rule: cảnh báo `SELECT *`, `= NULL / != NULL`, `DELETE` không có `WHERE`, `UPDATE` không có `WHERE`, `ORDER BY` theo số thứ tự, join bằng dấu phẩy / cartesian, alias trùng trong projection. Lint chạy hoàn toàn cục bộ — không gọi AI hay mạng. Từng rule cũng có thể tắt riêng theo mã (ví dụ `lint.comma-join`).

### 1.4 Data Grid

- **Page size** (25–1000), **Show row numbers**, **Wrap cell text**.

### 1.5 Connections

- **Reconnect last connection on startup** — tự kết nối lại kết nối cuối khi mở app.
- **Prefer TLS for new server connections** — ưu tiên TLS cho kết nối server mới.
- Lưu ý: mật khẩu nằm trong secret storage, không bao giờ ghi vào JSON cài đặt.

### 1.6 AI Providers

- **Enable Agent workspace**, **Allow Agent to auto-run read-only queries** (cho Agent tự chạy query chỉ-đọc).
- Hiển thị nhãn **Active provider**. API key được nhập trong **panel Agent** (biểu tượng bánh răng → API Key, §6) và lưu dạng secret.

### 1.7 Keybindings

Danh sách có tìm kiếm chứa mọi lệnh gán phím được cùng phím tắt đang áp dụng. Gõ vào **Search commands…** để lọc theo tên hoặc id lệnh.

- Bấm **Edit** trên một dòng, nhập token kiểu `mod+k`, `mod+shift+p`, `f5`, `mod+/` (modifier chấp nhận: `mod`/`cmd`/`ctrl`, `shift`, `alt`; phím: chữ, số, F1–F12, Enter, Escape, Delete, `/`). Nhấn Enter hoặc **Save**. Xoá trắng ô hoặc nhập lại mặc định sẽ gỡ override.
- **Reset** (từng dòng, chỉ hiện khi có override) và **Reset all to defaults**.
- Phím trùng nhau sẽ được cảnh báo (`N keybinding conflict(s) detected`) và các dòng liên quan được tô màu.

Danh mục mặc định (áp dụng nếu không override):

| Lệnh | Mặc định |
|---|---|
| Quick Open / (phụ) | `mod+p` / `mod+k` |
| Command Palette | `mod+shift+p` |
| Toggle Sidebar | `mod+b` |
| Find in Editor | `mod+f` |
| Save Query / Save As | `mod+s` / `mod+shift+s` |
| Run Query / Run Script | `F5` / `mod+shift+enter` |
| Format Query | `mod+shift+f` |
| New Connection / New Query | `mod+n` / `mod+t` |
| Open Table | `mod+o` |
| Focus Results | `mod+shift+r` |
| Toggle Comment | `mod+/` |

(`mod` = Ctrl trên Linux/Windows, Cmd trên macOS.)

### 1.8 Backup / Restore

Chỉ hiện khi provider đang dùng hỗ trợ; nếu không, thẻ báo *"Backup and restore are unavailable for the active provider."*

- Ô **Backup destination** → **Choose path** (hộp thoại chọn file hệ thống) → **Create backup**.
- Ô **Restore from backup** → **Choose file** → **Restore database** → xác nhận trong **"Overwrite the active database?"** (**Confirm restore** / **Cancel**).

Ghi chú theo driver (hiển thị ngay trong UI):

- **SQLite** — snapshot bằng `VACUUM INTO` (an toàn với WAL); restore bị từ chối khi connection còn active — hãy disconnect trước.
- **PostgreSQL** — cần `pg_dump` trong PATH để backup; restore dùng `psql` (plain) hoặc `pg_restore` (custom). Thiếu tool sẽ bị phát hiện trước khi chạy.
- **MySQL** — chưa hỗ trợ.

### 1.9 Security

- **Redact secrets in diagnostics, logs and saved query history** — che secret
  trong log/chẩn đoán và bản sao lịch sử truy vấn ghi ra đĩa (mặc định bật). Câu
  lệnh trong phiên hiện tại vẫn giữ nguyên văn; chỉ bản sao lưu vào workspace
  store được che các literal credential như `PASSWORD '…'`. Tắt tuỳ chọn này nghĩa
  là lưu nguyên văn ra đĩa.
- **Block secret export by default** — chặn export secret (mặc định bật).

### 1.10 Advanced

- **Verbose runtime logging**, **Experimental features**, và phiên bản schema cài đặt (chỉ đọc).

### 1.11 Diagnostics (hiện dưới mọi mục)

Hiển thị `DB Pro <version> · <os>/<arch>`, tình trạng driver, số connection, số execution đang chạy, và tối đa 5 lỗi có cấu trúc gần nhất.

| Nút | Tác dụng |
|---|---|
| **Copy diagnostics summary** | Copy bản tóm tắt dạng JSON vào clipboard (secret đã che) |
| **Export support bundle** | Ghi gói hỗ trợ đã che secret ra đĩa; đường dẫn hiện ở status message |

---

## 2. Server Monitor

Mở qua **Tools hub → Server Monitor**. Cần có kết nối đang mở (nếu chưa sẽ hiện "Connect from Explorer to monitor sessions."). Header hiển thị **Connected/Disconnected**, tên connection và driver, checkbox **Auto-refresh** (poll mỗi ~5 giây) và nút **Refresh**.

### 2.1 Health Advisor

Heuristic xác định chạy trên snapshot — không bao giờ thay đổi database. Mỗi phát hiện gồm badge mức độ (critical/warning/info), đối tượng ảnh hưởng, bằng chứng, giải thích và hành động gợi ý. Khi sạch sẽ hiện "No findings".

### 2.2 Sessions

- Bộ lọc **Active queries only** (mặc định bật).
- Khối **IDLE IN TRANSACTION** liệt kê tối đa 20 session idle-in-xact với `pid`, `xact_age`, `backend_age`, user.
- Mỗi thẻ session: `pid`, badge `current`, trạng thái, `query N ms`, `xact N ms`, badge `idle-in-xact`, user/database/application, đoạn query bị cắt ngắn, và các hành động:

| Nút | Tác dụng |
|---|---|
| **Open SQL** | Nạp query của session vào Query editor |
| **Cancel** | Xác nhận qua *"Cancel backend pid N?"* → **Cancel backend** / **Keep running** |
| **Terminate** | Xác nhận qua *"Terminate session?"* (ngắt kết nối client đó) → **Terminate** / **Cancel** |

Session của chính bạn (badge `current`) không thể bị cancel/terminate từ đây.

### 2.3 Server, Locks, Sizes

- **SERVER** — phiên bản, database hiện tại, số connection đang dùng/tối đa, dung lượng database.
- **LOCKS / BLOCKERS** — các cặp lock chặn nhau (pid bị khoá, granted/waiting, pid chặn, mode, relation), tối đa 30.
- **SIZE / STATS** — tối đa 20 relation: `schema.name (kind) · size · seq/idx scan · dead tuple`.
- **LOCAL STATE** (SQLite) — journal mode, số page/kích thước page, freelist, dung lượng file.

### 2.4 Top Queries (pg_stat_statements)

Hiện khi snapshot có dữ liệu workload; cảnh báo nếu thiếu extension.

- Nút sắp xếp: **total time · mean time · calls · rows**.
- Ô **Filter** khớp theo text query, database hoặc user.
- Tối đa 40 statement với calls/total/mean/rows, shared-buffer hit/read, temp đọc/ghi, và delta `Δ total` so với snapshot trước.
- **Open SQL** nạp statement đã chuẩn hoá vào editor.
- **Reset stats…** mở *"Reset pg_stat_statements?"* — xoá toàn bộ thống kê trên server (**Reset statistics** / **Cancel**).

### 2.5 Maintenance

Ô nhập schema/table cùng các nút **VACUUM**, **ANALYZE**, **VACUUM ANALYZE**. Mỗi nút mở *"Run maintenance?"* cảnh báo VACUUM chạy lâu có thể giữ lock — xác nhận hoặc huỷ.

### 2.6 Các màn hình PostgreSQL phụ (dưới snapshot monitor)

| Mục | Chức năng |
|---|---|
| **Database audit (server CSV log)** | Đọc CSV log của server nếu bạn cấu hình tag connection `audit:csvlog=/duong/dan/logfile.csv` (không bao giờ tự bật). Bộ lọc: Text, DB, User, Severity. **Load page**, **Export selected**, chọn/bookmark từng sự kiện, **Open SQL**. Hiện số byte đã quét và trạng thái bị cắt. |
| **FOREIGN DATA (FDW)** | **Load FDW inventory**; tạo foreign server qua **Preview CREATE** → **Create (confirm)**; xoá kèm preview (**Drop CASCADE** có xác nhận). Password/options bị che; `CREATE EXTENSION` không bao giờ tự chạy. |
| **LOGICAL REPLICATION** | **Load replication inventory**: publication (kèm bảng, owner), subscription (conninfo đã che), replication slot. **Preview DROP** / **Drop…** từng cái; tạo publication `FOR ALL TABLES` qua **Preview CREATE** → **Create (confirm)**. Không cung cấp `CREATE SUBSCRIPTION` (cần secret). |
| **EVENT TRIGGERS** | **Load event triggers**; tạo trigger (cần function có sẵn) qua **Preview CREATE** → **Create (confirm)**; **Drop** có xác nhận. Chỉ PostgreSQL, hook DDL mức database — không phải trigger mức bảng/dòng. |
| **SERVER SETTINGS (pg_settings)** | Lọc các GUC của server; `SET`/`RESET` mức session cho setting user-context; `ALTER SYSTEM` chỉ ở dạng preview. |

Driver không phải PostgreSQL sẽ hiện thông báo thay thế ("MySQL has no sessions", v.v.).

---

## 3. Security & Roles

Mở qua **Tools hub → Security & Roles**. Chỉ dành cho PostgreSQL và bị giới hạn theo capability: driver khác hiện *"User/role management is PostgreSQL-only"*; chưa kết nối thì hiện *"Connect a PostgreSQL database to manage roles and privileges."*

### 3.1 Roles / Users

**Refresh roles** nạp danh sách. Mỗi dòng role hiện badge quyền (`login`, `super`, `createdb`, `createrole`) và nút **Drop** → dialog *"Drop role `name`? This cannot be undone."*

**Create role**: nhập tên, checkbox **LOGIN** tuỳ chọn → **Create role**.

### 3.2 Chi tiết role (bấm vào một role)

- **ATTRIBUTES** — bấm một chạm `LOGIN`, `NOLOGIN`, `CREATEDB`, `CREATEROLE`, `NOCREATEDB`, `NOCREATEROLE` (alter-role, có dialog xác nhận).
- **Password** — đặt mật khẩu mới cho role (**Update password**).
- **Memberships** — liệt kê `member of <role>` kèm nút **Revoke** (có xác nhận); cấp qua ô *grant role name* → **Grant membership**.
- **PRIVILEGES** — quyền hiện có theo object; revoke cần xác nhận. Form **GRANT PRIVILEGE**: chọn loại object, nhập schema/object/privilege → **Grant privilege**.

### 3.3 Row-Level Security

Nhập **schema** và **table**:

- **Inspect RLS** — hiện `enabled`, `forced` và các policy hiện có (`name · command · PERMISSIVE/RESTRICTIVE · roles[…] · USING(…) · CHECK(…)`).
- Công tắc mức bảng: **Enable RLS**, **Disable RLS**, **Force RLS**, **No Force** — đều preview rồi mới xác nhận.
- Theo policy: **Drop policy**, **Load for edit** (đổ dữ liệu vào form).
- Form **CREATE / ALTER POLICY**: tên policy, command (`ALL/SELECT/INSERT/UPDATE/DELETE`), roles (phân tách dấu phẩy, để trống = `PUBLIC`), checkbox **Restrictive**, biểu thức `USING` và `WITH CHECK` → **Preview CREATE** hoặc **Preview ALTER** hiện đúng câu SQL → tick *"I understand this changes data visibility immediately"* → **Apply preview SQL**.

---

## 4. Saved Tasks & Backups

Mở qua **Tools hub → Saved Tasks & Backups**. Bộ lập lịch chỉ chạy khi DB Pro đang mở — không có daemon nền. Secret nằm trên connection.

Trạng thái rỗng: *"No saved tasks — Create a SQL, export, backup, or maintenance task to run manually."*

### 4.1 Tạo task

- **New SQL task** — tạo nháp từ text query hiện tại và connection đang active.
- **New backup task** — tạo nháp payload backup (đường dẫn output, tuỳ chọn **Custom (pg_dump -Fc) format**).

Trường trong form nháp: **Name**, **Description**, cộng trường payload:

| Payload | Trường | Hành vi khi chạy |
|---|---|---|
| SQL | Textarea SQL | Chạy SQL trên connection của task |
| Backup | Output path, custom format | Chạy lệnh backup |
| Export | Table (tuỳ chọn), Format (`csv`/`tsv`/`json`) | Chạy `SELECT` trên bảng và ghi kết quả ra file tạm (`db-pro-export-<id>.<fmt>`) |
| Maintenance | Operation (`vacuum`/`analyze`), Target (tuỳ chọn) | Chạy lệnh maintenance (đã xác nhận sẵn) |

**Save task** / **Cancel**.

### 4.2 Chạy và lập lịch

Mỗi thẻ task hiện tên, loại, mô tả, connection, lần chạy cuối (`status · ms · message`) và trạng thái lịch (`on/off · next <thời gian>`).

| Nút | Tác dụng |
|---|---|
| **Run** | Chạy ngay. Payload phá huỷ cần tick **Confirm destructive task run** → **Run destructive task** trước. |
| **Schedule 60s** / **Disable schedule** | Bật/tắt lịch chu kỳ 60 giây (app phải mở). Task phá huỷ chạy theo lịch bị chặn theo policy. |
| **Delete** | Hỏi *"Delete this saved task?"* → **Delete task** / **Keep task**. |

**RECENT RUNS** ở cuối liệt kê 12 lần chạy gần nhất (`OK`/`FAIL`/`CANCEL`, thời lượng, message).

---

## 5. Data Transfer & Migration

Mở qua **Tools hub → Data Transfer & Migration**. Dòng giới thiệu: *"Streaming transfer engine · bounded batches · cancellable."*

### 5.1 Thẻ Backup / Restore

Lối tắt nhảy tới **Settings → Backup / Restore** (`pg_dump`/`pg_restore` theo provider hoặc snapshot SQLite).

### 5.2 Synthetic Table Seed (chỉ dev/test)

Sinh dữ liệu giả xác định theo seed. Chọn bảng từ schema đã nạp, đặt **rows**, **seed**, **null%**:

- **Preview** — xem 8 dòng đầu được sinh.
- **Export SQL → Query** — ghi script `INSERT` vào Query editor.
- **Apply INSERT (run)** — export và thực thi luôn. Trên connection *Production* phải tick **"I confirm seeding this Production database"** trước.

### 5.3 Masking / Safe Sample

Mặc định chỉ preview — không mask tại chỗ phá dữ liệu.

- Ô **cols**: tên cột phân tách dấu phẩy (vd `email,phone`), hoặc bấm **Suggest cols** để tự nhận diện cột nhạy cảm.
- Chọn rule: **Redact / Hash / Partial / Fixed / Synthetic**; **Keyed hash** giữ liên kết join nhất quán.
- **Preview sample** hiện cặp gốc → đã mask; **Masked CSV export** chạy job export.

### 5.4 Copy / harness

Trường: **Source** / **Target** là connection id, **Tables** (`schema.table…`), **Format** (`csv` hoặc `json`), **Conflict** (`abort` / `skip` / `replace`).

Thanh công cụ:

| Nút | Tác dụng |
|---|---|
| **Run synthetic harness** | Job kiểm thử transfer streaming đầu-cuối |
| **Run then cancel** | Tương tự nhưng huỷ giữa chừng (kiểm tra cancel) |
| **CSV export harness** / **CSV import preview** / **JSONL export harness** / **Excel export harness** | Các job harness kiểm tra định dạng |
| **Copy tables** | Chạy copy DB→DB theo các trường trên (cần mở cả hai connection) |
| **Clear jobs** | Xoá danh sách job |

### 5.5 Jobs

Mỗi thẻ job hiện nhãn, badge trạng thái (`pending`/`running`/`succeeded`/`failed`/`cancelled`/`partial`), tiến độ (`read / wrote / bytes / committed batches / uncommitted / error rows / message`) và lỗi nếu có. Trạng thái rỗng: *"No transfers yet."*

---

## 6. Agent (AI Copilot)

Bật/tắt panel **Agent** bên phải bằng biểu tượng sparkles trên activity bar. Panel kéo rộng được (kéo cạnh trái).

### 6.1 Chế độ mặc định — Offline draft

Khi chưa có API key, badge provider hiện **Offline draft** / **Preview**: prompt tạo ra phản hồi nháp không thực thi — không gửi đi đâu và không chạy gì cả.

### 6.2 Cấu hình provider

Bấm biểu tượng bánh răng (**Agent settings (API key)**) trên header panel:

- Dán key (`gsk_…` cho Groq, `sk-…` cho OpenAI — provider tự nhận diện theo tiền tố) → **Save key** hoặc `Ctrl/Cmd+Enter`. **Forget key** để xoá. Key đi vào secret store, không bao giờ ghi plain text ra đĩa.
- Thông báo nêu rõ dữ liệu nào rời máy: *prompt của bạn, SQL mà prompt tham chiếu, tên và kiểu schema xung quanh, và tối đa 20 dòng kết quả mẫu (50 cột, 256 ký tự mỗi ô) khi agent chạy query. Kết nối database và SSH là các kết nối outbound duy nhất khác của app.*
- Checkbox **Auto-run read-only queries in Agent mode** đồng bộ với Settings → AI Providers.

### 6.3 Header & context

- Chọn chế độ: **Ask** (hỏi đáp), **Edit** (đề xuất sửa SQL), **Agent** (nhiều bước dùng tool). Bị khoá khi đang chạy hoặc đang chờ xác nhận.
- Icon: **Clear conversation** (khi rảnh), **Close Agent**, **Agent settings**.
- Chip context hiện ngữ cảnh trực tiếp: provider, badge `Auto-run Read-only`, connection, driver, schema, bảng đang chọn, cờ `EXPLAIN PLAN`.
- Hành động nhanh (hiện khi phù hợp): **Explain query**, **Optimize** (SQL hiện tại), **Explain table** (bảng đang chọn), **Investigate error** (lỗi DB gần nhất) — mỗi cái gửi một prompt dựng sẵn.

### 6.4 Thread

Trạng thái rỗng gợi ý sẵn các prompt (*"Show me the schema overview"*, *"Count rows in the active table"*, *"Explain query performance"*). Tin nhắn stream vào; activity hiện trạng thái (`Running`, `Needs approval`, `Done`, `Failed`, `Cancelled`) và thời lượng. Kết quả tool query hiện `N rows, M cols` kèm nút **Open in Results** / **Open sample in Results**. Session lỗi có nút **Retry**.

### 6.5 Xác nhận (approvals)

Agent tạm dừng ở các hành động cần phê duyệt:

| Tiêu đề | Nút đồng ý |
|---|---|
| Apply Agent change? (hiện diff −/+ SQL) | **Apply Change** |
| Run read-only query? | **Run Query** |
| Run mutation? (cảnh báo: sửa dữ liệu/schema) | **Run Mutation** |
| Execute destructive query? (có thể drop/truncate) | **Execute Destructive Query** (kiểu destructive) |
| Run unclassified query? | **Run Unclassified Query** |

**Reject** để từ chối. Query chỉ-đọc chỉ tự chạy nếu bạn bật auto-run.

### 6.6 Composer

Ô nhập cuối panel: gõ yêu cầu rồi gửi (hoặc **Stop** khi đang generate). Session agent gắn theo từng document query — đổi tab không mất hội thoại.

---

## 7. Workspace Files

Mở thư mục qua rail **SQL Scripts & Projects** → thẻ workspace → **Open Folder** (hoặc icon thư mục trên header rail). Sidebar sẽ có header **WORKSPACE** và thẻ hiện root đang dùng, đường dẫn, trạng thái trust, environment và các chip root.

- **Open Folder** / thư mục gần đây (tối đa 8) khi chưa mở gì.
- Thẻ workspace: tên + đường dẫn root đang dùng, **Close workspace** (✕), **Remove active root** (nhiều root), công tắc trust **Trust** / **Revoke** (*"Trust this workspace to enable writes and tasks"*), chọn **Environment**, chip **Roots:** khi có nhiều root, cảnh báo schema-drift và lỗi.
- Icon header: thêm/mở thư mục, **Refresh**.

### 7.1 Các tab

| Tab | Nội dung |
|---|---|
| **Files** | Cây file (`FILES · n`), icon **New SQL file** / **New folder**. Bấm `.sql` để mở trong editor; chuột phải thư mục: **New SQL here** / **Delete folder**, file: **Open**, **Add to Agent context**, **Find references**, **Delete**. |
| **Search** | **Find in files…** + **Replace with…**: **Find**, **Preview** (hiện các dòng `file · N hits`), **Replace all** → *"Replace all matches in the workspace?"* → **Confirm replace** / **Keep files**. Hàng **Rename from / to / Refactor** chạy refactor đổi tên. Kết quả tìm (`path:line`) bấm vào sẽ mở file `.sql`. |
| **Migrations** | File SQL phát hiện dưới các đường dẫn `migrations/` kèm version + status; bấm để mở. |
| **Tasks** | Chạy lệnh shell trong root workspace — cần xác nhận (*"Run this command with sh -lc?"* → **Confirm shell** / **Do not run**). Hiện kết quả cuối (lệnh, exit code, ms, stdout/stderr). **Benchmark** chạy đo thời gian cục bộ. |
| **Graph** | Cạnh phụ thuộc `file → object` trích từ `FROM`/`JOIN` (tối đa 60). |
| **Git** | **Refresh** dò git status của root đang dùng: nhánh, các mục `[XY] path` với **Stage** / **Unstage** / **Diff** / **Open**, ô commit message (chỉ khi nhập tay — không bao giờ tự động) và **Commit staged**. Diff hiện inline (cắt ở 8000 ký tự). Cảnh báo khi file đổi trên đĩa (*"Disk changed for … — unsaved editor buffer was kept"* → **Reload from disk** / **Dismiss**). |

### 7.2 Dải Agent context (cuối panel)

Đính context cho Agent: các nút **Active File**, **Selection**, **Table**, chip context có thể gỡ, và menu **⋯**: **Check schema drift**, **Export schema snapshot**, **Toggle split editor**, **Clear context**.

---

## 8. Ghi chú vòng đời kết nối

- Nút **Test connection** trong dialog kết nối đổi thành **Cancel test** khi đang kiểm tra (kèm chữ "Testing…"); huỷ sẽ báo *"Connection test cancelled"* và giữ dialog mở.
- **Delete connection** luôn hỏi qua dialog xác nhận (**Cancel** để huỷ).
- Status bar hiện **Connecting…** trong lúc chờ kết nối; lỗi runtime hiện ở vùng status message và trong **Settings → Diagnostics**.
