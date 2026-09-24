# Query Activity — Feature Research and Upgrade Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/query-activity/`  
**Current source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`

## 1. Mục tiêu

Báo cáo này đối chiếu Queries activity/sidebar hiện tại với workflow SQL editor được mô tả trong tài liệu chính thức của DBeaver và JetBrains DataGrip. Mục tiêu:

- phân biệt query documents, saved queries, execution history và snippets đang có;
- chỉ ra các lỗi an toàn/độ tin cậy dựa trên source, không suy đoán từ tên menu;
- đề xuất các key feature có thể nâng cấp theo thứ tự;
- giữ rõ ranh giới với central Query Editor và output/data panel đã có tài liệu riêng.

Đây là research/design report; không phải implementation plan, runtime verification, hay cam kết parity với sản phẩm tham khảo. Hiện trạng có source evidence trong [Query Activity Baseline](query-activity-baseline.md).

## 2. Nguồn chính thức đã tham khảo

### DBeaver

- [Script management](https://dbeaver.com/docs/dbeaver/Script-Management/) — lưu script vào project hoặc filesystem, import/export/rename, và phân biệt SQL console không gắn `.sql` file với script đã lưu.
- [Query Manager](https://dbeaver.com/docs/dbeaver/Query-Manager/) — log execution metadata, filters, storage/retention; một số tính năng lịch sử bền vững được phân theo edition.

### JetBrains DataGrip

- [Query consoles](https://www.jetbrains.com/help/datagrip/query-consoles.html) — console gắn với data source, connection session, quản lý/duplicate/rename, và vị trí lưu ngoài IDE project.
- [Scratch files](https://www.jetbrains.com/help/datagrip/scratches.html) — scratch runnable, có code assistance, ở ngoài project và có thể chuyển vào project khi trưởng thành.
- [Find recent queries and files](https://www.jetbrains.com/help/datagrip/find-recent-queries-and-files.html) — query history searchable theo data source; Local History có thể khôi phục cả query đã gõ nhưng chưa chạy.
- [Files tool window](https://www.jetbrains.com/help/datagrip/files-tool-window.html) — quản lý project SQL files, open files, scratches và query consoles trong các view liên quan.

Các nguồn này là capability references để so sánh workflow. Chúng không chứng minh behavior của DB Pro và không phải danh sách bắt buộc phải sao chép.

## 3. Hiện trạng Queries

Theo source baseline `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, Queries activity đã có:

- danh sách query documents đang mở, chọn/duplicate/rename/close và dirty marker;
- tạo query mới và scratch query;
- saved-query library theo folder, mở/copy/rename/delete và xác nhận delete;
- local history list ngắn để đưa SQL vào editor;
- sáu built-in SQL snippets và scratch shortcut;
- reducer/runtime command paths để load, save, rename, delete và refresh saved queries.

**Gap đúng mức độ:** sidebar history chỉ nhận các chuỗi SQL gần đây; structured execution history là một state path khác. Saved query/history click thay thế text buffer hiện tại thay vì mở một query document độc lập; thao tác này không có source-visible dirty confirmation. Context-menu “Open in Editor” không phát action, và “Rename query” không nhận tên mới từ người dùng. Chi tiết và line anchors nằm trong [baseline](query-activity-baseline.md).

Phạm vi này không bao gồm editor command bar, SQL editing, execution/result behavior hoặc output History dock; các phần đó có tài liệu riêng tại [`../query-editor-data-panel/`](../query-editor-data-panel/).

## 4. Gaps và key features đề xuất

### F1 — Query Library phải mở an toàn, giữ connection context

**Hiện tại:**

- Click saved query hoặc recent-history entry gọi `set_active_query_text(sql)` trên document đang active.
- Source path không yêu cầu xác nhận buffer dirty và không mở document mới.
- Các action từ saved-query/history chỉ truyền SQL string vào reducer; reducer gán chuỗi đó vào active document mà không chuyển connection/schema nguồn.
- “Open in Editor” đóng menu mà không phát action.

**Capability tham khảo:**

DataGrip gắn query console với data source/connection session, hỗ trợ nhiều console, duplicate và rename. DBeaver phân biệt script đã lưu với SQL console không có associated `.sql` file. Hai mô hình này đều thể hiện rõ identity/lifecycle của query, thay vì coi query chỉ là một chuỗi SQL.

**Nên nâng cấp thành key feature — Safe Query Open & Context:**

- Saved query click mở document riêng; không thay thế buffer hiện tại. Nếu vẫn cần “Replace current editor”, phải là action có tên riêng và dirty guard.
- Query document mang connection/schema/dialect context có nguồn gốc rõ; history replay dùng metadata gốc khi còn hợp lệ, hoặc yêu cầu chọn context khi không còn.
- Giữ source identity (`saved_query_id` hoặc history entry id) khi mở nếu cần cho Save/Update; không biến replay thành một record mới ngoài ý muốn.
- Làm cho “Open in Editor” thật sự thực hiện open; phân biệt với “Copy SQL” và “Insert into current editor”.
- Rename phải nhận tên đích do người dùng nhập, có validation/feedback từ command result; không dùng folder draft làm tên query.
- Mọi replace/close/rename path phải có semantics rõ với dirty documents.

