# Schema Compare — Feature Research and Upgrade Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/schema-compare/`  
**Current source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`

## 1. Mục tiêu

Báo cáo đánh giá Schema Compare native hiện tại trên ba outcome khác nhau: structural snapshot/diff, migration planning/apply và key-aware row comparison. Mục tiêu:

- làm rõ chiều Origin/Target của migration và sự khác biệt giữa schema diff với data diff;
- chỉ ra ranh giới của diff model và nguy cơ từ SQL placeholder;
- đối chiếu workflow chính thức của DBeaver và DataGrip;
- đề xuất thứ tự ưu tiên dựa trên correctness, không coi feature parity là yêu cầu mặc định.

Đây là source/design report, không phải implementation plan hay runtime verification. Evidence chi tiết ở [Schema Compare Baseline](schema-compare-baseline.md).

## 2. Tài liệu chính thức tham khảo

### DBeaver

- [Structure and data compare](https://dbeaver.com/docs/team-edition/desktop/Structure-and-Data-Compare/) — phân biệt Schema Compare, Liquibase changelog, Data Compare và Simple Structure Compare; mô tả tập object schema phổ biến như tables, views, columns, PK/FK, indexes, sequences và unique constraints.
- [Data compare](https://dbeaver.com/docs/dbeaver/Data-compare/) — workflow theo source-target pair, column mapping, key configuration, row/fetch limits và SQL export. Tài liệu nhấn mạnh keys cần unique để kết quả chính xác.

### JetBrains DataGrip

- [Schema comparison and migration](https://www.jetbrains.com/help/datagrip/schema-comparison-and-migration.html) — compare hai object cùng loại; script migration làm Target equal Origin; xem property/DDL diff, chọn thay đổi cần migrate, chỉnh script trước khi execute.
- [Compare data](https://www.jetbrains.com/help/datagrip/compare-data.html) — compare contents của table/view/query results và cấu hình giới hạn số rows.

Các nguồn này là workflow reference; không chứng minh behavior của DB Pro, và không yêu cầu sao chép toàn bộ capability thương mại.

### Ranh giới theo mục tiêu nội bộ

[Goal Phase F cho Schema Compare/Migration](../../goals/goal-phase-f-compare-migration.md) yêu cầu ghi “not compared” cho object category chưa được hỗ trợ và cấm sinh migration từ diff một phần. Goal ưu tiên structural compare provider-neutral trước khi mở rộng migration; data compare là capability giới hạn ở phase sau. Đây là nguồn định hướng sản phẩm, tách biệt với các ví dụ tham khảo thương mại.

## 3. Hiện trạng Schema Compare

Theo source baseline `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, workspace đang gộp các luồng sau:

- một schema snapshot trong app state; diff tables, views, routine identities và column name/type/nullability changes giữa snapshot với Explorer schema hiện tại;
- migration plan có operation risk/warnings, SQL preview, destructive checkbox và ExecuteDdl trên active connection;
- data compare giữa active connection và target ID nhập tay, key-based sample 1.000 rows, row-state counts/filter và sync-SQL comment preview-only.

**Gap P1:** SQL placeholder của CREATE TABLE/ADD COLUMN có thể tới Apply dù được đánh dấu supported. Core planner cũng có placeholder CREATE INDEX, nhưng UI adapter luôn truyền danh sách index diff rỗng nên nhánh đó hiện không thể tới từ UI này. Structural diff còn bỏ qua thuộc tính schema mà không báo thiếu coverage; adapter loại nullability changes và không chuyển view/routine/index deltas sang planner. Xem evidence và line anchors trong [baseline](schema-compare-baseline.md).

**Lưu ý scope:** Goal Phase F xếp keyed Data Compare vào capability giới hạn ở giai đoạn sau, nhưng view hiện tại đã nhúng row comparison theo sample. Tách bằng chứng/safety của luồng này khỏi schema migration; không dùng row-diff panel làm tiêu chí xác minh migration.

Direction quan trọng: snapshot được đưa vào planner làm source, schema hiện tại làm target; planner mô tả mục đích là đưa Target về Source. Apply vì vậy định hướng current schema về snapshot, trên connection đang active lúc apply. UI cần nói rõ chiều này để tránh hiểu nhầm.

## 4. Gaps và key features đề xuất

### F1 — Chỉ áp dụng migration có DDL đầy đủ và đã được kiểm chứng

**Hiện tại:**

- CREATE TABLE chỉ tạo `id INTEGER PRIMARY KEY`; ADD COLUMN dùng `TEXT`. Core planner còn có CREATE INDEX placeholder, nhưng UI adapter hiện đặt index diff rỗng nên chưa đưa nhánh đó vào migration preview. Hai loại placeholder DDL có thể tới từ UI vẫn được đánh dấu provider-supported; Apply không chặn warning.
- Apply xác nhận destructive plan, nhưng không chặn operation chưa hoàn thiện. Migration preview fingerprint chỉ so plan với fingerprint đã lưu của chính plan; không kiểm tra target DB có thay đổi từ lúc preview.
- Migration plan không được bind với connection/provider preview: apply dùng active connection id tại thời điểm click, trong khi compare state còn sau khi chuyển connection (`schema_compare_state.rs:75-88,119-140`; `query_session.rs:59-82`; `schema_workspace_state.rs:6-12`; `explorer_navigation.rs:209-223`).
- Planner có thể generate SQL theo snapshot->current direction nhưng view chưa trình bày rõ “Apply to active connection X, target will be made to match snapshot Y”.

