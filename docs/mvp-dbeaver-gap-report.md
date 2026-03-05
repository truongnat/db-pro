# DB Pro MVP Benchmark Report (So sánh chi tiết với DBeaver)

- Ngày cập nhật: 2026-03-05
- Phạm vi benchmark: trải nghiệm desktop SQL client cho cá nhân/team nhỏ, ưu tiên bản tương đương DBeaver Community cho workflow hằng ngày.
- Đối tượng: sản phẩm `DB Pro` (Tauri + Rust + React/shadcn).

## 1) Tóm tắt điều hành

DB Pro đã có nền tốt cho MVP kỹ thuật (kết nối đa engine, query chạy async, cancel query, schema navigator, autocomplete cơ bản). Tuy nhiên để đạt “cảm giác dùng” ngang DBeaver ở các tác vụ cơ bản mỗi ngày, còn thiếu khá nhiều chi tiết UX/interaction nhỏ nhưng quan trọng:

- Thiếu các "micro-interactions" chuẩn desktop DB client (keyboard-first, grid utilities, trạng thái tải/lỗi rõ ràng, history/query log, filter/search ở navigator và result).
- SQL editor chưa có các capability cốt lõi mà user kỳ vọng ngay (formatter, template/snippet, parameter binding UX, query history, multi-tab script workspace).
- Result grid mới ở mức hiển thị; chưa đủ hành vi làm việc với dữ liệu như ứng dụng DB thực chiến (copy/export linh hoạt, column operations, quick filter/sort, cell tooling).
- Connection UX chưa đạt chuẩn “pro tooling” (form step-by-step, test/tunnel/network profile, retry/reconnect, trạng thái connection sống).

Kết luận: cần một MVP v2 tập trung 100% vào chất lượng workflow (không thêm tính năng “đẹp mắt” trước), với backlog chi tiết theo feature-ID, DoD và thứ tự P0/P1/P2 bên dưới.

---

## 2) Cách audit

### 2.1 Nguồn benchmark DBeaver

Dùng tài liệu chính thức DBeaver (docs + homepage) để tránh suy diễn:

