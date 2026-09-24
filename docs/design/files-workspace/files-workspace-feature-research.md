# Files Workspace — Feature Research and Upgrade Report

**Research date:** 2026-09-24
**Feature folder:** `docs/design/files-workspace/`
**Current source baseline:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`

## 1. Mục tiêu

Báo cáo này đối chiếu Files activity hiện tại với capability trong tài liệu chính thức về workspace, search, trust, tasks và Git. Mục tiêu:

- xác định capability Files đang có và mức hoàn thiện thực tế;
- chỉ ra safety/UX gaps dựa trên source, không suy đoán từ tên tab;
- đề xuất các **key feature hướng người dùng** và thứ tự đầu tư;
- tách feature proposal khỏi quyết định implementation.

Đây là research/design report. Không phải kế hoạch triển khai, runtime verification, hay cam kết đạt parity với sản phẩm tham khảo.

## 2. Nguồn chính thức đã tham khảo

### Visual Studio Code

- [Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)
- [Multi-root Workspaces](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces)
- [Basic editing — Search across files](https://code.visualstudio.com/docs/editing/codebasics#_search-across-files)
- [Integrate with External Tools via Tasks](https://code.visualstudio.com/docs/debugtest/tasks)
- [Source control overview](https://code.visualstudio.com/docs/sourcecontrol/overview)

### Redgate Flyway

- [Flyway schema history table and migration states](https://documentation.red-gate.com/fd/flyway-schema-history-table-273973417.html)

These sources are capability references only. They do not prove DB Pro provider behavior or determine which features DB Pro should adopt.

## 3. Hiện trạng Files

Theo source baseline `b0500b9a7ecbe37b454f3d917881154c5f7a403c`, Files đã có:

- workspace shell với recent folders, nhiều roots, trust indicator và environment labels;
- file tree hỗ trợ mở SQL, tạo file/folder, xóa, add Agent context và tìm references;
- literal search, replace preview theo số match/file, Replace All và text-based “Refactor”;
- các sub-tab Migrate, Tasks, Graph và Git;
- Git status, stage/unstage, diff-to-HEAD, commit staged và xử lý thay đổi file ngoài;
- agent context lấy active file, selection và selected table.

Các mục **Migrate**, **Graph** và **Refactor** mới chỉ có behavior hẹp: migrations được dò từ tên/path với trạng thái `Unknown`; graph dựa trên scan keyword; refactor gọi literal replace. Chi tiết và line anchors nằm trong [Files Workspace Baseline](files-workspace-baseline.md).

## 4. Gaps cần nâng cấp

### F1 — Workspace trust và file mutations chưa nhất quán

**Hiện tại:**

- Trust mặc định Untrusted và có badge/toggle.
- Replace, refactor và Tasks kiểm tra Trusted.
- Tree Delete gọi xóa recursive directory/file mà không có confirmation trong view; action mapper bỏ qua kết quả lỗi.
- Một số thao tác create cũng bỏ qua lỗi ở action mapper.

**Capability tham khảo:**

VS Code dùng Restricted Mode cho folder chưa trust và giới hạn các capability có thể tự chạy code, gồm tasks/terminal; trust được biểu thị rõ và có thể thay đổi từ workspace UI. Workspace trust không nên bị đồng nhất với quyền filesystem, nhưng đây là mẫu tốt để phân loại thao tác đọc, ghi và thực thi.

**Nên nâng cấp:**

- Xác nhận destructive delete, hiển thị target chính xác và phân biệt file/folder.
- Không nuốt filesystem errors; báo thành công/thất bại với path.
- Phân loại quyền theo thao tác: browse/open, write/replace/delete, execute task, gửi dữ liệu vào Agent context.
- Áp dụng trust policy nhất quán cho những thao tác có thể chạy code hoặc tác động hàng loạt; nêu rõ lý do khi bị chặn.
- Khi có nhiều roots, thể hiện trust theo từng root hoặc quy tắc trust của toàn workspace; thêm root chưa trust không được âm thầm nâng quyền các root khác.

**Ưu tiên:** P1 — cần chốt safety contract trước khi mở rộng replace/refactor/task workflow.

**Action đề xuất:**

```text
RequestDeletePath { root, path }
ConfirmDeletePath { root, path }
SetWorkspaceTrust { root, level }
ReportWorkspaceOperation { operation, path, result }
```

### F2 — Search/Replace cần thành một workbench có thể review

**Hiện tại:**

- Search literal, case-insensitive; không có include/exclude scope, regex, whole-word hay per-root filter.
- Kết quả giới hạn ở 100 hits; UI hiển thị 40; preview replace chỉ liệt kê số hit theo file.
- Replace-all ghi các file lần lượt. Preview không đóng băng/revalidate version của file trước khi apply.
- “Rename symbol” là literal replacement, không có SQL symbol resolution.

**Capability tham khảo:**

VS Code trình bày kết quả cross-file theo file và vị trí, cho mở từng match tại editor; có regex, match-case, whole-word và include/exclude glob patterns.

**Nên nâng cấp thành key feature — Reviewable SQL Workspace Search:**

- Scope theo root/path/file type; include/exclude glob; match-case, whole-word và regex có semantics rõ.
- Nhóm hit theo root → file → line; click mở đúng document, line và match.
- Preview before/after theo từng file hoặc từng match; cho chọn file/match cần apply.
- Revalidate nội dung trước khi ghi; phát hiện dirty editor buffer và disk conflict.
- Báo số match được thay, file lỗi, file bị bỏ qua; không hiển thị success nếu apply dở dang.
- Giữ **Rename symbol** riêng; chỉ gọi refactor khi có SQL-aware binding hoặc nêu rõ đây là text replace.

**Ưu tiên:** P1 — đây là capability kế tiếp tận dụng Search UI/model đang có và giảm rủi ro mutation hàng loạt.

**Action đề xuất:**

```text
SetWorkspaceSearch { query, scope, include, exclude, mode }
PreviewWorkspaceReplace { query, replacement, scope }
ApplySelectedReplacements { preview_id, selected_matches }
OpenSearchHit { root, path, line, column }
```

**Acceptance criteria:**

- Search hai roots phân biệt đúng path trùng tên.
- Include/exclude không đọc hoặc sửa file ngoài scope.
- Regex/match-case/whole-word có test boundary riêng.
- Preview phản ánh đúng nội dung sẽ ghi.
- File đổi sau preview hoặc có editor buffer dirty thì apply không ghi đè âm thầm.
- Lỗi một file được báo cụ thể; UI không báo thành công toàn phần nếu chỉ ghi một phần.

### F3 — Workspace nhiều root cần identity và persistence thật

**Hiện tại:**

- Có thể add/select/remove nhiều roots; tree theo active root trong khi search quét index của mọi root.
- Root ID được tạo từ vị trí thêm root; environment names được khởi tạo sẵn nhưng connection/database/schema đều `None`.
- Source Files state không cho thấy workspace manifest lưu root names, trust và expansion state để khôi phục sau restart.

**Capability tham khảo:**

VS Code multi-root hỗ trợ tên hiển thị từng root, search theo root, lưu/mở workspace file và loại trừ folder khỏi file/search view.

**Nên nâng cấp:**

- Lưu workspace manifest có root path, display name, stable ID, trust policy và state được chọn rõ.
- Thêm/remove/rename root không làm đổi identity của root còn lại.
- Search result và file tab luôn disambiguate cùng tên file ở nhiều root.
- Định nghĩa ignore/exclude rules; hiện scan có danh sách skip hard-code.
- Chỉ gọi environments là environment profile sau khi profile ánh xạ thật tới connection/database/schema; nếu chưa, đổi presentation để không ngụ ý DB mapping.

**Ưu tiên:** P1 nếu target Files là project workspace lâu dài; P2 nếu Files chỉ cần session-local browsing.

### F4 — Migrate cần trạng thái từ database, không chỉ dò file

**Hiện tại:**

Migrate tab dò path/file name và gán `Unknown`; action duy nhất là mở SQL file. DB Pro đã có Schema Compare/Migration flow bên ngoài Files, nên không nên tạo một đường apply migration thứ hai không dùng chung safety policy.

**Capability tham khảo:**

Flyway theo dõi migration trong schema history table, bao gồm trạng thái, checksum và thành công/thất bại; có thể đối chiếu migrations với database. Đây là đối chiếu về workflow, không phải chỉ định dùng Flyway metadata trong DB Pro.

**Nên nâng cấp:**

- Đổi từ Migration File List thành Migration Status & Review: source root, connection/schema target, version/order, checksum drift, pending/applied/failed.
- Tái sử dụng Schema Compare/Migration Planner và mutation confirmation hiện có, thay vì viết executor riêng.
- Preview SQL và impact/risk trước khi gửi command; provider capability rõ cho PostgreSQL và SQLite.
- Không tự execute migration khi mở tab/refresh; yêu cầu hành động xác nhận rõ ràng.

**Ưu tiên:** P1 khi Files được dùng làm migration workflow; nếu chưa có target DB/history contract thì giữ là P2/backlog.

### F5 — Tasks cần named tasks và execution feedback rõ

**Hiện tại:**

Một ô free-form chạy một shell command trong root đã trust, lưu last result và cắt ngắn output; không có danh sách project tasks, streaming, cancel hay history.

**Capability tham khảo:**

VS Code tasks tích hợp scripts/tools theo project config, có task labels, command arguments, output handling, problem matcher và compound tasks; Workspace Trust gate task execution.

**Nên nâng cấp:**

- Named task từ project config hoặc command palette, không tự chạy task khi mở workspace.
- Hiển thị command/cwd/environment, có confirmation theo trust/risk policy.
- Output panel có streaming, exit status, duration, cancel/timeout và lịch sử giới hạn.
- Map known diagnostics sang Problems view; không parse arbitrary output thành problem nếu không có matcher.

**Ưu tiên:** P2 — UX đang là runner tối giản; trước hết chuẩn hóa trust boundary và output/error handling.

### F6 — Git nên phát triển thành Review Changes workflow

**Hiện tại:**

Files Git đã stage/unstage file, xem diff với HEAD, commit staged, probe status và cảnh báo disk change. Chưa có hunk-level staging, history/branch/remotes/push-pull/conflict resolution trong view.

**Capability tham khảo:**

VS Code Source Control ghép Changes/Staged Changes, diff editor, commit và graph/history; có workflow riêng để push/pull, undo và xử lý conflict.

**Nên nâng cấp:**

- Diff có line-level navigation và staged/unstaged state rõ.
- Review changed SQL trong editor trước commit; phân biệt working-tree diff với staged diff.
- Hiển thị branch, incoming/outgoing state và error action cụ thể trước khi tính remote support.
- Chỉ thêm hunk staging/history/merge khi có use case và Git adapter contract tương ứng.

**Ưu tiên:** P2 — Git core local đã có; tăng reviewability trước khi mở rộng remote operations.

### F7 — Diagnostics và dependency view cần SQL semantics

**Hiện tại:**

Diagnostics nhận diện hai textual patterns; Graph quét keyword và hiển thị danh sách tối đa 60 dòng. Các kết quả này có thể hữu ích như heuristic nhưng không đủ chính xác để gọi là SQL diagnostics/dependency graph.

**Nên nâng cấp:**

- Dùng SQL parser/tokenizer để loại comment/string và giữ source ranges.
- Chỉ hiển thị resolved references khi connection/schema context xác thực được; phân loại unresolved/ambiguous.
- Gắn diagnostic với line/range, severity, nguồn rule và quick action nếu có semantics an toàn.
- Provider-specific SQL behaviors phải độc lập; không suy ra PostgreSQL behavior từ SQLite hoặc ngược lại.

**Ưu tiên:** P2 — làm sau khi source ranges/context model ổn định; không nâng heuristics thành blocking diagnostics.

## 5. Đề xuất roadmap theo feature

### Phase 1 — Safe, useful file workflow

1. F1 — workspace trust/mutation safety và surfaced errors.
2. F2 — reviewable SQL workspace search/replace.
3. F3 — xác định rõ workspace session-local hay persistent; nếu persistent, thêm stable root identity/manifest.

### Phase 2 — Database-project workflows

1. F4 — database-backed migration status/review dùng chung Schema Compare/Migration Planner.
2. F5 — named task runner với output/cancel/error model.
3. F6 — review changes cho Git; giữ remote/merge scope riêng.

### Phase 3 — Semantic project intelligence

1. F7 — SQL-aware diagnostics và dependency navigation.
2. Environment profiles chỉ khi có mapping thật tới saved connection/database/schema và provider behavior được xác định.

## 6. Khuyến nghị scope cho feature đầu tiên

Không mở thêm sub-tab mới. Key feature đầu tiên nên là **Reviewable SQL Workspace Search/Replace**, nhưng gộp scope safety tối thiểu của F1 làm acceptance gate:

```text
Search query + scope
  → grouped hits with root/path/line
  → preview exact changed text
  → select changes
  → verify files/buffers have not diverged
  → apply selected changes
  → report written/skipped/failed paths