**Nên nâng cấp thành key feature — Executable Migration Safety:**

- Chặn Apply nếu có placeholder/incomplete SQL hoặc unresolved warning; supported phải có nghĩa là SQL thật, đủ definition và capability đúng provider.
- Lấy canonical object metadata từ source snapshot (hoặc yêu cầu chọn Origin/Target live pair), sinh DDL đầy đủ cho columns, keys, FKs, indexes và dependencies trước khi bật Apply.
- Kiểm tra target identity và schema generation/fingerprint tại thời điểm apply; stale target yêu cầu diff/re-plan.
- Nêu rõ Origin/Target, active target connection, destructive operations và collateral/dependency impact trong review.
- Thực thi qua mutation pipeline có failure result theo operation; chỉ báo thành công/clear preview sau khi runtime chấp nhận và DDL hoàn tất.

**Ưu tiên:** P1 — hiện tại user có thể thực thi placeholder DDL và tạo schema sai.

**Acceptance criteria:**

- Không có placeholder/comment-marker DDL nào tới ExecuteDdl.
- Chặn Apply nếu connection id hoặc provider hiện hành khác với target context đã ghi trong plan; không tự retarget migration.
- Mỗi operation trong preview tương ứng với đầy đủ metadata của Origin và capability của provider; operation thiếu thông tin bị block với lý do.
- Target khác hoặc đã đổi schema kể từ preview không được apply im lặng.
- Failed/partial apply giữ đủ operation/error evidence; completed apply refreshes, invalidates old diff/plan and cannot be replayed from stale preview.
- PostgreSQL và SQLite được verify độc lập; SQLite unsupported alter phải có strategy hoặc được chặn rõ ràng.

### F2 — Canonical structural diff với trạng thái “partial” trung thực

**Hiện tại:**

- Table presence, column presence, type/nullability, view identity và routine `(schema,name,type)` identity được so sánh.
- PK/FK flags/definitions, unique/check constraints, indexes, defaults, column order, trigger definitions, view/routine definitions và schema additions/removals chưa được diff trong UI path.
- UI diff có thể nói “Schemas are identical” trên phần subset này. Migration adapter map ít thông tin hơn nữa: nullability/view/routine changes bị rơi; index diffs luôn rỗng.

**Nên nâng cấp:**

- Dùng canonical snapshot/diff domain model có stable identity, object kind, full metadata và explicit `Supported / Unsupported / NotCompared` coverage theo provider.
- UI chỉ dùng “identical” nếu mọi category trong scope đã so sánh; nếu thiếu metadata thì hiển thị “no differences detected in compared properties” cùng danh sách category chưa kiểm tra.
- Một feature/capability không được biến thành migration operation cho tới khi diff tạo đủ source/target metadata.
- Bổ sung boundary cases: column rename-like change, PK/FK/unique/check/index/default, type vs nullability, routine signature/body, views/trigger và schema names.

**Ưu tiên:** P1 trước khi gọi kết quả là đầy đủ migration evidence; P2 nếu view chỉ là quick comparison có giới hạn và copy cảnh báo phạm vi.

### F3 — Snapshot lifecycle, direction và plan invalidation minh bạch

**Hiện tại:**

- Chỉ có một snapshot in-memory; label lưu tên connection/time, không lưu stable connection id.
- Planner hướng current Target về snapshot Source. UI hiển thị “Tables only in snapshot (Removed in current)” và “Tables only in current (Added in current)” nhưng không gắn tên connection và desired migration direction xuyên suốt flow.
- Sau DDL completion, schema refresh không invalidate old diff, migration plan, preview SQL hoặc destructive checkbox. Apply có thể bấm lại trên plan cũ.
- Thay snapshot mới cũng giữ diff/plan/preview cũ cho tới lần diff tiếp theo (`schema_compare_state.rs:50-72`).
- Connection switch không xóa/rebind compare state; plan cũ còn tồn tại dù active target đổi.

**Nên nâng cấp:**

- Chọn semantics rõ: snapshot lịch sử local để rollback, hay Origin/Target compare giữa hai live connections. Nếu hỗ trợ cả hai, làm rõ snapshot provenance và target binding.
- Cho phép đổi hướng Origin/Target rõ ràng; preview header luôn gọi tên source, target connection/schema và hướng sync.
- Invalidate diff/plan/confirmation khi snapshot, current schema, active target, provider hoặc DDL completion đổi.
- Thêm clear/replace/export snapshot lifecycle nếu snapshot trở thành artifact sản phẩm; không thêm persistence trước khi ownership/retention policy rõ.

