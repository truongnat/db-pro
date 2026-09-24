# Query Editor và Data Panel — Feature Menu Baseline

**Mục đích:** liệt kê feature theo đúng thứ tự menu/toolbar người dùng nhìn thấy, sau đó xác định feature kế tiếp nên đầu tư. Đây là product feature baseline, không phải technical architecture plan.

**Source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`

## 1. Query Editor — menu `Query actions`

Menu hiện tại được render trong `query_actions_surface_view.rs` và chia thành nhóm Run, Editor, AI prediction và Query library.

### 1.1 Nhóm Run

| Thứ tự | Menu hiện tại | Action | Tính năng hiện có | Feature kế tiếp nên đầu tư |
|---:|---|---|---|---|
| 1 | Run query / Run selection | `Run` | Chạy toàn query hoặc phần SQL đang select. | **Run current statement** khi cursor nằm trong statement, không cần select thủ công. |
| 2 | Format SQL | `Format` | Format SQL theo dialect/capability hiện tại. | **Format options**: indent, keyword case, comma style, preview trước/sau. |
| 3 | Explain query | `Explain` | Chạy EXPLAIN và hiển thị plan trong tab Explain. | **Explain selected statement** và lưu/so sánh plan. |
| 4 | Ask Agent | `AskAgent` | Gửi yêu cầu giải thích current/selected SQL cho Agent. | **Ask Agent with result/error context** và action sửa query có preview. |
| 5 | Save query | `Save` | Lưu query hiện tại. | **Save status/version indicator** và autosave draft rõ ràng. |
| 6 | Save query as… | `SaveAs` | Mở dialog đặt tên query. | **Save to folder/project/location** và overwrite/duplicate policy rõ ràng. |
| 7 | Visual query builder | `ToggleVisualBuilder` | Mở/đóng visual query builder. | **Builder → SQL preview → apply back to editor**, có cảnh báo khi SQL editor đã bị sửa thủ công. |

### 1.2 Nhóm Editor

| Thứ tự | Menu hiện tại | Action | Tính năng hiện có | Feature kế tiếp nên đầu tư |
|---:|---|---|---|---|
| 1 | Find in SQL | `ToggleSearch` | Tìm, đếm match, next/previous, chọn match trong editor. | **Replace / Replace all** với confirmation và undo rõ ràng. |
| 2 | Show transaction controls | `ToggleTransaction` | Hiện transaction bar, commit/rollback, auto-commit/manual. | **Transaction timeline/status**: statement nào pending, affected rows, rollback boundary. |
| 3 | Decrease font size | `DecreaseFont` | Giảm font tới 10 px. | **Editor appearance settings**: font family, line height, minimap/wrap. |
| 4 | Increase font size | `IncreaseFont` | Tăng font tới 24 px. | Cùng nhóm với **editor appearance settings**. |

### 1.3 Nhóm AI prediction

| Thứ tự | Menu hiện tại | Action | Tính năng hiện có | Feature kế tiếp nên đầu tư |
|---:|---|---|---|---|
| 1 | Generate SQL Prediction | `GeneratePrediction` | Yêu cầu AI đề xuất SQL tại cursor. | **Accept / reject / next suggestion** và undo prediction rõ ràng. |
| 2 | Off | `SetPredictionMode(Off)` | Tắt prediction. | **Per-document mode** để query khác nhau có setting khác nhau. |
| 3 | Subtle | `SetPredictionMode(Subtle)` | Prediction nhẹ, có disclosure egress. | **Prediction confidence/source context** trước khi accept. |
| 4 | Eager | `SetPredictionMode(Eager)` | Prediction chủ động hơn. | **Budget/latency control** và cancel/retry feedback. |

### 1.4 Nhóm Query library

| Thứ tự | Menu hiện tại | Action | Tính năng hiện có | Feature kế tiếp nên đầu tư |
|---:|---|---|---|---|
| 1 | SQL snippets | `ToggleSnippets` | Mở danh sách built-in snippets và chèn tại cursor. | **Search/custom snippets** với shortcut và connection/provider scope. |
| 2 | folder (optional) | Library folder input | Gán folder cho query operation. | **Folder picker/tree** thay vì nhập text tự do. |
| 3 | New folder | `CreateFolder` | Tạo query folder cho active connection. | **Rename/move/delete folder** và duplicate query. |

## 2. Query Editor — context menu/chrome phía trên editor

Đây là các action luôn xuất hiện ngoài menu More.

| Thứ tự | UI | Action | Hiện có | Feature kế tiếp |
|---:|---|---|---|---|
| 1 | File breadcrumb | — | Hiện path rút gọn của query file. | **Click breadcrumb để mở folder/file location**. |
| 2 | Connection/schema chip | `SelectConnection`, `SelectSchema` | Đổi connection/schema theo document. | **Context history**: connection/schema gần đây và cảnh báo unsaved context. |
| 3 | Run/Stop | `Run`, `Cancel` | Run query, stop khi provider hỗ trợ, báo lý do nếu không hỗ trợ. | **Run current statement** và **execution progress/affected rows**. |
| 4 | Explain | `Explain` | Mở query plan. | **Explain current statement/selection**. |
| 5 | Format | `Format` | Format active document. | **Format preview/settings**. |
| 6 | More | — | Mở Query actions menu. | **Command search trong menu** khi menu có nhiều action hơn. |

## 3. Query Editor — feature tương tác trong text editor

| Thứ tự | Feature | Hiện có | Feature kế tiếp nên đầu tư |
|---:|---|---|---|
| 1 | Syntax highlighting | Có dialect-aware SQL highlighting. | Theme/token customization và highlight provider-specific syntax rõ hơn. |
| 2 | Cursor/selection | Có cursor, selection, current line/column. | Multi-cursor và column selection nếu có nhu cầu thực tế. |
| 3 | Completion | Keyword, table, view, column, function, schema, CTE, snippet. | **Object navigation**: Ctrl/Cmd-click, F4, Go to definition. |
| 4 | Signature help | Hiện function signature theo schema symbol index. | Overload selection và parameter documentation đầy đủ hơn. |
| 5 | Hover | Rich hover metadata cho token. | Hover action: open definition, open data, copy qualified name. |
| 6 | Diagnostics | Parser, delimiter, database, lint, capability mismatch. | Quick fix menu và fix-all có preview. |
| 7 | Search | Find/next/previous/selection. | Replace/replace all. |
| 8 | AI prediction | Off/Subtle/Eager, request/cancel. | Accept/reject/next suggestion và per-document settings. |
| 9 | SQL snippets | Built-in insert. | Search/custom/provider-scoped snippets. |
| 10 | Bind parameters | Named/numbered/positional, in-memory values, secret flag. | Typed values, default values, parameter validation trước Run. |
| 11 | Transaction | Auto-commit/manual, commit/rollback, pending count. | Transaction timeline và statement boundary. |
| 12 | Visual builder | Có toggle surface. | Two-way sync và conflict resolution giữa builder/editor. |

## 4. Output dock — menu/tab thứ tự hiện tại

Output tab strip được render trong `query_output_tabs_view.rs`.

| Thứ tự | Tab | Action/state | Tính năng hiện có | Feature kế tiếp nên đầu tư |
|---:|---|---|---|---|
| 1 | Results | `OutputTab::Results` | Result sets, row count, duration, export, result selector, grid. | **Right-side mode, result rename/pin/detach/close**. |
| 2 | Chart | `OutputTab::Chart` | Bar/Line/Area/Scatter/Pie, X/Y/Agg/Series. | **Save chart config, export chart, multi-series support**. |
| 3 | Messages | `OutputTab::Messages` | Notices/execution details, latest 20 messages. | **Filter by severity/source/statement** và copy message details. |
| 4 | Explain | `OutputTab::Explain` | Explain, Explain Analyze confirmation, raw JSON, plan tree, findings, copy plan. | **Explain selected statement** và plan compare/history. |
| 5 | History | `OutputTab::History` | Search recent executions, status, duration, SQL preview, replay/open. | **Query Manager filters** theo connection/schema/time/status/affected rows. |

## 5. Results tab — toolbar theo thứ tự người dùng

Toolbar được render trong `result_grid_toolbar_view.rs`.

| Thứ tự | Control | Action | Hiện có | Feature kế tiếp |
|---:|---|---|---|---|
| 1 | Filter visible rows | state `grid_filter` | Lọc row đang hiển thị và báo matching count. | **Typed filter builder** với NULL/date/number/AND/OR. |
| 2 | Clear filter | clear `grid_filter` | Xóa filter nhanh. | Clear all filter/sort/layout trong một menu reset. |
| 3 | Row count badge | matching rows | Hiện số row sau filter. | **Total row count async** và phân biệt loaded/matched/total. |
| 4 | Copy Cell | `CopySelectedCell` | Copy cell được chọn. | Copy raw/formatted value tùy data type. |
| 5 | Copy Row | `CopySelectedRow` | Copy row dạng TSV. | Chọn delimiter/headers và copy selected rows. |
| 6 | CSV | `CopyVisibleCsv` | Copy visible rows CSV. | Export file streaming, không chỉ clipboard. |
| 7 | JSON | `CopyVisibleJson` | Copy visible rows JSON array. | JSON Lines, preserve numeric precision và NULL policy. |
| 8 | Markdown | `CopyVisibleMarkdown` | Copy visible rows Markdown. | Preview Markdown trước copy/export. |
| 9 | INSERT | `CopyVisibleInsert` | Copy visible rows thành INSERT SQL. | Chọn schema/table/identifier quoting và batch size. |
| 10 | Record | `record_inspector_open` | Toggle full-record inspector. | **Right-side inspector** với next/previous record. |
| 11 | Inspect | `InspectSelectedCell` | Mở advanced value inspector. | Gộp Record/Value thành một right-side data inspector. |

## 6. Results grid — header menu theo thứ tự

Header menu được render trong `result_grid_header_menu_view.rs`.

| Thứ tự | Menu | Action | Hiện có | Feature kế tiếp |
|---:|---|---|---|---|
| 1 | Sort Ascending | `Sort(Some(false))` | Sort A → Z/ascending. | Hiển thị client/server và generated order rõ ràng. |
| 2 | Sort Descending | `Sort(Some(true))` | Sort Z → A/descending. | Multi-column sort builder. |
| 3 | Clear Sort | `Sort(None)` | Xóa sort hiện tại. | Clear all sort với thứ tự ưu tiên hiển thị. |
| 4 | Add Filter | `AddFilter` | Thêm filter ở table-data mode. | Typed filter builder và preview predicate. |
| 5 | Copy Column Name | `CopyColumnName` | Copy tên column. | Copy qualified name/type/comment. |
| 6 | Copy Column Values | `CopyColumnValues` | Copy values của column. | Chọn visible/all/selected rows. |
| 7 | Move Column Left | `MoveLeft` | Đổi thứ tự column. | Drag reorder trực tiếp. |
| 8 | Move Column Right | `MoveRight` | Đổi thứ tự column. | Drag reorder trực tiếp. |
| 9 | Reset Column Order | `ResetOrder` | Reset order. | Named layout/profile. |
| 10 | Reset Column Widths | `ResetWidths` | Reset width. | Auto-size theo content/type/header. |
| 11 | Hide Column | `HideColumn` | Ẩn column. | Hide-by-default profile và quick show hidden count. |
| 12 | Show Columns | `ShowColumns` | Hiện hidden columns. | Chọn từng column để show/hide. |
| 13 | Reset Layout | `ResetLayout` | Reset order/width/hidden layout. | Save/persist named grid layouts. |
| 14 | Auto Size | `AutoSize` | Tự điều chỉnh kích thước column. | Async auto-size cho result lớn. |

## 7. Cell context menu — thứ tự feature dữ liệu

Cell menu được render trong `result_grid_cell_menu_view.rs`.

### 7.1 Nhóm copy

| Thứ tự | Menu | Action | Hiện có | Feature kế tiếp |
|---:|---|---|---|---|
| 1 | Copy Cell Value | `CopyCell` | Copy cell. | Copy raw/display/formatted mode. |
| 2 | Copy Row (TSV) | `CopyRow` | Copy một row TSV. | Copy row với metadata/headers tùy chọn. |
| 3 | Copy Selected Rows | `CopySelectedRows` | Copy các row selected. | Scope visible/filtered/all rõ ràng. |
| 4 | Copy Selected Rows with Headers | `CopySelectedRowsHeaders` | Copy selected rows kèm headers. | Chọn format từ một submenu thống nhất. |
| 5 | Copy Selected Rows as JSON | `CopySelectedRowsJson` | JSON array. | JSON Lines và precision policy. |
| 6 | Copy Selected Rows as Markdown | `CopySelectedRowsMarkdown` | Markdown table. | Preview/export file. |
| 7 | Copy Selected Rows as INSERT SQL | `CopySelectedRowsInsert` | INSERT SQL. | Chọn target table/schema và conflict policy. |
| 8 | Copy Row as JSON | `CopyJson` | JSON cho một row. | Include column types/PK metadata. |
| 9 | Copy Row as CSV | `CopyCsv` | CSV cho một row. | CSV options/NULL policy. |

### 7.2 Nhóm edit

Chỉ hiện khi result surface là editable và write policy cho phép.

| Thứ tự | Menu | Action | Hiện có | Feature kế tiếp |
|---:|---|---|---|---|
| 1 | Edit Cell | `EditCell` | Enter/F2 hoặc menu để edit. | Typed editor theo date/number/JSON/bytes. |
| 2 | Set to NULL | `SetNull` | Đặt cell thành NULL. | Explicit confirmation khi column NOT NULL. |
| 3 | Revert Cell | `RevertCell` | Revert staged cell. | Diff before/after inline. |
| 4 | Revert Row / Undo Delete | `RevertRow` | Revert row hoặc undo delete. | Chọn nhiều row để revert. |
| 5 | Duplicate Row | `DuplicateRow` | Duplicate row vào ChangeSet. | Preview generated INSERT và PK handling. |
| 6 | Delete Row | `DeleteRow` | Stage row deletion. | Affected-row preview và dependency warning. |

### 7.3 Nhóm filter/sort

| Thứ tự | Menu | Action | Hiện có | Feature kế tiếp |
|---:|---|---|---|---|
| 1 | Filter by this value | `FilterThisValue` | Tạo filter theo cell value. | Filter builder theo type và NULL semantics. |
| 2 | Sort Ascending | `SortAscending` | Sort theo column. | Multi-column sort state. |
| 3 | Sort Descending | `SortDescending` | Sort ngược theo column. | Server/client mode indicator. |

## 8. Record/Value inspector — thứ tự feature

### Record inspector

| Thứ tự | Feature | Hiện có | Feature kế tiếp |
|---:|---|---|---|
| 1 | Select row | Có selected row/cell. | Giữ row identity qua filter/sort. |
| 2 | Record preview | Hiện từng column và preview value. | Hiển thị PK/FK/type/write policy. |
| 3 | Inspect field | Mở value inspector cho field. | Previous/Next field. |
| 4 | Close | Đóng inspector. | Pin/unpin right panel. |

### Value inspector

| Thứ tự | Feature | Hiện có | Feature kế tiếp |
|---:|---|---|---|
| 1 | Raw | Text/raw value. | Copy display/raw distinction. |
| 2 | Pretty | Pretty JSON/value. | Format validation và diff. |
| 3 | Tree | JSON tree. | Expand/collapse/search path. |
| 4 | Hex | Bytes hex. | Offset/length selection. |
| 5 | Base64 | Bytes Base64. | Decode/encode explicit state. |
| 6 | Copy raw | Copy raw value. | Copy path/type/value bundle. |
| 7 | Export bytes | Export `.bin`. | Export selected representation. |
| 8 | Apply to ChangeSet | Apply edit. | Preview diff và validation trước apply. |
| 9 | Close | Close dialog/inspector. | Keep inspector open while navigating fields. |

## 9. Menu-based investment baseline

Đây là thứ tự đầu tư theo feature người dùng, không theo module code.

### Invest 1 — Results layout

Menu/action:

```text
Results
  → Bottom / Right
  → Resize
  → Maximize / Restore
  → Close / Reopen
