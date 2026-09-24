# ER Diagram — Feature Research and Upgrade Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/er-diagram/`  
**Current source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`

## 1. Mục tiêu

Báo cáo đối chiếu ER Diagram hiện tại với workflow diagram được mô tả trong tài liệu chính thức của DBeaver và JetBrains DataGrip. Mục tiêu:

- tách sidebar activity, central diagram workspace và schema-design draft;
- ưu tiên lỗi stale state có thể làm người dùng xem/điều hướng nhầm schema;
- đánh giá phạm vi tìm kiếm, quan hệ, thao tác canvas và Design Mode hiện có;
- đề xuất thứ tự nâng cấp mà không biến capability của sản phẩm tham khảo thành yêu cầu bắt buộc.

Đây là source/design research, không phải implementation plan hay runtime verification. Chi tiết source nằm trong [ER Diagram Baseline](er-diagram-baseline.md).

## 2. Tài liệu chính thức tham khảo

### DBeaver

- [ER Diagrams](https://dbeaver.com/docs/dbeaver/ER-Diagrams/) — mở diagram cho table/view/schema; diagram editor hỗ trợ arrange, search, cấu hình, export/print, nhiều kiểu routing/notation và generate SQL.
- [Edit mode](https://dbeaver.com/docs/dbeaver/Edit-mode/) — read-only mặc định; edit mode chuyển thao tác schema thành SQL để review trước khi Save, đồng thời có Revert.

### JetBrains DataGrip

- [Database diagrams](https://www.jetbrains.com/help/datagrip/creating-diagrams.html) — tạo diagram theo database object/schema/database, thêm table vào diagram, tìm phần tử, pan/zoom, export PNG/UML và clipboard.
- [Diagram settings](https://www.jetbrains.com/help/datagrip/diagrams.html) — cấu hình scope/layout và mức hiển thị key columns/columns.

Đây là capability reference; tài liệu không chứng minh behavior của DB Pro và không tạo yêu cầu parity.

## 3. Hiện trạng ER Diagram

Theo source baseline `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, sản phẩm đã có:

- central schema graph với table cards, PK/FK markers, data types, FK edges, pan/zoom, table navigation;
- large-schema search mode khi có hơn 200 tables, neighborhood một/hai hop, giới hạn 100 nodes trong neighborhood;
- spatial-index culling, mức chi tiết theo zoom và background layout worker;
- opt-in Design Mode với draft table/column/FK, undo/redo/discard, SQL plan preview và explicit apply qua Query runtime;
- Sidebar “ER diagram” riêng cho schema summary/navigation.

