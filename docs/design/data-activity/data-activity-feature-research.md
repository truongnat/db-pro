# Data Activity — Feature Assessment and Upgrade Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/data-activity/`  
**Current source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`  
**Originating scope:** [Issue #212](https://github.com/truongnat/db-pro/issues/212)

## 1. Mục tiêu

Báo cáo giải thích vì sao DB Pro có **Data** activity trên sidebar, hiện activity đó thực sự làm gì, và liệu nó có đáng giữ là một điểm điều hướng cấp một hay nên nhập vào Explorer/Quick Open.

Đây là source/product assessment, không phải implementation plan, runtime verification, hay cam kết parity với DBeaver/DataGrip. Các claim hiện trạng có line anchors trong [Data Activity Baseline](data-activity-baseline.md).

## 2. Nguồn chính thức và product authority

### DB Pro

- [Issue #212 — Data Activity: Recent, Pinned, and Reusable Dataset Navigation](https://github.com/truongnat/db-pro/issues/212) xác định mục tiêu: quay lại nhanh các bảng/kết quả thường dùng, **không tạo Data Editor hoặc Result Grid thứ hai**. Issue nêu thêm acceptance rằng bảng trùng tên ở nhiều connection phải không nhập nhằng.
- [Phase G — Productivity, Search & Personalization](../../goals/goal-phase-g-productivity.md) đặt Pinned/Recent objects vào Search và activity liên quan. Phần “current implementation” trong goal là baseline cũ hơn; dùng source SHA ở báo cáo này và issue #212 làm căn cứ cho implementation hiện tại.
- [Goal #182 — Database IDE product direction](https://github.com/truongnat/db-pro/issues/182) yêu cầu UI → typed action/service/provider boundaries và không ghép SQL PostgreSQL/SQLite trong renderer/view code.

### DBeaver và JetBrains DataGrip

- [DBeaver Data Editor](https://dbeaver.com/docs/dbeaver/Data-Editor/) — Data Editor là tab trong Database Object Editor hoặc Results cho custom SQL. Đây là surface xem/chỉnh sửa dữ liệu, có filter/sort, paging/fetch, save/cancel và row operations.
- [DataGrip Data Editor and Viewer](https://www.jetbrains.com/help/datagrip/data-editor-and-viewer.html) — mở data của database object từ Database Explorer; query results cũng hiển thị trong Result tab. Data Editor hỗ trợ browse/filter/sort/edit/export.
- [DataGrip Database Explorer](https://www.jetbrains.com/help/datagrip/database-explorer.html) — quản lý cây data source/schema/table; mở data editor từ object.

Các tài liệu này mô tả data grid là object/result editor được mở từ Explorer hoặc query. Chúng là capability references, không chứng minh DB Pro runtime và không quyết định một sidebar activity riêng là đúng/sai.

## 3. Vì sao có Data trong sidebar?

Issue #212 trả lời trực tiếp: đây là **fast-return activity** cho bảng và kết quả thường dùng, không phải một grid mới. Current source chỉ triển khai hai nhóm **Pinned Tables** và **Recent Tables**; row menu cho mở data, mở structure, tạo query cho bảng, pin/unpin và bỏ khỏi recent. Mở row đưa người dùng đến canonical Table workspace.

Phân biệt ba UI khái niệm:

1. **Data activity** trên rail — sidebar quick access cho pins/recent.
2. **Table Data view** — central workspace thực sự tải và hiển thị rows của bảng.
3. **Explorer** — database object tree dùng để tìm bảng/schema lần đầu.

Queries là SQL-document workspace riêng. Data activity chỉ mở Query bằng một draft khi người dùng chọn **New query for table**.

Nói ngắn gọn: Data có mặt để giảm đường quay lại bảng thường dùng. Tên activity dễ gây hiểu lầm vì nó nói “Data”, trong khi nội dung activity là “Pinned & Recent Tables”.

## 4. Đánh giá hiện trạng

| Tiêu chí | Đánh giá source-based | Evidence / tác động |
|---|---|---|
| Có mục đích sản phẩm rõ | Có | Issue #212 nêu fast-return và cấm duplicate grid; sidebar copy “fast reopen”. |
| Khác với Data Editor | Có | Sidebar chỉ là danh sách; click mở `WorkspaceTab::Table` và `TableView::Data` ở central workspace. |
| Khác với Explorer | Một phần | Explorer là full connection/schema tree; Data hiển thị recent/pinned. Cùng thao tác mở data/structure vẫn có ở Explorer. |
| Khác với Quick Open | Yếu | Palette hiển thị cùng pinned/recent list; Data thêm một surface luôn hiện và thao tác quản lý trực tiếp. |
| Label khớp nội dung | Yếu | “Data” nghe như grid; sidebar thực tế không chứa grid. Central workspace cũng có `TableView::Data`. |
| Target identity an toàn | Không đạt | Pins/recent là bare table-name strings; action dùng active connection/schema. Issue #212 acceptance yêu cầu phân biệt duplicate names across connections. |
| Coverage issue #212 | Một phần | Recent/pinned table flow hiện diện; recent query results, export reference, copy qualified name không có trong `SidebarDataAction`. |

### Kết luận sản phẩm

Data activity **không phải tab thừa về mặt chức năng**, vì nó biến pin/MRU thành lối vào hiển thị thường trực thay vì chỉ nằm trong Explorer tree hoặc một Quick Open popup. Việc dùng chung canonical Table workspace cũng tuân thủ đúng ý định “không duplicate Data Editor”.

Tuy nhiên, **“Data” chưa phải tên tốt cho sidebar hiện tại** và feature bị trùng capability với Quick Open. Nếu muốn giữ fast-return thường trực, giữ activity nhưng đổi nhãn theo nội dung — **Tables** hoặc **Pinned & Recent** — và sửa identity scope trước. Nếu mục tiêu là activity rail thật gọn, chuyển hai sections vào Explorer và giữ Quick Open; không xây một Data Editor thứ hai để hợp thức hóa tên.

Khuyến nghị mặc định: **giữ feature, đổi cách định vị; chưa mở rộng scope trước khi object identity được sửa.**

## 5. Các gaps cần xử lý

### F1 — Table reference phải giữ connection + schema — P1

**Hiện tại:** `pinned_tables` và `recent_tables` là `Vec<String>`, được persist/restore như vectors các tên bảng. `open_table(table)` giải quyết tên theo connection/schema đang active. Khi có `public.users` trên connection A và `audit.users` trên connection B, row hiện chỉ nói `users`; sau khi đổi active context, cùng item có thể mở một object khác cùng tên.

**Nên nâng cấp:**

- Dùng stable object reference tối thiểu `{ connection_id, database/catalog, schema, object_name, kind }` cho pin/recent; reuse cùng identity trong Quick Open.
- Row hiển thị target context (`connection · schema · table`) khi có trùng tên; không dispatch bằng display label.
- Khi connection không tồn tại hoặc object đã bị xóa/đổi schema, hiển thị missing reference và cho remove/rebind; không âm thầm mở same-name object khác.
- Quy định pin scope rõ: global, project, hay per connection. Recent history nên cùng scope được chọn.

**Acceptance criteria:**

- Hai connections có `public.users`/`audit.users` mở đúng target độc lập.
- Đổi active connection không làm thay đổi target của một pin/recent entry.
- Missing connection/table được trình bày rõ; thao tác remove/rebind không xóa nhầm entry khác.
- Persistence qua restart và reconnect giữ nguyên identity/context.
- PostgreSQL và SQLite được xác minh độc lập; schema/database naming semantics không suy diễn giữa providers.

Đây là gap P1 vì target table sai kết hợp với table data editing có thể dẫn đến mutation nhầm database.

### F2 — Định vị rõ activity và tránh trùng lặp không cần thiết — P2

**Hiện tại:** Data là core rail item, nhưng sidebar chỉ có Pins/Recents; Quick Open có cùng entries, Explorer có data/structure opening. Không có search, connection/schema subtitle, grouping hoặc filter trong Data view.

**Nên nâng cấp:**

- Đổi activity tooltip/label thành **Tables** hoặc **Pinned & Recent**; empty state giải thích pin ở Explorer/Quick Open và dữ liệu đang hiển thị được scope thế nào.
- Giữ activity nếu telemetry/use case cho thấy direct persistent list là shortcut được dùng; nếu không, hợp nhất với Explorer + Quick Open thay vì duy trì ba entry points tương tự.
- Khi giữ: hiển thị connection/schema, search/filter khi list tăng, thao tác clear/remove rõ; không biến activity thành bản sao của object tree.
- Đảm bảo selected state liên hệ đúng với central table workspace; cùng table name ở scope khác không được highlight sai.

**Ưu tiên:** P2 — ambiguity và duplication làm giảm discoverability, nhưng không phủ nhận use case fast-return của issue #212.

### F3 — Hoàn thiện reusable dataset references — P2

**Hiện tại:** Issue #212 scope nêu recent query-result references (nếu semantics persistence cho phép), export references và copy qualified name. Sidebar action model hiện chỉ có table name, không có result/export/copy-qualified-name variants.

**Nên nâng cấp theo thứ tự:**

1. Bổ sung **Copy qualified name** sau khi F1 có structured object identity và provider-aware formatting.
2. Nếu result reference có use case, lưu query/result metadata reference (document/query id, connection, schema, timestamp/label), không lưu row data ngầm.
3. Thêm export shortcut chỉ khi nó gọi canonical Transfer workflow, có target/format/progress/error feedback; không dựng exporter riêng trong sidebar.
4. Không giả vờ một query-result “recent item” có thể phục hồi data sau restart nếu chỉ còn metadata và query/context không còn hợp lệ.

**Ưu tiên:** P2; đây là scope expansion, không phải điều kiện để giữ Data activity.

### F4 — SQL draft generation cần identifier-safe boundary — P1 review

**Hiện tại:** `New query for table` ghép raw `{active_schema}.{table}` vào SQL text ở `sidebar_activities_view.rs:79-86`; Explorer `OpenQuery` dùng cùng dạng interpolation (`explorer_details.rs:121-124`). Đây không phải provider-aware identifier quoting và nằm trong UI action path, trái với kiến trúc ghi trong Goal #182.

**Failure scenario cần giữ trong review:** tên schema/table có khoảng trắng, reserved word hoặc quote có thể tạo SQL draft sai cú pháp; identifier chứa statement delimiters có thể làm draft thành nhiều statement. Source hiện chỉ chứng minh draft được đưa vào editor, không chứng minh tự động thực thi hay khai thác được qua safety classifier.

**Nên nâng cấp:**

- Gửi typed table reference tới centralized SQL-generation boundary, không format SQL trong activity renderer/reducer.
- Quote từng identifier component theo provider capability; không quote cả `schema.table` như một identifier.
- Kiểm tra name có quote/reserved word/whitespace và dữ liệu độc hại riêng cho PostgreSQL và SQLite; xác minh chính xác safety-classifier behavior trước khi kết luận exploitability.

**Ưu tiên:** P1 review theo SQL-safety architecture; chưa gọi là verified exploit nếu chưa chứng minh execution path.

## 6. Roadmap đề xuất

### Phase 1 — Chính xác target trước khi thêm chức năng

1. F1 — Structured object identity xuyên Data, Explorer và Quick Open.
2. F4 — Đưa query generation qua provider-aware boundary; kiểm tra identifier edge cases.
3. Thể hiện connection/schema target trên từng row và lỗi stale/missing rõ ràng.

### Phase 2 — Chốt vai trò activity

1. Chọn label **Tables** / **Pinned & Recent**, hoặc gộp Sections vào Explorer nếu không cần activity riêng.
2. Thêm search/filter/clear chỉ nếu lượng entry/use case cần; giữ popup Quick Open cho keyboard-first navigation.
3. Cập nhật empty/loading/error states cho list khi connection đổi hoặc object không còn tồn tại.

### Phase 3 — Reusable references theo nhu cầu

1. Copy qualified name.
2. Recent query-result references dưới dạng metadata-only và có reopen semantics xác định.
3. Export shortcut reuse canonical Transfer workflow.

## 7. Evidence status

Current behavior claims dựa trên source SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và [baseline](data-activity-baseline.md). Product intent/acceptance dựa trên Issue #212; architecture constraint dựa trên Issue #182. Capability references lấy từ official docs ở §2. Không chạy build/test, không capture UI, và không thử PostgreSQL/SQLite multi-connection or identifier scenarios trong report này.
