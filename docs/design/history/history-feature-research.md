# History — Feature Research and Upgrade Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/history/`  
**Current source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`

## 1. Mục tiêu

Đánh giá native **History** activity và toàn bộ các đường query-history liên quan: sidebar list, structured execution records, output History pane, Quick Open, và meta-store repository. Mục tiêu là xác định người dùng thực sự xem được lịch sử nào, có thể phục hồi/replay an toàn không, và product goal muốn đặt History ở đâu.

Đây là source/product research, không phải implementation plan, runtime verification, hay cam kết parity. Bằng chứng hành vi hiện tại nằm trong [History Activity Baseline](history-baseline.md). Query Activity baseline đã phân biệt local history với output History dock; báo cáo này mở rộng phần lưu trữ/API và gom các surface dưới góc nhìn History.

## 2. Product intent và tài liệu tham khảo

### DB Pro

- [Phase G — Productivity, Search & Personalization](../../goals/goal-phase-g-productivity.md) đặt Query History trong Queries activity, mô tả history có filter + favorites và gọi implementation hiện tại là “Local dual” (20 in-memory + 500 persisted).
- [Full Product Goal](../../goals/goal-full-product.md) đặt Open Queries, Saved Queries, Query History, Snippets và Scratch dưới Queries; kiến trúc lưu trữ liệt kê history ở SQLite meta store, còn eframe storage cho UI-local state (`§3.1–3.2`, `§4.6`). Hai tài liệu cùng thống nhất placement dưới Queries nhưng để lại khác biệt về authority/persistence của execution history.
- [Query Activity Baseline](../query-activity/query-activity-baseline.md) là sibling scope cho Queries activity, saved-query library và local-history sidebar; không xem đây là một implementation/acceptance plan đã được duyệt.

### DBeaver

- [Query Manager](https://dbeaver.com/docs/dbeaver/Query-Manager/) mô tả execution log có timing, affected rows và errors; filters theo loại query, thời gian, connection/catalog/schema; record display limit, log file, retention và clear history. Một số filter/retention/history features chỉ có ở một số edition; đây là reference, không giả định DB Pro phải copy toàn bộ.

### JetBrains DataGrip

- [Find recent queries and files](https://www.jetbrains.com/help/datagrip/find-recent-queries-and-files.html) phân biệt query history theo data source (có text search, chọn query để đưa vào editor) với Local History để phục hồi query đã gõ nhưng chưa chạy. Phân biệt “Open/insert” với “Replay/execute” là một ranh giới hữu ích cho DB Pro.

Các official docs là workflow references, không phải bằng chứng behavior của DB Pro.

## 3. Hiện trạng History

Ở SHA trên, tên **History** có thể chỉ ít nhất bốn thứ khác nhau:

1. **History rail activity**: saved queries và tối đa 15 SQL strings cục bộ, không phải execution log; cùng library sections cũng xuất hiện trong Queries activity.
2. **Structured UI execution history**: tối đa 500 `UiQueryHistoryEntry` có status/context/counts/errors; được lưu trong eframe storage và hiển thị ở output dock.
3. **Quick Open history results**: tối đa 30 entries từ structured UI list, nhưng lấy từ đầu list nên là oldest retained records.
4. **SQLite meta-store history**: `QueryService` ghi successful query records vào `meta.db`; `QueryApi.history` expose list per connection, nhưng không có UI call site trong source search.

Hai vấn đề trực tiếp ảnh hưởng user:

- History activity không cho xem/recover metadata của execution history; string list của nó không được lưu qua hook eframe đã thấy, trong khi structured list sống ở output dock/Quick Open.
- History sidebar click thay thế text buffer đang active mà không dùng connection/schema gốc hay dirty confirmation; ngược lại output-pane “Replay” tự tạo document theo context cũ và lập tức thực thi. Hai thao tác có cùng tên miền “history” nhưng semantics khác nhau đáng kể.

Chi tiết action flow, retention và line anchors: [baseline §1–4](history-baseline.md).

## 4. Gaps và key features đề xuất

### F1 — History entry có một source of truth và đúng lifecycle

**Hiện tại:**

- Có ba hình thức lưu: `query_history` trong memory (20 distinct SQL strings), `query_history_entries` trong eframe storage (cap 500, gồm failed/cancelled), và `query_history` table trong SQLite meta store (successful service results; source path không có pruning/clear).
- Core `QueryHistory` không biểu diễn failed/cancelled/error detail như `UiQueryHistoryEntry`. Meta store có `history()` read API nhưng UI không gọi; UI lại hiển thị local eframe collection.
- Native worker gọi QueryApi với database/schema là `None` cho single-query UI execution, nên meta-store row trên đường này không nhận các context fields mà UI history entry giữ (`worker.rs:1545-1551`).
- Run success/failure/cancel và multi-statement có thể để lại kết quả không giống nhau giữa meta-store history và UI history.

**Nên nâng cấp:**

- Product docs chưa thống nhất source: Full Product Goal đặt query history ở meta store, Phase G baseline nói local dual 20+500. Khuyến nghị chốt một canonical execution-history record; dùng SQLite meta store làm source of truth theo kiến trúc, eframe chỉ giữ filter/selection/view state.
- Định nghĩa một record contract gồm stable id, SQL, connection/database/schema, started-at/duration, outcome, row/affected-row counts và error fields. Ghi rõ semantics cho failure, cancellation, multi-statement partial/error result và best-effort persistence.
- Wire History activity, output pane và Quick Open vào cùng data source; UI views có thể khác nhưng không được xây ba history lists độc lập.
- Bump meta-store schema migration khi thêm fields; giữ reads/writes qua `QueryHistoryRepository`/application boundary.

**Ưu tiên:** P2 — các views không cùng data source, retention hay outcome coverage; Activity History không phải execution log đầy đủ.

### F2 — Open, insert và replay phải là actions khác nhau

**Hiện tại:**

- Activity history row thay thế nội dung document hiện tại bằng SQL string và giữ context đang có; source path không kiểm tra dirty state.
- Output History pane có **Replay**, tạo document mới có connection/schema cũ rồi tự chạy SQL ngay.
- Meta history list theo connection, nhưng không được UI sử dụng.

**Nên nâng cấp:**

- Dùng stable history entry id; `Open` tạo document mới với source context, `Insert` là action riêng trên active editor, `Copy` chỉ copy SQL, và `Replay` là action thực thi rõ ràng.
- Không replace dirty document ngầm. Nếu có replace action, đặt tên riêng và dùng dirty guard.
- Trước replay, kiểm tra recorded connection còn tồn tại/đã connect và khôi phục database/schema nếu hợp lệ; nếu không, yêu cầu chọn context thay vì lặng lẽ dùng connection hiện hành.
- Giữ SQL safety policy/confirmation pipeline hiện có; lịch sử không phải đường tắt để né guard cho mutation.

**Ưu tiên:** P2 — History sidebar có thể thay SQL đang soạn mà không hỏi trước hoặc restore source context; Replay cũng nên phân biệt rõ với thao tác chỉ mở query.

### F3 — Recent phải thật sự recent; outcome và preview phải trung thực

**Hiện tại:**

- Quick Open chỉ lấy 30 entry đầu của list được append theo thời gian; nó bỏ qua 470 entry mới hơn khi list đủ 500.
- Output pane hiển thị Cancelled giống Success/“OK”.
- Preview cắt UTF-8 string ở byte offset 117, có thể panic khi ranh giới cắt nằm trong multibyte character.

**Nên nâng cấp:**

- Dùng ordering có contract rõ (newest-first), sort theo `started_at`/sequence thay vì index không ổn định; Quick Open lấy mới nhất, không lấy record cũ nhất.
- Render ba outcome riêng: success, failed, cancelled; đưa timestamp, connection/database/schema và lỗi vào result details.
- Truncate bằng character boundary-safe API; thêm test cho multi-byte SQL crossing preview cutoff.
- Test retention boundary, duplicate SQL behavior, partial multi-statement outcome, and Quick Open selecting the intended history ID.

**Ưu tiên:** P1 cho UTF-8 panic; P2 cho recency/status/metadata presentation.

### F4 — History search, filters, retention và privacy rõ ràng

**Hiện tại:**

- History sidebar không filter; output pane filter SQL text nhưng không lọc status/time/connection/schema. Meta-store `list` nhận connection + limit nhưng contract không có retention/delete.
- Meta-store lưu query text; UI-local structured entries cũng lưu SQL trong eframe storage. Không thấy UI clear/export/retention settings trên các History surfaces.

**Nên nâng cấp:**

- Search SQL theo text và filters theo connection, database/schema, thời gian và outcome; hiển thị context để người dùng phân biệt hai query giống SQL nhưng khác target.
- Chốt retention policy theo count/time và clear/export behavior; hỗ trợ xóa theo phạm vi có thông báo cụ thể.
- Nêu rõ SQL được lưu local trong meta store, duration, retention và phạm vi backup; không tự đưa history text vào global search index nếu chưa có privacy decision.

**Ưu tiên:** P2 — discoverability, storage growth và query-text retention; cần owner quyết định retention/privacy thay vì mặc định vô hạn.

### F5 — Đặt History ở đúng activity theo information architecture

**Hiện tại:**

- `Activity::History` ở Tools/Management rail chỉ chứa saved queries + local history; Queries activity cũng hiển thị saved queries + history.
- Hai product goals đặt Query History thành một nhóm trong Queries, không mô tả history như tool/activity độc lập.

**Nên nâng cấp:**

- Khuyến nghị nhập Query History vào Queries activity theo full product goal; bỏ duplicate History rail item sau khi mọi entry point được chuyển.
- Giữ một top-level History activity chỉ nếu scope mở rộng thành cross-feature timeline (ví dụ execution + migration/job/audit) với model, filters và retention riêng; không đổi tên cho một query string list.

**Ưu tiên:** P2 — giảm trùng lặp và làm nhãn khớp nội dung; không chặn các fix correctness/safety.

## 5. Roadmap đề xuất

### Phase 1 — Sửa lỗi và bảo vệ SQL

1. F2 — Safe open/replay context, dirty guard, semantics rõ và giữ safety pipeline.
2. F3 — Fix Unicode-safe preview, correct cancelled status, newest-first Quick Open.

### Phase 2 — Một execution history nhất quán

1. F1 — Canonical record contract, meta-store read/write wiring và outcome parity.
2. F4 — Search/filter, count/time retention, clear/export và privacy decision.
3. Migration, cap và restart behavior verify independently; no provider claims inferred from UI/source.

### Phase 3 — Information architecture

1. F5 — Đưa History vào Queries theo product goal hoặc chốt một cross-feature timeline có scope độc lập.
2. Cập nhật Quick Open/search source để dùng canonical records, tránh duplicate adapters.

F1–F5 là đề xuất outcome, không phải feature lifecycle/commitment. Không đổi schema hoặc history persistence trước khi canonical-store, retention, SQL-privacy và migration behavior được chốt.

## 6. Ranh giới và câu hỏi còn mở

- Full Product Goal đặt query history ở meta store, còn Phase G current-implementation row nói local dual 20+500. Owner cần xác nhận cách chuyển vai trò của hai store khi chọn meta store làm canonical execution history.
- Service lưu SQL text nhưng không nhận bound parameter values trong history save call; vẫn cần quyết định policy với literal nhạy cảm mà người dùng tự viết vào SQL (`query_service.rs:166-175`).
- Retention theo max rows, age, per connection hay workspace? Có user-triggered clear/export không?
- Failed/cancelled/partial multi-statement executions được lưu với outcome nào, và execution result failure có đủ metadata để record không?
- `Replay` tự chạy ngay hay mở review/pre-run state trước? Cần xử lý connection/schema đã mất như thế nào?
- History có nằm trong Queries activity hay trở thành cross-feature timeline? Current product goals đưa Query History dưới Queries.
- Có cần local-history cho query đã gõ nhưng chưa chạy? Đây là Local History/recovery scope, khác với execution history của report này.
- PostgreSQL/SQLite integration and eframe restart behavior cần được xác minh riêng; report này chưa làm runtime checks.

## 7. Evidence status

Source claims dựa trên SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và được ghi trong [History Activity Baseline](history-baseline.md). Product scope dựa trên Phase G và Full Product Goal. Workflow references dựa trên official DBeaver/JetBrains documentation tại §2. Không chạy build/test, không có native UI traversal/screenshot, eframe restart check, hoặc PostgreSQL/SQLite runtime verification.

## Tổng kết bằng tiếng Việt

History hiện không phải một execution log thống nhất: rail list là 20 SQL strings trong memory, output pane dùng structured local records cap 500, Quick Open lấy nhầm 30 record cũ nhất, và meta store có một history API chưa được UI dùng. Ưu tiên bảo vệ SQL đang soạn và tách Open khỏi Replay; sửa lỗi UTF-8/status/recency; sau đó thống nhất storage/retention và đặt Query History dưới Queries theo product goal.