**Gap ưu tiên:** cache graph chỉ so sánh table count và graph version. Schema load/connection reset không invalidates graph; nếu schema mới có cùng số table, canvas có thể tiếp tục dùng node/relationship cũ. Baseline cũng cho thấy Design Mode hiện là draft-create surface dạng panel, chưa bao phủ đầy đủ mục tiêu chỉnh sửa graph/schema của issue [#226](https://github.com/truongnat/db-pro/issues/226). Xem chi tiết source và điều kiện xảy ra tại [baseline](er-diagram-baseline.md).

## 4. Gaps và key features đề xuất

### F1 — Schema-scoped graph invalidation và refresh chính xác

**Hiện tại:**

- `ensure_diagram_graph` rebuild condition chỉ so sánh số nodes và `schema_version`.
- Khi table count đổi, version tăng trước khi worker hoàn tất; nếu frame kế tiếp đến trước matching result, dirty check có thể phát thêm request/version và làm result trước đó stale (`diagram_view.rs:107-153`). Source chưa xác nhận tần suất hoặc độ trễ runtime.

**Nên nâng cấp thành key feature — Schema Identity & Refresh Correctness:**

- Gắn cache key/version với identity thực của datasource + schema/introspection generation; invalidate graph, spatial index và pending layout result khi context đổi.
- Giữ version ổn định cho cùng một schema generation trong lúc worker chạy; chỉ cập nhật generation khi schema input thực sự đổi, để result hợp lệ không bị vô hiệu bởi lần render kế tiếp.
- Bổ sung regression scenario: schema A/B có cùng số tables nhưng tên/columns/FKs khác nhau; sau switch, canvas/search/click chỉ dùng B. Bao gồm switch qua schema rỗng rồi đến schema cùng size.
- Kiểm tra riêng PostgreSQL và SQLite qua connection switch/introspection; không suy luận từ graph unit test.

**Ưu tiên:** P1 — nguy cơ hiển thị hoặc điều hướng tới object của connection/schema khác.

**Acceptance criteria:**

- Sau refresh hoặc connection/schema switch, graph nodes, FK edges, search result và click target đều khớp schema hiện hành, không phụ thuộc số lượng table.
- Layout result cũ không thể ghi đè graph của context mới.
- Khi introspection đang chạy/thất bại, UI phân biệt rõ current data, stale snapshot và empty/error state.

### F2 — Diagram scope, filtering và navigability

**Hiện tại:**

- Graph source là toàn bộ `table_details` đã load; FK edge cần target table tồn tại trong cùng danh sách.
- Schema trên 200 tables vào search mode; tên table/column tạo seed và neighborhood FK giới hạn 100 nodes. “Show all” thoát chế độ đó.
- Canvas có pan/zoom và culling; node click mở table workspace.
- Source hiện tại không thể hiện open diagram cho một table đã chọn với neighborhood-focused scope tùy ý, add/remove table khỏi custom diagram, lưu layout theo diagram, hoặc export/print graph.

**Capability tham khảo:**

DBeaver và DataGrip cho phép mở diagram theo table/view/schema hoặc database; tài liệu DBeaver còn mô tả custom diagram, search, arrange, export/print, và nhiều notation/routing. DataGrip mô tả thêm việc thêm table vào diagram và export PNG/UML. Những capability này gợi ý các workflow scope-aware, không buộc DB Pro phải sao chép toàn bộ editor.

**Nên nâng cấp theo nhu cầu:**

- Ưu tiên mở focused diagram/neighborhood từ table hoặc FK relation, kèm đường quay lại toàn schema.
- Nêu rõ scope đang xem: connection, database/schema và số object trong graph; search result nên chỉ rõ số matches và số nodes/edges đang hiển thị.
- Xem xét lưu layout/filter theo user workspace nếu việc sắp xếp thủ công trở thành workflow; quyết định persistence trước khi thêm custom diagram model.
- Export/print chỉ nên thêm khi có nhu cầu chia sẻ/reporting rõ; chọn format portable, không thêm nhiều format cùng lúc.

**Ưu tiên:** P2 — nâng cao khám phá schema sau khi F1 chặn stale context.

### F3 — Design Mode: mở rộng có kiểm soát và đầy đủ review semantics

**Hiện tại:**

- Draft mode có table/column/FK create, undo/redo, discard và SQL preview/apply.
- Design panel là form/panel độc lập, không cho kéo/thả/sắp xếp node hoặc chỉnh trực tiếp persisted object.
- `DraftColumn` có `is_unique`, nhưng UI tạo column luôn false; mapper lập `ColumnDefinition` chỉ truyền nullable/PK/type/name, không chuyển unique field. Không có UI source-visible để cấu hình multi-column PK/FK, rename/remove column hoặc index.
- Stale fingerprint hiện chỉ hash danh sách `schema.table`; thay đổi columns/FK khi tên tables giữ nguyên không làm fingerprint stale.
- Apply kiểm tra connected rồi dispatch SQL vào Query runtime. Source không chứng minh transaction/atomicity hay provider runtime success.
- Apply xóa draft trước khi gọi `dispatch_query`; handler bỏ qua kết quả bool và luôn báo “applied”. Nếu đang có query chạy hoặc destructive-run gate chặn dispatch, plan có thể mất nhưng chưa được gửi đi (`diagram_design_actions.rs:76-83`; `workspace_actions.rs:42-50`; `events_query_dispatch.rs:110-143`).

**Capability tham khảo:**

DBeaver giữ diagram read-only mặc định; edit mode tạo SQL plan để review, Save mới áp dụng, Revert bỏ draft. Issue [#226](https://github.com/truongnat/db-pro/issues/226) cũng yêu cầu draft thay vì graph mutation trực tiếp, explicit preview/apply, stale-schema guard, undo/redo và composite key support.

**Nên nâng cấp:**

- Giữ read-only/inspect mode mặc định; làm rõ trạng thái draft và mọi side effect trước khi apply.
- Hoàn thiện fingerprint theo introspection generation hoặc canonical schema metadata, không chỉ table-name set; stale detection phải bao phủ columns/FKs liên quan đến plan.
- Chốt transaction, partial-failure, retry và post-apply refresh semantics theo ObjectMutationService/query runtime contract; không tuyên bố atomicity nếu runtime không đảm bảo.
- Chỉ báo apply thành công và clear draft sau khi runtime dispatch chấp nhận request; khi bị chặn, giữ/recover plan và hiển thị nguyên nhân.
- Chỉ mở rộng thao tác draft (rename/remove/keys/indexes/reposition) khi mỗi thao tác có mutation plan, preview, discard/undo và provider capability gate tương ứng.
- Thêm tests cho semantic outputs thực sự: unique/PK/FK definitions được giữ trong generated plan; schema metadata thay đổi khiến draft stale; failed apply không báo thành công và refresh đúng sau success.

**Ưu tiên:** P1 cho fingerprint/context/apply acknowledgment integrity; P2 cho phần thao tác draft còn thiếu nếu create-only là scope được chấp nhận. Không coi issue #226 đã đáp ứng trọn acceptance chỉ dựa trên issue status; source hiện có các giới hạn cụ thể ở trên.

### F4 — Sidebar activity và central workspace có state nhất quán

**Hiện tại:**

Activity rail mở cả Diagram activity và Diagram workspace tab. Explorer connection menu chuyển central tab và có thể connect connection nhưng không đổi `Activity::Diagram`; sidebar vẫn có thể đang ở Explorer hoặc activity trước đó.

**Nên nâng cấp:**

- Chọn semantics rõ: ER Diagram là activity/sidebar độc lập, central workspace tab, hay hai lớp phối hợp.
- Từ mỗi entry point, đảm bảo rail highlight, sidebar content, connection context và active workspace tab nhất quán; nếu giữ sidebar theo user selection thì tab cần tự mô tả context.
- Tránh dùng một nhãn “ER diagram” cho sidebar map summary và central editor nếu thao tác/ownership khác nhau; cân nhắc tên sidebar như “Schema map” nếu đây chỉ là navigator.

**Ưu tiên:** P2 — state/IA consistency, thấp hơn stale graph correctness.

## 5. Roadmap đề xuất

### Phase 1 — Bảo vệ tính đúng của graph

1. F1 — Context-aware invalidation và layout result isolation khi đổi/reload schema.
2. Regression scenario cho same-table-count transition, connection switch và empty-schema transition.
3. Xác nhận source graph, search results và click navigation cùng một schema context.

### Phase 2 — Làm rõ phạm vi khám phá

1. F2 — Table-focused neighborhood và scope labels.
2. Quyết định có cần lưu layout/custom diagrams không trước khi thêm persistence.
3. Chỉ thêm export/print khi có use case chia sẻ cụ thể.

### Phase 3 — Đóng contract Design Mode

1. F3 — Stale metadata guard và preview/apply/provider transaction behavior.
2. Hoàn thiện hoặc thu hẹp rõ ràng draft operation set so với #226; không hiển thị affordance chưa được lập plan.
3. F4 — Chuẩn hóa entry-point/sidebar/tab state.

F1–F4 là định nghĩa outcome, không phải cam kết parity. PostgreSQL và SQLite cần được verify độc lập cho mọi mutation/runtime behavior.

## 6. Ranh giới và câu hỏi còn mở

- [ ] Diagram scope là active database/schema, selected table neighborhood hay user-defined subset?
- [ ] Layout/search/zoom state tồn tại per connection, per schema, per diagram hay chỉ trong session?
- [ ] Có nhu cầu export/share/print và format nào thực sự cần?
- [ ] Design Mode hiện create-only có chủ đích hay chưa hoàn thành so với #226?
- [ ] Apply mutation cần transaction nào; thế nào là partial failure, refresh và recovery trên từng provider?
- [ ] Stale schema guard cần cover toàn schema hay chỉ objects được mutation plan tham chiếu?

Đây là design decisions, không chặn kết luận F1: graph cache hiện không phân biệt hai schemas khác nhau có cùng node count và version.

## 7. Evidence status

Source claims dựa trên SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và được ghi chi tiết trong [ER Diagram Baseline](er-diagram-baseline.md). Reference claims dựa trên official docs tại §2. Không chạy build/test, không có native UI traversal/screenshot/recording, và không có PostgreSQL/SQLite runtime verification trong research này.

## Tổng kết bằng tiếng Việt

ER Diagram đã có graph, tìm kiếm cho schema lớn, điều hướng tới table và Design Mode tạo SQL plan. Ưu tiên sửa cache invalidation vì đổi sang schema cùng số bảng có thể giữ graph cũ; sau đó hoàn thiện stale fingerprint, làm rõ phạm vi diagram và chốt mức hỗ trợ Design Mode trước khi mở rộng thao tác.