```

Lý do: đây là feature trực tiếp đáp ứng yêu cầu data hiển thị bên phải và là container cho các feature sau.

### Invest 2 — Record/Value inspector bên phải

Menu/action:

```text
Record
  → Select record
  → Inspect field
  → Previous/Next field
  → Apply / Revert
```

Lý do: tận dụng primitive hiện có, tăng tốc inspect data mà không phải rời grid.

### Invest 3 — Result lifecycle

Menu/action:

```text
Result 1
  → Rename
  → Pin
  → Close
  → Detach
  → Compare
```

Lý do: cần trước khi compare hoặc chạy nhiều statement/result trong thực tế.

### Invest 4 — Filter/sort nâng cao

Menu/action:

```text
Filter
  → By value
  → By type
  → AND/OR
  → NULL/date/number operators
  → Client/server mode
  → Clear all
```

Lý do: feature kế tiếp tự nhiên sau khi result surface ổn định.

### Invest 5 — Query Editor navigation

Menu/action:

```text
SQL token
  → Go to definition
  → Open data
  → Copy qualified name
  → Find usages
  → Query outline
```

Lý do: completion/hover đã có; bước kế tiếp là biến metadata thành navigation action.

### Invest 6 — Query history manager

Menu/action:

```text
History
  → Search
  → Filter connection/schema/status/time
  → Replay original context
  → Replay current context
  → Pin/delete/clear