```

Trước khi feature này được xem là hoàn tất:

- Xóa file/folder phải có confirmation, exact target, và error feedback.
- Replace/refactor chỉ ghi trong trusted root theo chính sách đã định; không ghi đè buffer dirty hoặc file đã đổi sau preview.
- Apply phải có behavior rõ khi ghi dở dang; không claim atomicity nếu không có transaction/filesystem staging protocol.
- Hành vi provider DB không áp dụng ở đây; đây là local workspace capability.

## 7. Không đầu tư trước

- Không gọi text replacement là semantic rename/refactor.
- Không gọi migration file listing là migration status hoặc migration runner.
- Không gọi keyword scan là dependency graph đã resolve.
- Không chạy project scripts/tasks khi mở workspace hoặc refresh.
- Không thêm push/pull/merge support trước khi local Git review and error states đủ rõ.
- Không đưa environment labels giả thành connection profiles.
- Không clone toàn bộ VS Code Explorer; lấy các capability phù hợp với SQL-project workflow.
- Không gộp generate/preview/apply file changes thành một action không thể review.

## 8. Kết luận

Files đã có một nền tảng hữu dụng: multi-root browsing, SQL file actions, search/replace cơ bản, Git operations, tasks và agent context. Khoảng trống lớn nhất là **độ tin cậy của thao tác ghi/xóa**, **khả năng review chính xác trước khi apply**, và **khác biệt giữa tab label với capability thực**.

Thứ tự đề xuất: **F1 safety contract → F2 reviewable SQL Search/Replace → F3 workspace identity/persistence decision → migration/task/Git workflows → SQL-aware diagnostics/dependencies**. Đây là feature roadmap để product review, không phải implementation authorization.

## 9. Evidence boundary

Các claim về DB Pro hiện trạng dựa trên source SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và được phân loại là source evidence trong [baseline](files-workspace-baseline.md). Các capability so sánh dựa trên tài liệu chính thức được liệt kê tại §2. Chưa chạy UI runtime traversal, destructive-operation scenario, task execution scenario, live Git repo matrix, PostgreSQL migration scenario, hay SQLite migration scenario trong research này.