- [DBeaver Community homepage](https://dbeaver.io/)
- [SQL Editor](https://dbeaver.com/docs/dbeaver/SQL-Editor/)
- [SQL Assist and Auto Complete](https://dbeaver.com/docs/dbeaver/SQL-Assist-and-Auto-Complete/)
- [SQL Templates](https://dbeaver.com/docs/dbeaver/SQL-Templates/)
- [SQL Execution](https://dbeaver.com/docs/dbeaver/SQL-Execution/)
- [Database Navigator](https://dbeaver.com/docs/dbeaver/Database-Navigator/)
- [Data Editor](https://dbeaver.com/docs/wiki/Data-Editor/)
- [Data Viewing and Editing](https://dbeaver.com/docs/dbeaver/Data-Viewing-and-Editing/)
- [Result Set Navigation](https://dbeaver.com/docs/dbeaver/Navigation/)
- [Data Transfer](https://dbeaver.com/docs/dbeaver/Data-transfer/)
- [Data Export](https://dbeaver.com/docs/dbeaver/Data-export/)
- [Data Import](https://dbeaver.com/docs/dbeaver/Data-import/)
- [Query Manager](https://dbeaver.com/docs/dbeaver/Query-Manager/)
- [Create Connection](https://dbeaver.com/docs/dbeaver/Create-Connection/)
- [Secure Storage](https://dbeaver.com/docs/dbeaver/Security/)
- [Driver Manager](https://dbeaver.com/docs/dbeaver/Driver-Manager/)
- [ER Diagrams](https://dbeaver.com/docs/wiki/ER-Diagrams/)
- [Shortcuts](https://dbeaver.com/docs/dbeaver/Shortcuts/)

### 2.2 Nguồn audit DB Pro (code thực tế)

- Frontend: `src/App.tsx`, `src/features/connections/*`, `src/features/sql-editor/*`, `src/features/query/*`, `src/features/navigator/*`
- Rust backend: `src-tauri/src/commands/*`, `src-tauri/src/sample.rs`, `src-tauri/src/secrets.rs`, `src-tauri/src/state.rs`, `src-tauri/src/storage.rs`

---

## 3) Baseline DB Pro hiện tại (đã có gì)

## 3.1 Điểm mạnh hiện có

- Quản lý connection CRUD + persist JSON + secure password trong keychain.
- Auto bootstrap `Sample SQLite` + seed dữ liệu demo khi khởi động.
- Chạy query async với timeout, cancel query, page-size, offset.
- Schema navigator cho SQLite/PostgreSQL/MySQL (schema/table/view/column).
- Auto refresh navigator sau DDL (`CREATE/ALTER/DROP/TRUNCATE/RENAME`).
- SQL editor có syntax highlight theo dialect + autocomplete keyword/object/column (catalog từ navigator).
- UI desktop khá sạch, dùng shadcn; chia panel editor/result.

## 3.2 Hạn chế lớn hiện tại

- Chưa có workspace đa tab script/query như DBeaver.
- Chưa có formatter SQL, templates/snippets, variables/parameters UX.
- Chưa có query history/query manager để truy vết thao tác.
- Result grid thiếu nhiều thao tác data-centric (copy modes, filter/sort UI, column tooling, export ngay từ result).
- Navigator chưa có filter/search và context actions mạnh (Generate SQL/Open Data nhanh).
- Connection flow chưa có các advanced config (SSH/SSL/proxy profile, retry/reconnect policy, health-state rõ).
- Chưa có test coverage và kiến trúc clean architecture rõ theo tầng.

---

## 4) Ma trận tính năng chi tiết cho MVP (từng feature nhỏ)

Ghi chú trạng thái:
- `DONE`: đã có tương đối dùng được
- `PARTIAL`: có nhưng UX/behavior chưa đủ
- `MISSING`: chưa có

## 4.1 Connection & Security

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| CONN-001 | Add connection (SQLite/Postgres/MySQL) | DONE | Modal mở nhanh <150ms, validate bắt buộc, submit bằng Enter | P0 |
| CONN-002 | Edit connection | DONE | Prefill chuẩn, giữ password cũ nếu để trống, cảnh báo nếu đổi host/port | P0 |
| CONN-003 | Delete connection | DONE | Confirm dialog rõ tên connection, chặn xóa sample mặc định | P0 |
| CONN-004 | Test connection | DONE | Timeout rõ ràng, hiển thị latency, lỗi có nguyên nhân (auth/network/ssl) | P0 |
| CONN-005 | Auto-create sample SQLite | DONE | First-run luôn có sample, nếu file hỏng thì recreate an toàn | P0 |
| CONN-006 | Auto-connect mặc định vào SQLite sample | PARTIAL | Khi app mở lần đầu chọn sample + load navigator thành công | P0 |
| CONN-007 | Persist connection profiles | DONE | Không lưu plaintext password trong JSON | P0 |
| CONN-008 | Password secure storage (OS keychain) | DONE | Save/load/delete password đồng bộ vòng đời connection | P0 |
| CONN-009 | Connection form draft cache | DONE | Draft create-mode được restore sau restart | P1 |
| CONN-010 | Selected connection restore | DONE | Mở app nhớ connection gần nhất nếu còn tồn tại | P0 |
| CONN-011 | Advanced network settings (SSH/SSL/Proxy) | MISSING | UI tab advanced + model config + test tunnel/ssl | P1 |
| CONN-012 | Connection status badge (connected/disconnected/testing) | PARTIAL | Trạng thái realtime ở sidebar + header, không chỉ text chung | P0 |
| CONN-013 | Retry & reconnect policy | MISSING | Retry nhanh 1 lần với lỗi transient, cho phép reconnect thủ công | P1 |
| CONN-014 | Duplicate connection | MISSING | Clone profile (không clone password mặc định), rename hậu tố Copy | P2 |
| CONN-015 | Folder/group cho connections | MISSING | Group theo project/env (dev/stg/prod) | P2 |
| CONN-016 | Import/export connection profiles | MISSING | Export JSON (không password), import merge có conflict handling | P2 |
| CONN-017 | Reset data (connections + cache) | DONE | Có confirm 2 lớp, reset xong app về trạng thái first-run | P1 |
| CONN-018 | Prevent UI freeze khi test/save connection | PARTIAL | Action async, nút disabled đúng, luôn có cancel/timeout feedback | P0 |

## 4.2 Database Navigator

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| NAV-001 | Load schema tree theo engine | DONE | SQLite/Postgres/MySQL đều có schema->table/view->column | P0 |
| NAV-002 | Manual refresh | DONE | Nút refresh luôn responsive, có spinner + disabled state | P0 |
| NAV-003 | Auto refresh sau DDL | DONE | `CREATE/ALTER/DROP/TRUNCATE/RENAME` refresh trong 1 lần run | P0 |
| NAV-004 | Error panel cho navigator | DONE | Hiển thị lỗi ngắn gọn + action retry | P0 |
| NAV-005 | Empty state | DONE | Khi schema rỗng hiển thị thông điệp rõ nghĩa | P0 |
| NAV-006 | Object count badge | DONE | Badge đồng bộ với tree hiện tại | P1 |
| NAV-007 | Search/filter objects | MISSING | Ô filter realtime theo schema/table/view/column | P0 |
| NAV-008 | Persist expand/collapse state | MISSING | Nhớ trạng thái node theo connection | P1 |
| NAV-009 | Context menu object actions | MISSING | Open Data, Generate SELECT, Copy name, DDL preview | P0 |
| NAV-010 | Drag object name vào SQL editor | MISSING | Drag-drop insert quoted identifier đúng dialect | P1 |
| NAV-011 | Link with editor (focus sync) | MISSING | Cursor object trong editor -> highlight object trong navigator | P1 |
| NAV-012 | Lazy-load per schema | MISSING | DB lớn không bị block vì load toàn bộ columns cùng lúc | P0 |
| NAV-013 | Timeout riêng cho metadata queries | PARTIAL | Timeout per query metadata + partial tree nếu một node fail | P0 |
| NAV-014 | Permission-aware metadata errors | PARTIAL | Thiếu quyền object nào chỉ warning object đó, không fail toàn tree | P1 |

## 4.3 SQL Editor (core UX)

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| SQL-001 | Syntax highlighting theo dialect | DONE | Keyword/string/number/comment rõ ràng, không lag khi file lớn | P0 |
| SQL-002 | Run selection/current/all | DONE | Logic statement detection đúng với quote/comment cơ bản | P0 |
| SQL-003 | Shortcut `Cmd/Ctrl+Enter` chạy query | DONE | Không trigger khi đang composing IME, phản hồi <50ms | P0 |
| SQL-004 | Autocomplete keyword | DONE | Gợi ý theo prefix + thứ tự ưu tiên hợp lý | P0 |
| SQL-005 | Autocomplete schema/table/view/column | DONE | Dùng catalog navigator + fallback keyword | P0 |
| SQL-006 | Keyboard navigation trong popup gợi ý | PARTIAL | Up/Down/Tab/Enter/Esc hoạt động nhất quán mọi trường hợp | P0 |
| SQL-007 | Completion details (type/signature/preview) | PARTIAL | Item hiển thị kind + preview cột cho table/view | P1 |
| SQL-008 | SQL formatting command | MISSING | Nút + shortcut format theo dialect, preserve comment | P0 |
| SQL-009 | SQL templates/snippets | MISSING | Snippet `sel`, `ins`, `up`, `cte`, custom template cơ bản | P1 |
| SQL-010 | SQL variables / parameter binding | MISSING | Hỗ trợ `:param` + dialog bind trước execute | P1 |
| SQL-011 | Multi-tab SQL workspace | MISSING | Tạo/đóng/đổi tab script, mỗi tab giữ text + result context | P0 |
| SQL-012 | Script autosave/recover | MISSING | Crash/restart khôi phục tab + unsaved buffer | P0 |
| SQL-013 | Query outline (statement tree) | MISSING | Jump nhanh giữa statement trong script dài | P2 |
| SQL-014 | Explain plan action | MISSING | Nút Explain + tab kết quả plan text trước | P2 |
| SQL-015 | Error annotation inline | MISSING | Lỗi runtime map vị trí statement/caret khi có thể | P1 |
| SQL-016 | Minimap/find-replace nâng cao | PARTIAL | Tìm trong editor có match count + next/prev | P2 |
| SQL-017 | Editor font/line height preferences | MISSING | Settings per user, persist local | P2 |
| SQL-018 | Active connection/schema switch trong editor | MISSING | Đổi datasource/schema không mất text hiện tại | P1 |

## 4.4 Query Execution Engine

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| QRY-001 | Async execution (không block UI) | DONE | Run query không khóa interaction sidebar/editor | P0 |
| QRY-002 | Cancel running query | DONE | Cancel trả trạng thái trong <1s nếu backend hỗ trợ | P0 |
| QRY-003 | Frontend + backend timeout | DONE | Timeout đồng nhất, error message rõ timeout source | P0 |
| QRY-004 | Postgres statement/lock timeout | DONE | Set timeout trước execute, reset per session nếu cần | P0 |
| QRY-005 | Row query paging | DONE | pageSize + offset + hasMore hoạt động ổn định | P0 |
| QRY-006 | Non-row query affectedRows | DONE | DDL/DML trả message nhất quán | P0 |
| QRY-007 | Schema change detection | DONE | Đánh dấu `schemaChanged` đúng cho DDL chính | P0 |
| QRY-008 | Query queue per connection | PARTIAL | Không cho chạy chồng query cùng connection nếu chưa support | P0 |
| QRY-009 | Concurrency nhiều connection | MISSING | Cho phép chạy song song connection khác nhau | P1 |
| QRY-010 | Query retry policy cho lỗi transient | MISSING | Optional retry cho network reset/timeout acquire | P2 |
| QRY-011 | Transaction control (auto/manual commit) | MISSING | Toggle auto-commit + commit/rollback actions | P1 |
| QRY-012 | Parameterized query execution flow | MISSING | Prompt bind params trước khi chạy | P1 |
| QRY-013 | Query profiling metrics | PARTIAL | Hiển thị execution time, rows fetched, page info chuẩn | P0 |
| QRY-014 | Safe guard query nguy hiểm | MISSING | Confirm cho `DROP/TRUNCATE` khi bật chế độ safe mode | P2 |

## 4.5 Result Grid & Data Interaction

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| GRID-001 | Scroll dọc/ngang ổn định | PARTIAL | Không mất scroll sau resize/layout switch | P0 |
| GRID-002 | Sticky header | DONE | Header luôn cố định khi cuộn dọc | P0 |
| GRID-003 | Virtualization cho dataset lớn | DONE | >200 rows virtualized, không drop FPS mạnh | P0 |
| GRID-004 | Rows per page selector | DONE | 100/250/500/1000/2000, persist cache | P0 |
| GRID-005 | Prev/Next page actions | DONE | Disable logic chuẩn ở boundary | P0 |
| GRID-006 | Cell monospace + null rendering | PARTIAL | `NULL` hiển thị nhất quán, format type-aware tối thiểu | P1 |
| GRID-007 | Copy cell/row/selection | MISSING | `Cmd/Ctrl+C` copy grid selection chuẩn tab-delimited | P0 |
| GRID-008 | Advanced copy modes | MISSING | Copy as CSV/JSON/SQL Insert/Markdown | P1 |
| GRID-009 | Column resize | MISSING | Kéo resize cột, min/max width, persist per tab session | P1 |
| GRID-010 | Column reorder | MISSING | Drag header đổi thứ tự hiển thị | P2 |
| GRID-011 | Column show/hide | MISSING | Panel chọn cột hiển thị nhanh | P1 |
| GRID-012 | Sort by column | MISSING | Sort server-side bằng query wrapper nếu khả dụng | P1 |
| GRID-013 | Quick filter by value | MISSING | Right-click value -> apply filter condition | P1 |
| GRID-014 | Open value editor cho text lớn | MISSING | Modal/editor phụ cho JSON/XML/text dài | P1 |
| GRID-015 | Export from result tab | MISSING | Xuất nhanh CSV/JSON từ kết quả query hiện tại | P0 |
| GRID-016 | Fetch all rows guardrail | MISSING | Nút fetch-all có warning với dataset lớn | P2 |
| GRID-017 | Record count total (optional) | MISSING | Action đếm tổng dòng có spinner/cancel | P2 |
| GRID-018 | Result tab pin/rename | MISSING | Đặt tên result theo query comment/title | P2 |
| GRID-019 | Multiple result sets handling | MISSING | Script nhiều statement hiển thị nhiều result hợp lý | P1 |
| GRID-020 | Chart tab cơ bản từ result | MISSING | Bar/line nhanh cho 2-3 cột | P2 |

## 4.6 Data Transfer

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| XFER-001 | Export CSV từ table/result | MISSING | Wizard đơn giản, delimiter/encoding/header | P0 |
| XFER-002 | Export JSON | MISSING | Array JSON + pretty/compact | P1 |
| XFER-003 | Export XLSX | MISSING | Sheet name + include headers | P2 |
| XFER-004 | Import CSV vào table | MISSING | Mapping cột + preview + null handling | P1 |
| XFER-005 | Save export/import config as task | MISSING | Lưu cấu hình chạy lại 1-click | P2 |
| XFER-006 | Background transfer progress | MISSING | Progress bar + cancel + log lỗi | P1 |
| XFER-007 | Export từ query text trực tiếp | MISSING | Chạy query + stream ra file không render full grid | P2 |
| XFER-008 | Error report sau transfer | MISSING | Tạo summary số row thành công/thất bại | P1 |

## 4.7 Query History, Tasks, Observability

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| OBS-001 | Query history local | MISSING | Lưu query gần đây theo connection + timestamp | P0 |
| OBS-002 | Query log view (query manager lite) | MISSING | Lọc theo connection/time/status | P1 |
| OBS-003 | Error log panel | MISSING | Danh sách lỗi gần nhất, click để xem chi tiết | P1 |
| OBS-004 | Clear history | MISSING | Nút clear + confirm | P1 |
| OBS-005 | Re-run từ history | MISSING | 1-click đưa query vào editor và chạy | P0 |
| OBS-006 | Task runner đơn giản | MISSING | Chạy lại export/import saved-task thủ công | P2 |
| OBS-007 | Scheduler | MISSING | Ngoài MVP cơ bản; chỉ nghiên cứu sau | P3 |
| OBS-008 | Telemetry nội bộ (opt-in) | MISSING | Thu metrics UX để tối ưu performance | P2 |

## 4.8 UX, Keyboard, Accessibility, Polish

| ID | Tính năng | Trạng thái DB Pro | Yêu cầu MVP chi tiết (DoD) | Ưu tiên |
|---|---|---|---|---|
| UX-001 | Keyboard-first cho toàn flow | PARTIAL | Connection list/editor/grid đều dùng được không chuột | P0 |
| UX-002 | Focus ring và tab order chuẩn | PARTIAL | Không bị trap focus, modal có focus return | P0 |
| UX-003 | Shortcut map core | PARTIAL | `Cmd/Ctrl+Enter`, `Cmd/Ctrl+L`, `Cmd/Ctrl+F`, `Esc` cancel UI | P0 |
| UX-004 | Consistent loading states | PARTIAL | Skeleton/spinner/message nhất quán mọi panel | P0 |
| UX-005 | Consistent error surfaces | PARTIAL | Toast + inline lỗi + retry action | P0 |
| UX-006 | Empty states chất lượng | PARTIAL | Tất cả màn hình rỗng đều có CTA rõ | P1 |
| UX-007 | Responsive desktop resizing | PARTIAL | Resize cửa sổ không vỡ layout/overflow | P0 |
| UX-008 | Theme tokens ổn định | PARTIAL | Token hóa đầy đủ spacing/radius/shadow/semantic colors | P1 |
| UX-009 | Accessibility tối thiểu | PARTIAL | Role/aria labels cho interactive elements | P1 |
| UX-010 | Undo/redo cho editor + form | MISSING | Form thay đổi có reset/undo rõ ràng | P2 |

---

## 5) MVP đề xuất (bản có thể cạnh tranh workflow cơ bản)

## 5.1 Phạm vi MVP bắt buộc (P0)

### Luồng 1: Connection đến Query đầu tiên trong 30 giây

- Chọn connection có sẵn (sample SQLite) hoặc thêm Postgres/MySQL.
- Test connection trả kết quả nhanh + lỗi rõ nguyên nhân.
- Navigator load ổn định, không treo vô hạn.
- SQL editor autocomplete hoạt động, shortcut run nhanh.
- Result grid có scroll dọc/ngang ổn định, page-size, paging, copy cơ bản.

### Luồng 2: Workflow query hằng ngày

- Run selection/current/all ổn định.
- Cancel query hoạt động thực sự.
- Query history cơ bản (re-run nhanh).
- DDL run xong navigator tự cập nhật.

### Luồng 3: Xuất dữ liệu nhanh

- Export CSV từ query result hoặc table.

## 5.2 P1 ngay sau MVP

- SQL formatter, template snippets.
- Navigator search/filter + context menu actions.
- Grid quick filter/sort + advanced copy.
- Import CSV.
- Error/log panel và query manager lite.

## 5.3 P2/P3 (sau khi ổn định)

- Multi-tab scripts full workspace.
- Task scheduling.
- ER diagram, explain plan, charts nâng cao.
- SSH/SSL/proxy profiles đầy đủ.

---

## 6) Thiết kế UX chi tiết cho panel Connections (theo yêu cầu “tỉ mỉ”)

## 6.1 Information architecture

- Khu trái chia 3 khối rõ:
  - `Connection Actions`: Add, Import, Reset
  - `Connections List`: searchable list + grouping
  - `Connection Health`: status hiện tại (connected/testing/error, latency)

## 6.2 List item spec (mỗi connection card)

- Dòng 1: tên + engine badge + trạng thái chấm màu
- Dòng 2: target rút gọn (host/db hoặc path), hover show full
- Dòng 3: metadata nhỏ (last success, avg latency)
- Actions on hover/focus: edit, duplicate, delete, quick test
- Keyboard:
  - Up/Down: đổi selected
  - Enter: activate connection
  - `E`: edit
  - `Del/Backspace`: delete (nếu cho phép)

## 6.3 Add/Edit modal spec

- Step layout (không nhồi 1 form dài):
  - Step 1: Engine + basic identity (name)
  - Step 2: Endpoint/auth
  - Step 3: Advanced (timeout/ssl/ssh/proxy)
  - Step 4: Test & save
- Password field:
  - reveal/hide icon
  - indicator "stored in keychain"
  - edit mode: để trống = giữ password cũ
- Footer actions:
  - Cancel
  - Test Connection
  - Save
- Validation policy:
  - realtime field-level
  - summary lỗi ở đầu form

## 6.4 Reset & cache behavior

- Reset Data mở confirm 2 bước:
  - Step 1: reset connections?
  - Step 2: reset UI cache (selected connection, draft, page-size)?
- Sau reset:
  - recreate sample DB
  - auto select sample
  - auto load navigator + starter query

---

## 7) Clean Code + Clean Architecture (frontend + Rust)

## 7.1 Frontend target architecture

```
src/
  domain/
    connection/
    query/
    navigator/
  application/
    use-cases/
      run-query.ts
      refresh-navigator.ts
      save-connection.ts
    services/
      query-history-service.ts
  infrastructure/
    tauri/
      connection-gateway.ts
      query-gateway.ts
      navigator-gateway.ts
    storage/
      local-cache.ts
  presentation/
    pages/
      workbench-page.tsx
    features/
      connections/
      navigator/
      sql-editor/
      result-grid/
    components/ui/
```

Nguyên tắc:
- UI component không gọi `invoke` trực tiếp.
- Use-case giữ business flow (timeout, retry, status mapping).
- Gateway/adapter tách transport khỏi domain.
- State machine rõ cho async state (`idle/loading/success/error/cancelled`).

## 7.2 Rust backend target architecture

```
src-tauri/src/
  domain/
    model/
    error/
  application/
    use_cases/
      execute_query.rs
      load_navigator.rs
      save_connection.rs
  infrastructure/
    repositories/
      connection_repo_json.rs
      keychain_secret_repo.rs
    db/
      sqlx_executor.rs
      metadata_reader/
  interface/
    tauri_commands/
  bootstrap/
    app_setup.rs
```

Nguyên tắc:
- `commands` chỉ là transport adapter.
- Business rules nằm ở `application`.
- SQLx + keychain + file storage là infrastructure có trait abstraction để test.
- Error type thống nhất và map về UI contract rõ ràng.

## 7.3 File-size guardrails

- Frontend: file > 250-300 dòng cần tách.
- Rust: file > 300-350 dòng cần tách.
- Mỗi module có `README.md` ngắn mô tả boundary và dependency.

---

## 8) Non-functional requirements cho MVP

## 8.1 Performance SLO

- Open app -> interactive: <= 2.0s (cold), <= 1.0s (warm)
- Test connection feedback: <= 3s (LAN), timeout mặc định 8s
- Load navigator (schema vừa): <= 2s
- Execute SELECT nhỏ: first paint result <= 1.5s
- Scroll grid: 55-60 FPS với 10k rows paged + virtualization

## 8.2 Reliability

- Không được treo vô hạn bất kỳ panel nào.
- Mọi operation network phải có timeout + cancel path.
- Không để unhandled promise rejection gây blank screen.

## 8.3 Security

- Không persist plaintext password.
- Mask secrets trong log/error.
- Sanitize connection URL khi render thông báo lỗi.

---

## 9) Checklist nghiệm thu MVP (ready-to-ship)

## 9.1 Connection

- [ ] Tạo/sửa/xóa connection ổn định cho SQLite/Postgres/MySQL
- [ ] Test connection có timeout + lỗi rõ
- [ ] Password luôn qua keychain
- [ ] Sample SQLite luôn sẵn và auto-select khi first-run
- [ ] Không còn trạng thái navigator loading vô hạn

## 9.2 SQL Editor

- [ ] Run selection/current/all đúng mọi tình huống comment/quote phổ biến
- [ ] Autocomplete có keyboard nav hoàn chỉnh
- [ ] Có SQL formatter
- [ ] Có query history cơ bản

## 9.3 Result Grid

- [ ] Scroll dọc/ngang không mất trong mọi kích thước cửa sổ
- [ ] Rows-per-page hoạt động và persist
- [ ] Copy cell/selection hoạt động bằng phím tắt
- [ ] Export CSV từ result

## 9.4 Stability

- [ ] Query thật PostgreSQL không bị treo UI
- [ ] Cancel query thật hoạt động
- [ ] Timeout xử lý đúng path
- [ ] Không còn blank white screen

## 9.5 Clean architecture & code quality

- [ ] Không còn file “God component”
- [ ] UI/business/invoke đã tách tầng
- [ ] Có test tối thiểu cho use-cases quan trọng (run query, refresh navigator, connection save)

---

## 10) Kế hoạch triển khai đề xuất (8 tuần)

### Sprint 1 (Tuần 1-2): Stability trước

- Chốt lỗi treo query/navigator với DB thật.
- Chuẩn hóa async state + timeout/cancel.
- Sửa triệt để scroll/layout result grid.

### Sprint 2 (Tuần 3-4): Core UX parity

- Navigator search + context actions.
- Result copy/export CSV + column ops tối thiểu.
- SQL formatter + keyboard polish.

### Sprint 3 (Tuần 5-6): Workflow depth

- Query history + re-run.
- Import CSV cơ bản.
- Connection panel advanced UX (step form, health status).

### Sprint 4 (Tuần 7-8): Refactor clean architecture

- Tách tầng frontend + backend theo kiến trúc ở mục 7.
- Thêm regression test cho luồng P0.
- Hardening release checklist.

---

## 11) Danh sách việc nên loại khỏi MVP để giữ tốc độ

- Visual Query Builder
- Task Scheduler
- ER Diagram edit mode nâng cao
- AI assistant nâng cao
- Cloud explorer / ODBC / NoSQL mở rộng

Lý do: các mục này không quyết định chất lượng workflow SQL cơ bản ngay giai đoạn đầu, trong khi chi phí phát triển + rủi ro lớn.

---

## 12) Kết luận

Nếu mục tiêu là “cảm giác dùng chuyên nghiệp như DBeaver ở tác vụ cơ bản”, bản MVP kế tiếp cần ưu tiên tuyệt đối vào:

1. Độ ổn định và phản hồi (không treo, không loading vô hạn, query/cancel chắc chắn).
2. SQL editor workflow (autocomplete keyboard-first + formatter + history).
3. Result grid workflow (scroll/copy/export/filter/sort cơ bản).
4. Connection UX chuẩn công cụ chuyên nghiệp (form, test, health, security).

Khi 4 cụm này đạt chuẩn, sản phẩm sẽ chuyển từ “demo có chức năng” sang “tool có thể dùng hàng ngày”.