**Ưu tiên:** P1 — rủi ro trực tiếp mất SQL chưa lưu và thực thi query trong context không cùng nguồn; hai luồng trọng yếu của Query workspace.

**Action đề xuất:**

```text
OpenSavedQuery { saved_query_id, source_connection, source_schema }
OpenHistoryEntry { history_entry_id, source_connection, source_schema }
InsertSqlIntoActiveDocument { sql }
ReplaceActiveDocumentSql { sql, confirm_if_dirty }
RenameSavedQuery { saved_query_id, new_name }
```

**Acceptance criteria:**

- Khi active document dirty, mở saved query/history không làm thay đổi SQL của document đang mở.
- Saved query mở thành document mới với connection/schema nguồn đúng; context đã bị xóa/disconnected được xử lý rõ, không fallback im lặng sang connection khác.
- History replay giữ connection/schema/SQL gốc hoặc yêu cầu người dùng chọn context; không silently dùng context của tab đang active.
- Open in Editor, Copy SQL và Insert SQL là ba kết quả khác biệt, quan sát được.
- Rename dùng tên mới đã nhập; cancel giữ nguyên tên; lỗi runtime không báo thành công giả.
- Kiểm tra PostgreSQL và SQLite độc lập; không suy luận connection/schema semantics của một provider từ provider kia.

### F2 — Searchable Query History & Recovery

**Hiện tại:**

- Queries sidebar hiển thị tối đa 15 entry từ danh sách string, bỏ trùng SQL và giữ tối đa 20 SQL string (`query_execution_actions.rs:143-150`; `sidebar_query_library_view.rs:48-67`).
- Structured execution record có connection/schema, thời gian, duration, status, row count, affected rows và error details; reducer giữ tối đa 500 record (`query_history_events.rs:6-29`).
- Source hiện tại không chứng minh history state tồn tại sau restart. Sidebar history cũng không truy cập structured record.

**Capability tham khảo:**

DataGrip cung cấp history search theo data source, đồng thời Local History có thể phục hồi code chưa chạy. DBeaver Query Manager mô tả metadata, filter theo time/connection/catalog/schema, cùng retention/storage controls; tài liệu nêu một số khả năng persist phụ thuộc edition.

**Nên nâng cấp:**

- Dùng một query-history model thống nhất cho sidebar/editor, không duy trì hai danh sách có hành vi khác nhau.
- Tìm theo SQL; lọc theo connection/schema, thời gian, status và error.
- Row preview cho duration/status/rows; mở history record vào document mới theo F1.
- Quy định retention, clear/export và persistence rõ ràng; nếu history chỉ giữ trong session thì ghi rõ thay vì gọi là history bền vững.
- Tách execution history khỏi recovery của text chưa chạy; recovery cần document/local change history, không thể suy ra từ executed SQL log.

**Ưu tiên:** P1 sau F1 nếu recovery SQL và audit execution là yêu cầu sản phẩm; P2 nếu use case chỉ cần vài câu lệnh gần đây trong session. Chốt retention/privacy/storage policy trước khi bật durable history.

**Acceptance criteria:**

- Search trả query theo substring/case rules được xác định; không làm lộ SQL giữa các connection scope ngoài ý muốn.
- Filter metadata khớp entry; missing/dropped connection được hiển thị thay vì gán context khác.
- Restart behavior đúng với persistence policy đã chọn; clear history thực sự loại bỏ dữ liệu theo contract.
- Query chưa chạy có recovery story riêng, không được tuyên bố khôi phục từ execution history.

### F3 — Saved Query Library có quản lý tên/folder thực sự

**Hiện tại:**

- Library group theo folder và cho collapse/expand; có copy, delete confirmation và một action rename không nhận tên mới.
- Không thấy search/filter query hoặc move query giữa các folder trong sidebar action model.
- Tạo folder/delete folder command paths tồn tại ngoài một workflow đầy đủ trong từng query row.

**Capability tham khảo:**

DBeaver lưu script trong Scripts project folder hoặc filesystem và có workflow import/export/rename. DataGrip cho tổ chức query consoles dưới data source bằng directories và di chuyển console. Đây là hai loại nội dung khác nhau; DB Pro cần chọn semantics nhất quán giữa query library record và SQL file.

**Nên nâng cấp:**

- Rename dialog/editor, create/rename/move folder, move query và delete confirmations nhất quán.
- Search query theo name/SQL và filter folder; thao tác open/copy hiển thị title/source context.
- Làm rõ saved query thuộc connection nào, có được mở/offline hay không, và behavior khi active connection đổi.
- Nếu thêm import/export/file-backed script, phân biệt nó với saved-query record; không âm thầm coi hai loại là tương đương.