```

Lý do: history hiện có nhưng còn là pane đơn giản, chưa phải operational query manager.

### Invest 7 — Compare

Menu/action:

```text
Compare
  → Select source result
  → Select target result/object
  → Select key columns
  → Show added/removed/changed
  → Export diff
  → Generate script
```

Lý do: compare nên làm sau khi result identity, lifecycle và filter semantics ổn định.

## 10. Không đầu tư trước

- Full Visual Query Builder rewrite.
- Multi-window native detach.
- Hàng loạt export format mới.
- Full Tree/Text/Transpose suite.
- Explain plan comparison.
- Plugin extractor framework.
- Multi-cursor/editor polish không liên quan đến data panel.

## 11. Kết luận

Nếu đi theo đúng thứ tự menu/product feature, baseline nên là:

```text
Right Results
  → Record/Value Inspector
  → Result Lifecycle
  → Filter/Sort Builder
  → SQL Object Navigation
  → Query History Manager
  → Result/Data Compare
```

Đây là thứ tự user-facing hợp lý hơn việc bắt đầu bằng các task kỹ thuật như “result identity” hoặc “layout model”. Các task kỹ thuật đó vẫn cần làm bên dưới, nhưng không nên dùng chúng làm tên feature/roadmap chính.

## 12. Evidence boundary

Current feature list dựa trên source commit `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. Các “feature kế tiếp” là đề xuất product baseline, chưa phải runtime evidence hoặc implementation commitment.
