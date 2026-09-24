# Query Editor và Data Result Panel — Investment Baseline

**Date:** 2026-09-24  
**Source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`  
**Status:** PLANNING / baseline only

## 1. Mục tiêu

Xác định các feature cần đầu tư trước khi bắt đầu code cho Query Editor và vùng data/result. Baseline này ưu tiên nền tảng có giá trị trực tiếp cho mục tiêu sản phẩm: **Query Editor ở trung tâm/trái và dữ liệu kết quả có thể hiển thị ở panel bên phải**.

Đây là scope recommendation, chưa phải implementation plan và chưa đánh dấu feature nào là completed.

## 2. Context đã xác nhận

### Verified từ source

- Query Editor hiện render trong workspace native egui.
- Result hiện nằm trong bottom output dock.
- Output có Results, Chart, Messages, Explain và History.
- Result grid đã có virtualized rows, filter visible rows, sort, column resize/reorder/hide, copy/export, record inspector và value inspector.
- Table-data mode có edit cell, Set NULL, duplicate/delete row, ChangeSet, apply/discard và write policy.
- Query document có connection/schema binding riêng.
- Query execution có run, cancel theo capability, transaction control, diagnostics và history.
- Multi-statement execution có nhiều result sets và active result index.

### Inferred

- Khó khăn lớn nhất hiện tại không phải thiếu result-grid primitive, mà là thiếu một **result surface layout/lifecycle ổn định** cho right-side panel.
- Nếu triển khai compare hoặc detach trước khi định danh document/request/result rõ ràng, state có nguy cơ bị trộn giữa query tab và result tab.
- Nếu bổ sung filter mà không phân biệt client-side/server-side, người dùng có thể hiểu sai phạm vi dữ liệu và hiệu năng.

### Open/unknown

- Product muốn right panel là mode mặc định hay chỉ là tùy chọn?
- Result panel có cần detach thành window thật hay chỉ cần dock trái/phải/bottom?
- Compare phase đầu so sánh result sets, table data, hay schema/object?
- Có yêu cầu persistence layout giữa các phiên hay chỉ trong workspace hiện tại?

Các câu hỏi này không chặn baseline kỹ thuật; chúng chỉ ảnh hưởng thứ tự và mức độ đầu tư của Phase 2.

## 3. Gap và risk cần giải quyết trước

- **[BLOCKER / correctness] Result identity:** result đang được truy cập qua active document + active result index. Right dock, detach hoặc compare cần identity ổn định theo document/request/result, không chỉ theo index.
- **[BLOCKER / UX] Layout semantics:** source hiện là bottom dock; mục tiêu right-side cần một layout contract rõ ràng để editor/output không tranh chấp height/width.
- **[RISK / data] Filter projection:** client filtering/sorting phải giữ original row index để edit/copy/inspect không trỏ nhầm row.
- **[RISK / safety] Edit boundary:** query result phải read-only; chỉ table-data mode mới được ChangeSet/edit theo write policy.
- **[RISK / provider] Capability mismatch:** cancel, parameters, explain, edit và export provider-specific không được suy diễn từ PostgreSQL sang SQLite.
- **[GAP / workflow] Result lifecycle:** chưa có rename, pin, detach, close individual result và compare result.
- **[GAP / navigation] Editor object navigation:** completion/hover đã có, nhưng hyperlink/F4/outline chưa thành feature hoàn chỉnh.

## 4. Investment thesis

Đầu tư trước vào **result surface foundation**, không bắt đầu bằng việc thêm hàng loạt format export hoặc object browser.

Lý do:

1. Right-side data display là mục tiêu UX trực tiếp.
2. Result layout là nền tảng dùng lại cho inspector, compare, chart và explain.
3. Result identity/lifecycle giải quyết rủi ro state trước khi mở rộng compare/detach.
4. Grid primitive hiện tại đã đủ để tạo một vertical slice mà không rewrite renderer.
5. Mỗi phase có thể kiểm chứng độc lập bằng UI và tests.

## 5. Recommended investment order

## Phase 0 — Result surface foundation (MUST)

### M0.1 — Dock position model

**Feature:** Bottom/Right output dock mode.

**Scope:**

- Thêm layout mode `Bottom | Right` cho output dock.
- Giữ bottom là default để không phá flow hiện tại.
- Right panel có min/max width và resize handle ngang.
- Toolbar có chuyển mode và reset layout.
- Không detach window thật trong phase này.

**Definition of Done:**

- User chuyển Bottom ↔ Right mà không mất active document, active result, selection hoặc staged state.
- Right panel resize được và không làm editor nhỏ hơn min size.
- Output close/maximize/restore hoạt động ở cả hai mode.
- Layout calculation có unit tests cho bottom/right/closed/maximized.

**Effort:** M.

### M0.2 — Stable query result identity

**Feature:** Identity và routing cho result panel.

**Scope:**

- Phân biệt document id, execution/request id và result index/id.
- Output tab state được resolve theo document + result context.
- Selection, chart config, inspector và grid projection không dùng index đơn thuần làm identity dài hạn.
- Multi-result reducer giữ context khi đổi output layout.

**Definition of Done:**

- Hai query documents chạy cùng SQL không trộn result/selection/output tab.
- Multi-statement result vẫn chọn đúng statement sau khi đổi panel mode.
- Result panel mở lại đúng document/result sau khi chuyển workspace tab.
- Test có hai documents và ít nhất hai result sets mỗi document.

**Effort:** M/L.

### M0.3 — Right-side record/value inspector

**Feature:** Đưa Record inspector và Value inspector thành side panel thực sự.

**Scope:**

- Resizable right inspector trong result surface.
- Record mode và Value mode dùng chung inspector boundary.
- Giữ selected row/column khi sort/filter nếu original identity còn tồn tại.
- Giữ JSON/bytes modes và ChangeSet apply behavior.
- Read-only/write-block reason luôn hiển thị.

**Definition of Done:**

- Chọn row → mở Record inspector bên phải.
- Chọn field → mở Value inspector bên phải.
- JSON Raw/Pretty/Tree và Bytes Raw/Hex/Base64 vẫn hoạt động.
- Apply chỉ tạo/thay đổi ChangeSet; không tự execute.
- Filter/sort không khiến inspector hiển thị row khác.

**Effort:** M.

### M0.4 — Safety and capability boundary

**Feature:** Làm rõ trạng thái read-only/editable và provider capability trong result surface.

**Scope:**

- Query results luôn read-only.
- Table-data edit chỉ bật khi table mode, provider và write policy cho phép.
- Hiển thị reason cho cancel/parameters/explain/edit unsupported.
- Không dùng COPY/EXPLAIN/ILIKE/GLOB capability của provider này cho provider khác.

**Definition of Done:**

- PostgreSQL và SQLite có test capability matrix tối thiểu.
- Read-only result không có edit/delete/apply action.
- Unsupported action hiển thị reason, không emit unsupported command.
- Staged changes không bị mất khi đổi panel layout.

**Effort:** M.

## Phase 1 — Result usability (SHOULD)

### S1.1 — Result lifecycle

- Rename result.
- Pin result.
- Close current/close others.
- Hiển thị statement source/range.
- Reopen result từ execution history.

**Definition of Done:**

- Result action chỉ tác động result thuộc document/request hiện tại.
- Pin/rename/close được giữ trong phiên hiện tại.
- Close không xóa query history hoặc query document.

**Effort:** M.

### S1.2 — Typed filter/sort builder

- Phân biệt client-side và server-side.
- Filter theo type: text, number, date, boolean, NULL.
- AND/OR group cơ bản.
- Preview predicate cho server-side table data.
- Clear all filters/sorts.

**Definition of Done:**

- Query-result filter không sửa SQL gốc.
- Table-data server filter tạo predicate parameterized.
- Row identity sau projection vẫn đúng cho copy/edit/inspect.
- Empty/NULL/large numeric values có test.

**Effort:** L.

### S1.3 — Query Manager baseline

- Filter history theo connection, schema, status, time range.
- Hiển thị duration, affected rows, error code.
- Replay bằng original context hoặc current context.
- Clear entry/range.

**Definition of Done:**

- Replay original context không chạy nhầm connection.
- Failed execution giữ error summary.
- History filter không làm mất underlying entries.
- Có giới hạn memory/list rõ ràng.

**Effort:** M.

### S1.4 — Query object navigation

- Ctrl/Cmd-click hoặc action tương đương để mở definition.
- `F4`/command để Go to definition.
- Outline statement/CTE/JOIN/subquery.
- Context menu token: definition, data, copy qualified name.

**Definition of Done:**

- Token không resolve được thì không mở object sai.
- Connection/schema context của object được giữ.
- Navigation từ SQL không phá cursor/selection của document.
- PostgreSQL/SQLite object resolution được test riêng.

**Effort:** L.

## Phase 2 — Compare and advanced data (COULD)

### C2.1 — Compare result sets

- Compare current result với result khác hoặc table data.
- Chọn key columns.
- Diff added/removed/changed.
- Export diff.
- Generate reconciliation script, không auto-apply.

**Effort:** L.

### C2.2 — Result display modes

- Table.
- Transpose.
- Tree cho JSON/nested values.
- Text/raw mode.

**Effort:** M/L.

### C2.3 — Background export

- Streaming export.
- Progress/cancel.
- Scope selected/visible/all rows.
- XLSX/XML/HTML/TXT/DBUnit khi có consumer rõ ràng.

**Effort:** L.

### C2.4 — Explain plan comparison

- Save plan.
- Compare plan before/after.
- Persist plan in history.
- Provider-specific plan adapters.

**Effort:** L.

## 6. NOT NOW

- Detachable native window hoặc multi-window result workspace — chờ xác nhận product layout requirement.
- Plugin/custom extractor framework — chưa có consumer thứ hai; dùng enum format nội bộ trước.
- Full DataGrip-style Tree/Text/Transpose suite — chỉ làm sau khi right inspector và result identity ổn định.
- Query variables/delimiter system hoàn chỉnh — không cần cho right-panel vertical slice.
- Full schema/object compare từ Query Editor — thuộc compare feature riêng, không trộn với result compare.
- PostgreSQL-specific advanced plan visualizer — chưa mở rộng trước khi provider matrix rõ ràng.

## 7. Alternatives

### A — Chỉ thêm right dock

- **Ưu:** nhanh, ít code.
- **Nhược:** không giải quyết result identity, inspector state, compare/detach.
- **Effort:** S/M.
- **Verdict:** chỉ phù hợp prototype; không đủ làm baseline production.

### B — Result surface foundation trước (khuyến nghị)

- **Ưu:** giải quyết layout, identity, inspector và safety theo thứ tự; tái sử dụng cho compare/chart/explain.
- **Nhược:** cần sửa state/layout trước khi có nhiều feature visible.
- **Effort:** M/L.
- **Verdict:** phù hợp nhất với architecture hiện tại và mục tiêu right-side data.

### C — Full database IDE result platform

- **Ưu:** bao phủ nhiều capability giống DBeaver/DataGrip.
- **Nhược:** scope quá lớn, khó runtime verify, dễ tạo abstraction trước consumer.
- **Effort:** L/XL.
- **Verdict:** over-engineering ở phase hiện tại.

## 8. Decision criteria

- **Correctness — weight 5:** không trộn result/document và không edit nhầm data.
- **Reversibility — weight 4:** giữ bottom mode và có thể rollback right mode.
- **Time-to-value — weight 4:** user thấy right-side data sớm.
- **Reuse — weight 4:** nền tảng dùng được cho inspector/compare/chart/explain.
- **Provider safety — weight 5:** PostgreSQL/SQLite phải được account độc lập.
- **Scope control — weight 4:** không rewrite grid/editor không cần thiết.

## 9. Recommendation

**Chọn Alternative B: Result surface foundation trước.**

Thứ tự đầu tư:

```text
M0.1 Dock position
  → M0.2 Result identity
  → M0.3 Right inspector
  → M0.4 Safety/capability boundary
  → S1.1 Result lifecycle
  → S1.2 Filter/sort semantics
  → S1.3 Query Manager
  → S1.4 SQL object navigation
  → C2 Compare/advanced modes