**Ưu tiên:** P1 cho sửa rename và hoàn thành menu action; P2 cho search/move/import-export sau khi F1 xác định identity/lifecycle của saved query.

### F4 — Query document, console và scratch có lifecycle rõ

**Hiện tại:**

Sidebar phân biệt New Query và New Scratch, nhưng cả hai đi vào query-document session. Source path tạo scratch bằng active connection/schema và phát feedback; baseline này không xác nhận persistence sau restart hay file-backed lifecycle.

**Capability tham khảo:**

DataGrip tách query consoles gắn datasource khỏi scratch files ở ngoài project; Files tool window quản lý cả hai. DBeaver phân biệt script có `.sql` file với console không có file, nơi đóng console loại bỏ nội dung; Save có thể chuyển console thành SQL editor/script. Đây là những tradeoff lifecycle khác nhau, không phải một chuẩn duy nhất.

**Nên nâng cấp:**

- Chọn và ghi rõ semantics của Query, Scratch, Saved Query và SQL File: ownership, persistence, connection binding, rename/move/export, close/reopen.
- Chỉ gọi scratch là disposable nếu user được cảnh báo và dirty close có guard; nếu cần phục hồi, chọn persistence scope rõ.
- Không trộn file browser/project navigation vào Queries sidebar nếu Files workspace đã làm việc đó; tích hợp qua Open in Files/Save as file khi có contract.
- Tách query-console context-bound (nếu cần) khỏi SQL file portable để không tạo giả định sai về connection.

**Ưu tiên:** P2 — trước tiên kiểm chứng state restore/close contract hiện có; chỉ thêm loại document nếu use case đòi hỏi.

### F5 — Snippet catalog có thể tìm và tùy chỉnh

**Hiện tại:**

Sidebar hiển thị sáu snippet cố định. Chọn snippet chèn SQL vào active editor; catalog không có user-defined entries hoặc search trong đường sidebar.

**Nên nâng cấp:**

- Thêm search, category và favorite cho snippet; hỗ trợ user snippets sau khi format/variable contract được xác định.
- Provider/dialect tags phải rõ; không gợi ý SQL không hỗ trợ trong connection hiện tại.
- Insert phải phân biệt replace vs append/selection insertion và giữ cursor/selection semantics.
- Giữ built-in examples như một phần của catalog, không thay thế query library bằng snippets.

**Ưu tiên:** P2 — hữu ích cho authoring nhưng thấp hơn safe open/recovery/library correctness.

## 5. Roadmap đề xuất

### Phase 1 — Bảo vệ SQL và context

1. F1 — Mở saved query/history thành document an toàn, có source connection/schema.
2. Sửa “Open in Editor” no-op và rename để nhập tên thật; bổ sung command result feedback.
3. Chốt dirty-document semantics cho replace/close/open và kiểm tra saved-query lifecycle.

### Phase 2 — Một history có thể tìm và phục hồi

1. Quyết định session-only hay durable history, retention, clear/export và privacy.
2. Cho Queries activity tiêu thụ structured records; thêm search/filter/replay qua F1.
3. Thiết kế recovery cho query chưa chạy riêng với history execution.

### Phase 3 — Quản lý library/document và snippets

1. Folder create/rename/move/search cho saved queries; phân biệt saved record với SQL file.
2. Chốt lifecycle Query/Scratch/Console/File và project/connection scoping.
3. Mở rộng snippets tùy theo nhu cầu authoring/provider.

F1/F2/F3/F4/F5 là feature definitions theo outcome, không phải cam kết chọn mọi capability. Không thêm API, service hay persistence backend trước khi lifecycle, ownership và privacy policy được quyết định.

## 6. Ranh giới và câu hỏi cần giữ mở

- [ ] Query Library là per connection, per project hay user-wide? Source hiện load theo connection hiện tại.
- [ ] Saved query mở read/copy hay update in-place? Identity phải quyết định cùng semantics Save/Save As.
- [ ] Query history có persist qua restart không; retention, export/clear và dữ liệu SQL nhạy cảm được xử lý thế nào?
- [ ] Scratch có session-only, application-wide hay project persistence? Close/restart contract hiện chưa được xác nhận ở đây.
- [ ] History replay sau khi connection bị xóa/đổi schema cần yêu cầu người dùng chọn lại context thế nào?
- [ ] Các behavior source/runtime cần kiểm tra riêng trên PostgreSQL và SQLite.

Các câu hỏi trên là design decisions còn mở, không chặn kết luận F1: thao tác mở hiện tại ghi đè active document text trong source path.

## 7. Evidence status

Source claims dựa trên SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và được ghi rõ trong [Query Activity Baseline](query-activity-baseline.md). External workflow claims dựa trên các official references ở §2. Không chạy build/test, không có UI traversal/screenshot, và không có PostgreSQL/SQLite runtime verification trong research này.
