# Problems — Feature Research and Placement Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/problems/`  
**Source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`

## 1. Mục tiêu và kết luận

Đánh giá placement của **Problems** trong native DB Pro theo ý kiến đã nêu: Problems không nên nằm trong sidebar. Kết luận source và product goal ủng hộ hướng đó: đây là danh sách diagnostics có phạm vi document/workspace và hành động chính là quay lại đúng SQL source, không phải một miền điều hướng bền vững như Explorer, Queries hay Data.

**Khuyến nghị:** bỏ Problems khỏi activity rail/sidebar; giữ một danh sách Problems có thể gọi theo nhu cầu trong lower tool panel của workspace, kèm status/count affordance và command palette action. Giữ inline diagnostics trong editor. Không biến danh sách workspace-wide thành một `OutputTab` per-query nếu chưa tách rõ state/scope.

Đây là product research và placement recommendation, không phải implementation plan, lifecycle transition, hay cam kết parity. Source details và giới hạn evidence nằm trong [Problems Activity Baseline](problems-baseline.md).

## 2. Product intent và tham khảo workflow

### DB Pro

- Full Product Goal định nghĩa final rail gồm Explorer, Search, Queries, Data, ER, Agent, Monitoring, Transfer, Settings; Problems không nằm trong danh sách. Rail được giữ compact và context thuộc sidebar (`docs/goals/goal-full-product.md:221-265`).
- Query Editor đã có parser, delimiter, database/execution, và lint diagnostics; một số lint có deterministic fix. Status bar hiện đếm diagnostics của active query document, nhưng mở output dock ở Messages (`docs/design/query-editor-data-panel/query-editor-data-panel-baseline.md:227-260`; source anchors trong baseline).
- Problems hiện gom diagnostics từ toàn bộ query documents đang mở cộng workspace SQL-file diagnostics. Đây là phạm vi rộng hơn active editor, nhưng view lại gắn với activity rail/sidebar (`problems-baseline.md:13-43`).
- “Diagnostics” trong Settings là app/support summary, tách biệt SQL Problems. Hai tên và palette action phải tiếp tục phân biệt (`crates/ui/src/palette_catalog.rs:112-125`; `crates/ui/src/palette_actions.rs:19-27`).

### Visual Studio Code

- [Problems in Visual Studio Code](https://code.visualstudio.com/docs/editing/editingevolved) trình bày errors/warnings trong status bar; chọn status-bar summary mở Problems panel. Diagnostics cũng có editor markers/overview ruler và điều hướng next/previous. Đây là tham khảo cho mô hình *inline signal + on-demand aggregate panel*, không phải yêu cầu sao chép VS Code.

### JetBrains IDEs

- [Problems tool window (IntelliJ IDEA)](https://www.jetbrains.com/help/idea/problems-tool-window.html) gom current-file/project problems, hỗ trợ nhảy về editor và Quick Fix. Đây là tham khảo cho khả năng của tool window; tài liệu này không áp đặt panel phải ở cạnh nào trong DB Pro.

Các tài liệu vendor mô tả workflow tham khảo, không phải bằng chứng behavior của DB Pro hay yêu cầu parity.

## 3. Placement assessment

| Option | Fit | Trade-off |
|---|---|---|
| Dedicated activity rail/sidebar | Thấp | Luôn chiếm một destination điều hướng cho danh sách diagnostics; lệch rail mục tiêu và ép lỗi của editor/workspace thành “activity”. |
| Inline editor diagnostics only | Chưa đủ | Tốt cho lỗi tại cursor/viewport nhưng mất tổng quan trên nhiều document và workspace files hiện được gom trong Problems. |
| Workspace-level lower Problems tool panel | Cao nhất | Phù hợp mô hình IDE và không chiếm rail; cần state/list có scope rộng hơn per-document output và giữ đường mở panel nhanh. |
| Reuse `OutputTab::Messages` | Không nên dùng nguyên trạng | Messages là query execution notices/details; status bar hiện route diagnostics đến đó, nhưng list Problems có filter, cross-document/workspace scope, source locations và Quick Fix. |

### Recommended interaction contract

1. **Remove the top-level Problems activity** from the rail/sidebar. Preserve access with a Problems command and a status/count control; these should reveal/focus the lower Problems panel instead of switching activity.
2. **Make the aggregate workspace-scoped.** Preserve coverage of all open query documents and workspace SQL-file diagnostics. A future panel can be an explicit Problems tab in a generalized lower tool-panel owner, or a separate lower tool panel; it should not be represented as a per-document `OutputTab` without changing that ownership model.
3. **Keep editor-local diagnostics.** Markers and precise editor navigation remain the fastest way to see a problem at its source; the panel complements rather than replaces them.
4. **Preserve and complete current list actions.** Keep severity/source filters, row-to-source navigation, and deterministic Quick Fix. Selecting a query problem focuses its document and exact range. For workspace SQL files, open the source and position the caret at the diagnostic line; the current row handler passes only the path and does not perform this line jump.
5. **Separate three concepts.** Problems = code/workspace diagnostics; Messages = query execution notices/details; Settings → Diagnostics = redacted app support information. Do not make “Diagnostics” a second label for Problems.
6. **Use contextual counts.** Keep the per-document count in the Query status bar, and add an aggregate error/warning count only if its scope is explicit. Activating it should open the matching Problems panel/list, not just Messages.
7. **Restore old sessions deliberately.** Workspace sessions persist `Activity::Problems` by label. A cutover must define where legacy `"Problems"` activity state restores (e.g. Queries/Query with the lower panel closed or focused); do not rely accidentally on the current unknown-activity → Explorer fallback.

## 4. Implementation slices if this direction is accepted

These are sequencing suggestions, not opened lifecycle tasks:

- **P2 — Placement cutover:** remove Problems as a rail/sidebar destination; re-route the palette action and accessibility/keyboard entry point to focus/open the lower panel.
- **P2 — Workspace Problems panel:** introduce a workspace-scoped diagnostics view while retaining existing query and workspace-file source coverage, filters, navigation, and Quick Fix.
- **P2 — Status and session compatibility:** define the per-document versus aggregate count, make the click destination match that scope, and map saved `"Problems"` activity values intentionally.
- **P2 — Workspace-file row navigation:** pass the diagnostic line through file activation and place the caret at that line; selection currently opens the file without applying its diagnostic location (`problems-baseline.md:24,41`).
- **Verification:** exercise multiple open query documents plus a workspace SQL file; select each source, position the workspace-file caret at the diagnostic line, apply an available fix, reopen a saved session containing the legacy Problems activity, and confirm Settings support diagnostics remains separate. These are future acceptance scenarios, not tests run by this research.

## 5. Risks and unresolved design choices

- The existing lower panel is app-shell positioned but its content and `OutputTab` state are query-oriented. Adding a Problems label alone would not supply the all-document/workspace scope or navigation lifecycle (`problems-baseline.md:42-51`).
- The existing `ProblemEntry` stores document and diagnostic indexes; a persistent/re-sorted workspace panel should ensure row actions still resolve the intended source after document or diagnostic changes. Stable identity is a design concern, not a reproduced bug.
- The workspace scanner’s diagnostics use line numbers and do not carry source ranges or Quick Fix data in the Problems entry; future navigation can open the SQL file but cannot select a precise range from the current model (`problems-baseline.md:25-49`).
- Workspace-file rows currently open the SQL document without moving the caret to the diagnostic line, even though the diagnostic’s line is available (`problems-baseline.md:24,41`).
- Whether the workspace Problems panel should include only SQL diagnostics or later diagnostics from migrations/tasks is unconfirmed; current source aggregation does not establish broader scope.
- No independent design review or runtime evidence has been collected. Do not treat this recommendation as a completed implementation or lifecycle approval.

## 6. Evidence status

Source/product research at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. No code changed. No build, tests, native UI traversal, screenshots, persisted-session restore, or PostgreSQL/SQLite runtime was performed. Existing source tests were not run. Official links above are workflow references only.

## 7. Tổng kết

Problems đúng hơn là tool panel theo ngữ cảnh workspace/editor, không phải một mục activity cố định trong sidebar. Nên giữ diagnostics inline, thêm aggregate Problems panel ở lower workspace panel và status/count để mở nhanh; giữ riêng Messages và Settings → Diagnostics. Cần tổng quát hóa state lower panel vì output hiện tại đang theo query document, đồng thời migrate session label `"Problems"` có chủ đích.