```

**Confidence:** high cho M0.1/M0.2/M0.4 vì source hiện tại xác nhận rõ bottom dock, active-result state và capability boundaries. Medium cho M0.3 vì layout “right-side inspector” là product direction được suy ra từ yêu cầu hiện tại, chưa có screenshot/spec chi tiết.

**Điều gì có thể làm recommendation thay đổi:**

- Product xác nhận không cần right-side layout → M0.1 có thể hạ thành SHOULD.
- Compare đầu tiên là schema compare, không phải result compare → C2.1 chuyển sang feature folder compare/schema.
- Runtime profiling chứng minh grid projection là bottleneck trước layout → ưu tiên performance work trước S1.2.

**Sacrifice:** Chưa làm full export formats, detach window, advanced result modes và schema compare ngay; đổi lại có right-side surface ổn định và state safety trước.

## 10. Verification baseline

### Unit

- Bottom/right/closed/maximized layout calculations.
- Result identity resolution.
- Filter/sort projection giữ original row identity.
- Client/server filter classification.
- Capability gating.
- Export/JSON/bytes precision behavior.

### Integration

- One document with multiple result sets.
- Two documents with simultaneous/serial executions.
- Query result read-only versus table-data editable.
- Staged cell/row changes while switching output layout.
- PostgreSQL and SQLite query context separately.

### Native UI runtime

Verify at 1280×800, 1440×900 và 1920×1080:

- Bottom → Right → Bottom.
- Resize right panel.
- Maximize/restore/close output.
- Select row → Record inspector.
- Select JSON/bytes value → Value inspector.
- Filter/sort → inspector identity remains correct.
- Edit/apply/discard in table-data mode.
- Query result remains read-only.
- Loading/error/empty/result states.

### Safety

- Explain Analyze always requires explicit confirmation.
- Unsupported cancel/edit/parameters shows reason.
- No result action may mutate a different document/request.
- Export overwrite remains confirmation-driven.

## 11. Stop conditions

Dừng hoặc downscope nếu:

- Right layout làm phá vỡ result/document identity không giải quyết được trong scope M0.
- Cần rewrite toàn bộ `result_grid` để đạt right panel; khi đó tách performance/layout spike riêng.
- Không thể xác định writable table-data context đáng tin cậy cho apply/discard.
- Compare yêu cầu schema/object semantics vượt ngoài result surface.
- Runtime verification không thể tạo dữ liệu đủ đại diện cho cả PostgreSQL và SQLite.

## 12. Bias audit

- **Anchoring:** Không mặc định full DataGrip clone; đã so sánh phương án chỉ right dock và full platform.
- **Shiny-object:** Không ưu tiên Tree/Text/Transpose trước layout/state foundation.
- **Sunk cost:** Reuse result grid hiện tại nhưng không bảo vệ mọi behavior nếu identity model cần thay đổi.
- **Premature optimization:** Chưa thêm plugin extractor hoặc multi-window trước khi có consumer.
- **Confirmation:** Benchmark dùng nhiều nguồn DBeaver/DataGrip, nhưng recommendation vẫn dựa trên DB Pro source chứ không sao chép toàn bộ feature list.

## 13. Evidence boundary

Hiện trạng DB Pro dựa trên source commit `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. Recommendation là design analysis, chưa phải runtime evidence, chưa chứng minh provider behavior và chưa thay thế PLAN/CHECKLIST/VERIFICATION của implementation feature.