**Ưu tiên:** P1 cho stale target/plan safety; P2 cho snapshot history/export UX.

### F4 — Key-aware Data Compare result chính xác và có thể đọc

**Hiện tại:**

- Source mặc định là connection đang active; target là ID nhập tay. Schema/table/key fields cũng nhập tự do.
- Service sample tối đa 1.000 rows mỗi side theo thứ tự key; `truncated` được báo. Duplicated comparison key trong sample gây validation error.
- Data Compare controls chỉ xuất hiện sau khi đã có structural diff; central empty state trả về trước phần row comparison (`schema_compare_view.rs:89-101,260-284`).
- Result panel hiển thị counts và state/key cho tối đa 200 row examples nhưng không render cell-level `column_changes`; sync preview tối đa 50 entries là comment-only.
- `DataDiffLoaded.request_id` bị bỏ qua; completion muộn có thể ghi đè result của request mới hơn.

**Nên nâng cấp:**

- Dùng connection picker hiển thị tên/driver/database cho source/target thay vì yêu cầu copy connection ID; xác nhận cùng schema/table mapping hoặc báo unsupported.
- Chọn key từ PK/unique metadata khi có; nếu user nhập custom key, kiểm tra duplicate và nêu rõ uniqueness requirement.
- Hiển thị changed columns với source/target values theo policy bảo mật; nói rõ counts của sample khác với total table row counts.
- Gắn request ID/context pair vào pending state; bỏ response cũ sau khi source/target/schema/table/key đổi hoặc request mới bắt đầu.
- Tách Data Compare khỏi structural-diff empty state để người dùng có thể bắt đầu row compare độc lập.
- Giữ sync preview rõ ràng là illustrative nếu chưa có đầy đủ executable SQL; không gọi đó là export/apply.

**Ưu tiên:** P2 — cải thiện reliability/diagnosability sau F1/F2; request-result mismatch cần sửa trước khi nhiều request đồng thời được hỗ trợ.

## 5. Roadmap đề xuất

### Phase 1 — Chặn migration sai

1. F1 — Placeholder/incomplete DDL không được apply; target identity/freshness và provider capabilities được kiểm tra.
2. F3 — Hiển thị rõ Origin/Target và hướng current-to-snapshot; invalidate plan sau successful apply.
3. Regression scenarios cho table/column/index placeholder, destructive confirmation, stale target và SQLite unsupported operation.

### Phase 2 — Làm diff coverage trung thực

1. F2 — Mở rộng canonical metadata diff; tách `identical` khỏi “not compared”.
2. Bảo đảm planner chỉ nhận deltas có đủ metadata để tạo SQL.
3. Đối chiếu migrations PostgreSQL/SQLite độc lập với expected schema after apply.

### Phase 3 — Data compare workflow rõ ràng

1. F4 — Connection picker, key selection/uniqueness, cell-level review và sample semantics.
2. Giữ response đúng request/context và làm rõ preview-only sync semantics.
3. Chỉ thêm export/apply data sync khi row/column mapping, transaction, confirmation và conflict behavior đã có contract.

F1–F4 là outcome proposals, không phải cam kết parity. Không claim migration/data sync safe cho provider nào nếu chưa có runtime evidence provider đó.

## 6. Ranh giới và câu hỏi còn mở

- [ ] Snapshot là baseline để khôi phục schema cũ hay Origin trong compare giữa hai connections?
- [ ] Migration target luôn là connection active hay target được chọn riêng trong flow?
- [ ] Có cần snapshot persistence/export; ownership theo session, workspace hay project?
- [ ] DDL planner sẽ lấy full object definition từ source DB snapshot hay schema files?
- [ ] Category nào phải được hỗ trợ trước khi UI có thể kết luận “identical”?
- [ ] Data Compare chỉ sampling để inspect hay sẽ có executable sync/export? Nếu sau này sync: transaction/rollback/conflict policy nào?
- [ ] Với PostgreSQL và SQLite, capability/atomicity/rebuild strategies cụ thể ra sao?

Những quyết định này không chặn F1: placeholder SQL hiện có đường apply dù chưa đại diện schema metadata.

## 7. Evidence status

Source claims dựa trên SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và được chi tiết hóa trong [Schema Compare Baseline](schema-compare-baseline.md). External references tại §2 là official DBeaver/JetBrains docs. Không chạy build/test, không có native UI traversal/screenshot, và không có PostgreSQL/SQLite runtime verification trong research này.

## Tổng kết bằng tiếng Việt

Schema Compare hiện ghép snapshot/diff, migration DDL và row compare vào cùng một workspace. P1 là migration có thể apply CREATE/ALTER placeholder thay vì schema thật; ngoài ra diff chỉ kiểm tra một phần metadata và Data Compare dùng sample. Chặn DDL chưa đầy đủ trước, rồi làm rõ Origin/Target, diff coverage và lifecycle của kết quả.