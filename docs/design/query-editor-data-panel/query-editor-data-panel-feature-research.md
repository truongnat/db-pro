# Query Editor và Data Result Panel — Feature Research

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/query-editor-data-panel/`  
**DB Pro source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`

## 1. Mục tiêu

Research này đối chiếu Query Editor và vùng data/result hiện tại với capability được mô tả trong tài liệu chính thức của DBeaver và JetBrains DataGrip. Mục tiêu là tìm phần đang thiếu và xác định hướng nâng cấp có giá trị nhất.

Trong source DB Pro, output hiện là **bottom output dock**. DBeaver/DataGrip cũng có các mô hình result panel/in-editor result khác nhau, vì vậy layout docking được coi là một feature riêng thay vì chỉ là vấn đề UI polish.

## 2. Nguồn chính thức

### DBeaver

- [SQL Editor](https://dbeaver.com/docs/dbeaver/SQL-Editor/)
- [SQL Execution](https://dbeaver.com/docs/dbeaver/SQL-Execution/)
- [Data Editor](https://dbeaver.com/docs/dbeaver/Data-Editor/)
- [Query Manager](https://dbeaver.com/docs/dbeaver/Query-Manager/)
- [Data Export](https://dbeaver.com/docs/dbeaver/Data-export/)
- [Database Navigator](https://dbeaver.com/docs/dbeaver/Database-Navigator/)

### JetBrains DataGrip

- [Data editor and viewer](https://www.jetbrains.com/help/datagrip/data-editor-and-viewer.html)
- [Database Explorer](https://www.jetbrains.com/help/datagrip/database-explorer.html)
- [Viewing reference information](https://www.jetbrains.com/help/datagrip/viewing-reference-information.html)
- [Schema comparison](https://www.jetbrains.com/help/datagrip/schema-comparison.html)

Một số trang DataGrip/pgAdmin không phản hồi ổn định trong lần fetch này; các đề xuất dưới đây chỉ dùng claim cụ thể từ những trang đã fetch được và không coi tên feature của sản phẩm khác là runtime evidence cho DB Pro.

## 3. Các capability benchmark nổi bật

### DBeaver SQL Editor

Tài liệu chính thức mô tả SQL Editor gồm:

- Script panel.
- Toolbar có thể tùy chỉnh.
- Result panel.
- Nhiều result trong một tab.
- Đổi active datasource/schema nhưng giữ nguyên SQL text.
- Toggle/maximize result panel.
- SQL outline.
- Error indication gắn tại vị trí query.
- SQL templates và SQL assist/auto-complete.
- Hyperlink từ identifier trong SQL tới object editor.
- Evaluate SQL expression.
- Select row count.
- Export trực tiếp từ query.
- Parameters, variables và delimiters.
- Mở object definition bằng phím tắt.

### DBeaver SQL Execution

Tài liệu mô tả thêm:

- Chạy một query, phần SQL được highlight, hoặc toàn bộ script.
- Một query có thể tạo nhiều result sets.
- Result tab có thể đổi tên, pin, kéo thả.
- Result tab có thể detach để đặt cạnh nhau.
- Result tab còn đồng bộ với SQL Editor khi editor gốc còn mở.
- Query history/manager ghi execution time, duration, affected rows và errors.

### DBeaver Data Editor

Tài liệu mô tả:

- Data Editor dùng cho table/view data và query result.
- Có top toolbar, left sidebar, right sidebar và bottom toolbar.
- Column context menu và cell context menu.
- Tính total row count.
- Table status indicators giải thích vì sao data không editable.
- Tắt metadata queries để giảm latency/cost ở database lớn hoặc tính phí theo query.

### DBeaver Query Manager

Tài liệu mô tả:

- Query log tổng hợp execution time, duration, affected rows, errors.
- Filter theo time range, connection, catalog, schema.
- Lưu history trong local SQL history database.
- Cấu hình cleanup old records.
- Clear query log.

### DBeaver Data Export

Tài liệu mô tả export từ table/data editor hoặc trực tiếp từ query. Các format được dẫn gồm CSV, DBUnit, JSON, Markdown, source code, SQL, TXT, XML, XLSX và HTML.

### DataGrip Data Editor

Tài liệu chính thức mô tả:

- Data editor cho database object data và query result sets.
- Table, Tree, Text và Transpose modes.
- View/edit value riêng trong value editor.
- Compare data của hai database objects.
- Sort bằng query server-side hoặc client-side.
- Filter conditions và search trong table.
- Data extractors cho file/clipboard, custom CSV/DSV extractor.
- Add/delete/clone rows.
- Navigation tới related rows/subsets/specified row.
- In-editor results pane trong query console.

## 4. Gap và feature cần nâng cấp

### R1 — Flexible result docking: bottom/right/in-editor

**Hiện tại:**

- DB Pro chỉ có output dock phía dưới Query Editor.
- Có resize dọc, maximize, restore, close.
- Không có right-side split cố định, detach, hoặc đặt editor/result cạnh nhau.

**Benchmark:**

- DBeaver cho phép show/hide result panel, đổi horizontal/vertical layout, maximize result panel và detach result tab.
- DataGrip hỗ trợ result trong Services hoặc in-editor results pane.

**Đề xuất:**

- Giữ bottom dock làm default.
- Thêm display mode: Bottom / Right / In-editor.
- Cho phép drag splitter theo orientation.
- Detach result thành surface riêng.
- Giữ result sống độc lập với scroll/editor viewport.
- Persist layout per workspace/document.

**Action đề xuất:**

```text
SetOutputDockPosition(Bottom | Right | InEditor)
ToggleOutputDock
MaximizeOutput
DetachResult
AttachResult
SplitEditorAndResult
ResetOutputLayout
```

**Ưu tiên:** P1. Đây là gap lớn nhất so với yêu cầu “data hiển thị ở phía bên phải”.

---

### R2 — Result tab lifecycle chưa đủ

**Hiện tại:**

- Có nhiều result sets và chọn `Result 1`, `Result 2`, ...
- Có active result index per document.
- Chưa thấy result tab naming, pin, detach, close từng result, hoặc close-all-except-current.

**Benchmark:**

DBeaver hỗ trợ đổi tên, pin, kéo thả, detach và quản lý nhiều result tabs.

**Đề xuất:**

- Tên result theo statement/table hoặc alias.
- Rename result.
- Pin result.
- Close result hiện tại.
- Close others.
- Detach result.
- Reopen result từ execution history trong cùng document.
- Hiển thị statement range/source SQL cho từng result.

**Action đề xuất:**

```text
RenameResult
PinResult
CloseResult
CloseOtherResults
DetachResult
SelectResultSource
```

**Ưu tiên:** P1.

---

### R3 — SQL object navigation trong editor còn thiếu

**Hiện tại:**

- Có completion, hover và signature help.
- Chưa thấy hyperlink click trên table/view identifier.
- Chưa thấy `F4`/Go to definition trực tiếp từ token.
- Chưa có outline/structure view cho query.

**Benchmark:**

DBeaver hỗ trợ Ctrl-hover hyperlink tới table/view editor, mở definition bằng shortcut và SQL Outline.

**Đề xuất:**

- Ctrl/Cmd-click table/view/function để mở object.
- `F4` hoặc shortcut tương đương để mở definition.
- Context menu token:
  - Go to definition;
  - Open data;
  - Copy qualified name;
  - Find usages.
- Outline panel hiển thị CTE, SELECT, JOIN, subquery, parameters và statement boundaries.
- Highlight toàn bộ reference khi hover/click.

**Action đề xuất:**

```text
GoToSqlDefinition
OpenSqlObjectData
CopySqlObjectName
FindSqlObjectUsages
ToggleQueryOutline
NavigateStatement
```

**Ưu tiên:** P1.

---

### R4 — Execution scope và execution utilities chưa đủ

**Hiện tại:**

- Có run current statement/selection và run all.
- Có cancel theo capability.
- Có query parameters cơ bản.
- Có transaction control.

**Benchmark:**

DBeaver có explicit execution cho query/highlighted script/whole script, evaluate SQL expression, select row count, direct export from query, parameters/variables/delimiters.

**Đề xuất:**

- Action chạy statement dưới cursor rõ ràng, không chỉ selection/all.
- `Select row count` không cần materialize toàn bộ result.
- `Evaluate expression` cho expression đang chọn.
- Query variables riêng với bind parameters.
- Script delimiter configuration.
- Export query trực tiếp, kể cả query dài/chạy lâu.
- Execution target hiển thị rõ connection/schema trước khi run.
- Per-statement execution status trong script.

**Action đề xuất:**

```text
RunCurrentStatement
RunSelection
RunAllStatements
CancelExecution
SelectRowCount
EvaluateSqlExpression
SetQueryVariable
ConfigureDelimiter
ExportQueryDirectly
```

**Ưu tiên:** P1 cho row count/direct export; P2 cho variables/delimiters.

---

### R5 — Result data view modes còn hạn chế

**Hiện tại:**

- Result grid dạng table.
- Có Chart tab.
- Có Record inspector.
- Có advanced value inspector cho JSON/bytes.
- Chưa có Tree/Text/Transpose mode thống nhất.

**Benchmark:**

DataGrip hỗ trợ Table, Tree, Text và Transpose modes; có value editor riêng cho cell.

**Đề xuất:**

- Table mode: grid hiện tại.
- Tree mode: object/JSON/nested value explorer.
- Text mode: một record hoặc raw result text.
- Transpose mode: column/value vertical layout cho bảng rộng.
- Value editor mở inline hoặc right-side.
- Persist mode per result/document.
- Không biến JSON pretty view thành một feature chỉ có trong dialog; cho phép chuyển mode ngay trong panel.

**Action đề xuất:**

```text
SetResultViewMode(Table | Tree | Text | Transpose)
OpenValueEditor
CloseValueEditor
TransposeResult
ExpandNestedValue
CollapseNestedValue
```

**Ưu tiên:** P1 cho Transpose và Value Editor; P2 cho Tree/Text.

---

### R6 — Filtering/sorting cần phân biệt client-side và server-side

**Hiện tại:**

- Có filter visible rows trong result grid.
- Có sort/filter action ở header/cell.
- Table data mode có data query sort/filter state.
- Query result mode chủ yếu thao tác trên projection trong memory.

**Benchmark:**

DataGrip cho phép sort bằng query gửi xuống database hoặc sort client-side; filter hỗ trợ điều kiện và search.

**Đề xuất:**

- Hiển thị rõ filter đang chạy ở client hay server.
- Query result: client filter/sort mặc định, không sửa SQL bất ngờ.
- Table data: server-side filter/sort mặc định vì dataset có thể lớn.
- Filter builder typed theo column/data type.
- AND/OR groups.
- NULL/empty/string/number/date operators.
- Preview generated WHERE/ORDER BY.
- Clear all filters/sorts.
- Row count before/after filter.

**Action đề xuất:**

```text
OpenFilterBuilder
ApplyClientFilter
ApplyServerFilter
ClearAllFilters
ClearAllSorts
PreviewGeneratedPredicate
CountFilteredRows
```

**Ưu tiên:** P1.

**Safety:** Generated predicates phải parameterized; không nối user input trực tiếp vào SQL.

---

### R7 — Data editor mode và row operations chưa đầy đủ

**Hiện tại:**

- Có edit cell, set NULL, duplicate row, delete row.
- Có revert cell/row, ChangeSet, apply/discard.
- Có read-only column reason.
- Có table-data editable mode tách khỏi query-result read-only mode.

**Benchmark:**

DataGrip hỗ trợ add/delete/clone row, related-row navigation, subset navigation và go-to-row. DBeaver có status indicators, sidebars và total row count.

**Đề xuất:**

- Add new row rõ ràng trong toolbar.
- Go to row number.
- Navigate related row từ FK.
- Show row source/primary key identity.
- Show affected-row estimate trước Apply.
- Staged change summary theo cell/row.
- Apply selected changes hoặc apply all.
- Conflict/stale-row detection khi update không còn match.
- Total row count async, cancelable.
- Metadata loading state theo table/column.

**Action đề xuất:**

```text
AddRow
GoToRow
OpenRelatedRow
ShowChangeSet
ApplySelectedChanges
ApplyAllChanges
DiscardSelectedChanges
CalculateTotalRowCount
CancelTotalRowCount
```

**Ưu tiên:** P1 cho AddRow/ChangeSet summary; P2 cho related row/count nâng cao.

---

### R8 — Record/value inspector nên trở thành panel bên phải thực sự

**Hiện tại:**

- `Record` inspector đang được draw trong flow dọc của result grid.
- Advanced value inspector mở dạng dialog.
- JSON có Raw/Pretty/Tree.
- Bytes có Raw/Hex/Base64.
- Có Copy raw, Export bytes, Apply to ChangeSet.

**Gap so với yêu cầu right-side data display:**

- Chưa có right-side inspector cố định.
- Chưa có điều hướng Previous/Next row trong inspector.
- Chưa có lock/unlock panel.
- Chưa có pin selected record khi scroll grid.
- Chưa có breadcrumb `row → column → value`.

**Đề xuất:**

- Right inspector width resizable.
- `Record` mode và `Value` mode trong cùng side panel.
- Previous/Next record.
- Previous/Next field.
- Keep selected record while filtering/sorting khi identity còn tồn tại.
- Show PK/FK metadata and write policy.
- Open related record from FK.
- Dock/undock inspector.

**Action đề xuất:**

```text
OpenRightRecordInspector
OpenRightValueInspector
ResizeInspector
PinInspector
NextRecord
PreviousRecord
NextField
PreviousField
OpenRelatedRecord
```

**Ưu tiên:** P1 nếu product direction yêu cầu data ở phía bên phải.

---

### R9 — Export hiện đủ cơ bản nhưng thiếu export workflow nâng cao

**Hiện tại:**

- CSV, TSV, JSON, Markdown, INSERT, COPY.
- Copy trực tiếp một số format.
- Export dialog có output path.
- Có overwrite confirmation.
- Có byte export.

**Benchmark:**

DBeaver mô tả thêm DBUnit, source code, TXT, XML, XLSX, HTML và export trực tiếp từ query. DataGrip có data extractors, custom CSV/DSV extractor và clipboard/file targets.

**Đề xuất:**

- XLSX, HTML, XML, TXT, DBUnit.
- Chọn visible rows/all rows/selected rows.
- Chọn current result/all result sets.
- Column mapping/rename trước export.
- Encoding, delimiter, newline, NULL representation.
- Include/exclude headers.
- Streaming export cho result lớn.
- Background export progress/cancel.
- Custom extractor/plugin contract nếu cần về sau.

**Action đề xuất:**

```text
OpenExportWizard
SelectExportScope
SelectExportFormat
ConfigureExtractor
StartBackgroundExport
CancelExport
```

**Ưu tiên:** P1 cho scope/streaming; P2 cho format mở rộng.

---

### R10 — Query history cần nâng từ pane thành query manager

**Hiện tại:**

- History pane hiển thị recent executions.
- Search theo SQL text.
- Hiển thị status, timing và preview.
- Có one-click replay/open.
- Giới hạn list hiển thị 50 entry trong pane.

**Benchmark:**

DBeaver Query Manager lưu query, execution time, duration, affected rows và errors; filter theo time/connection/catalog/schema; có local history database và cleanup.

**Đề xuất:**

- Filter theo:
  - time range;
  - connection;
  - schema/database;
  - status;
  - duration;
  - affected rows.
- Hiển thị affected rows và error code.
- Pin/favorite history entry.
- Copy SQL/result metadata.
- Re-run with current context hoặc original context.
- Delete one entry/delete range/clear all.
- Retention policy và cleanup old records.
- Persistent history database/file với giới hạn dung lượng.
- Distinguish query text, statement, script execution và result set.

**Action đề xuất:**

```text
FilterQueryHistory
ReplayWithOriginalContext
ReplayWithCurrentContext
PinHistoryEntry
DeleteHistoryEntry
ClearHistory
ConfigureHistoryRetention
```

**Ưu tiên:** P1 cho filters/affected rows; P2 cho retention/persistence nâng cao.

---

### R11 — Result tab comparison và data compare còn thiếu

**Hiện tại:**

- Có nhiều result set nhưng chỉ chọn từng result.
- Có chart và history.
- Chưa có compare hai result set hoặc compare hai table data ngay trong data surface.

**Benchmark:**

DataGrip Data Editor có compare data của hai database objects và cho phép cấu hình tiêu chí khác nhau.

**Đề xuất:**

- Compare current result với result khác.
- Compare result với table/object.
- Chọn key columns.
- Diff added/removed/changed rows.
- Ignore extra columns tùy chọn.
- Export diff report.
- Generate reconciliation SQL nhưng không auto-apply.

**Action đề xuất:**

```text
CompareResults
CompareResultWithObject
SelectCompareKeys
ConfigureIgnoredColumns
ExportDataDiff
GenerateReconciliationScript
```

**Ưu tiên:** P1 nếu compare là product direction chính.

---

### R12 — Explain/plan surface nên có compare và history context

**Hiện tại:**

- Explain và Explain Analyze.
- Explicit confirmation cho Explain Analyze vì có thể execute write.
- Raw JSON toggle.
- Parsed plan tree.
- Findings/advisor messages.
- Copy plan.

**Nên nâng cấp:**

- So sánh plan trước/sau khi sửa query.
- Lưu plan vào history entry.
- So sánh runtime metrics giữa executions.
- Hiển thị statement source/range liên quan đến plan.
- Provider-specific plan adapters thay vì chỉ PostgreSQL-shaped parser.
- Export plan JSON/Markdown.
- Explain selected statement trong multi-statement script.

**Action đề xuất:**

```text
ExplainSelectedStatement
SaveExplainPlan
CompareExplainPlans
ExportExplainPlan
OpenPlanSourceStatement
```

**Ưu tiên:** P2 sau khi result layout và execution state ổn định.

## 5. Mức độ ưu tiên đề xuất

### P0 — Safety/Correctness gate

- Không để right-side editor chạy query trên sai connection/schema.
- Không mất result khi đổi layout hoặc detach.
- Không apply staged data edits nhầm result/document.
- Export lớn phải có cancellation hoặc giới hạn rõ ràng.
- Explain Analyze confirmation phải giữ nguyên.

### P1 — Core product upgrades

1. Right-side/in-editor result layout.
2. Result tab lifecycle: rename/pin/detach/close.
3. SQL object navigation: hyperlink/F4/outline.
4. Typed server/client filter builder.
5. Right-side record/value inspector.
6. Query manager filters và affected rows.
7. Add row/ChangeSet summary.
8. Compare results/source-target flow.

### P2 — Depth and polish

1. Tree/Text/Transpose modes.
2. Extended export formats.
3. Query variables/delimiters.
4. Explain plan comparison.
5. Retention/custom extractors.
6. Related-row navigation nâng cao.

## 6. Scope nên triển khai đầu tiên

Nếu mục tiêu trước mắt là cải thiện “Query Editor + data phía bên phải”, scope coherent nhất là:

```text
Query Editor
  ├── giữ editor ở trái/trung tâm
  ├── chạy query và giữ document context
  └── mở Results ở right-side dock

Right Result Panel
  ├── Results / Chart / Messages / Explain / History
  ├── virtualized grid
  ├── filter/sort/column layout
  ├── Record inspector
  └── Value inspector
```

### Slice 1 — Right-side result layout

Actions tối thiểu:

```text
SetOutputDockPosition(Right)
ResizeOutputDock
ToggleOutputDock
MaximizeOutput
ResetOutputLayout
```

Acceptance criteria:

- Editor vẫn giữ focus/cursor khi result dock mở.
- Result panel resize không làm mất selected document/result.
- Right panel có min/max width.
- Có keyboard/toolbar để chuyển Bottom ↔ Right.
- Layout được restore khi mở lại document.
- Output dock không che mất error/diagnostic state.

### Slice 2 — Record/value inspector

Actions tối thiểu:

```text
OpenRecordInspector
OpenValueInspector
CloseInspector
InspectNextField
InspectPreviousField
ApplyValueToChangeSet
```

Acceptance criteria:

- Inspector hiển thị đúng row identity sau filter/sort.
- JSON/bytes không bị coercion mất dữ liệu.
- Read-only/write-block reason luôn hiển thị.
- Apply chỉ tác động ChangeSet, không tự execute.

### Slice 3 — Result lifecycle and compare

Actions tối thiểu:

```text
RenameResult
PinResult
DetachResult
CompareResults
ExportResultDiff
```

Acceptance criteria:

- Result gắn đúng document/request.
- Switching query document không trộn result state.
- Compare không tự apply mutation.
- Source/target/result context luôn hiển thị rõ.

## 7. Provider và capability implications

Mọi nâng cấp database-facing phải account độc lập cho PostgreSQL và SQLite.

| Capability | PostgreSQL | SQLite | Ghi chú |
|---|---|---|---|
| Query cancel | Capability-dependent | Capability-dependent | UI phải show reason khi unsupported. |
| Bind parameters | Có capability model | Có capability model | Không giả định cùng syntax. |
| Server-side filtering | Có | Có | Predicate/quoting phải provider-aware. |
| Table data edit | Có thể hỗ trợ | Có thể hỗ trợ | Phải tôn trọng PK/write policy/ChangeSet. |
| Explain plan | PostgreSQL plan parser hiện có | Cần adapter riêng | Không suy diễn plan PostgreSQL cho SQLite. |
| COPY export | Có ý nghĩa PostgreSQL | Không emit như PostgreSQL mặc định | Capability-gated. |
| ILIKE/GLOB lint | Có provider distinction | Có provider distinction | Diagnostics hiện đã phản ánh một phần. |
| Result compare | Có | Có thể hỗ trợ | Key/diff semantics cần explicit. |

## 8. Không nên làm trong phase đầu

- Không rewrite toàn bộ editor renderer chỉ để đổi bottom thành right panel.
- Không thêm mọi format export trước khi có background/cancelable export.
- Không tự động biến mọi result filter thành SQL mutation.
- Không cho query result read-only surface âm thầm ghi database.
- Không dùng result index đơn thuần làm identity lâu dài khi có detach/compare; cần document/request/result identity.
- Không coi DataGrip/DBeaver behavior là bằng chứng provider runtime của DB Pro.

## 9. Kết luận

Query Editor của DB Pro hiện đã có nền tảng mạnh: document-scoped context, completion, hover/signature help, diagnostics/lint, prediction, transactions, parameters, snippets, query history và nhiều output modes. Result grid cũng đã có filtering, sorting, column layout, virtualized rendering, editing, ChangeSet, export, chart, record inspector và value inspector.

Khoảng cách lớn nhất so với các database IDE trưởng thành là:

1. output chưa có right-side/in-editor docking;
2. result tab chưa có lifecycle detach/pin/rename;
3. thiếu object navigation/outline trong SQL;
4. filtering/sorting chưa thể hiện rõ client/server semantics;
5. record/value inspector chưa trở thành right-side workspace;
6. query history chưa thành query manager có filter/retention đầy đủ;
7. thiếu data/result compare;
8. execution utilities như row count, expression evaluation, variables và direct export chưa đầy đủ.

Nếu làm theo compare/product direction, nên bắt đầu từ right-side result layout, sau đó là result identity/lifecycle, rồi mới triển khai compare và advanced data modes.

## 10. Evidence boundary

Hiện trạng DB Pro dựa trên source commit `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. Capability của DBeaver/DataGrip dựa trên các official docs link ở mục 2. Chưa có runtime UI evidence hoặc PostgreSQL/SQLite runtime evidence cho các đề xuất trong report này.
