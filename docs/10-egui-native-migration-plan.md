# Kế hoạch migrate UI từ React sang egui native

**Trạng thái:** Đề xuất để review trước khi bắt đầu implementation  
**Phạm vi:** DB Pro desktop UI  
**Mục tiêu:** thay toàn bộ React/Vite/WebView UI bằng UI native Rust dùng `egui`/`eframe`, giữ nguyên domain, application services, database adapters và các hợp đồng dữ liệu hiện có.

## 1. Tóm tắt quyết định

Đây không nên là một phép “dịch JSX sang egui” trực tiếp. React hiện tại đảm nhiệm cả UI, orchestration, cache và một phần runtime; egui chỉ thay lớp presentation/event loop. Kiến trúc đích nên là:

```text
┌──────────────────────────────────────────────────────────┐
│ eframe/winit + egui native                               │
│ shell · panels · tabs · dialogs · grid · editor · graph  │
├──────────────────────────────────────────────────────────┤
│ Rust UI application state + command/event reducer        │
│ async task bridge · persistence · focus/keyboard         │
├──────────────────────────────────────────────────────────┤
│ db-pro-core application services                         │
│ connection · query · schema · table data · export       │
├──────────────────────────────────────────────────────────┤
│ db-pro-infrastructure                                   │
│ PostgreSQL · SQLite · secrets · metadata · SSH          │
└──────────────────────────────────────────────────────────┘
```

**Khuyến nghị:** bỏ Tauri khỏi runtime production sau khi cutover. Không chạy `egui` bên trong Tauri WebView và không giữ React làm hidden fallback lâu dài; hai event loop/window model sẽ làm tăng độ phức tạp và không đem lại lợi ích native thực sự.

Tauri commands hiện nằm ở `crates/tauri-app`; cần tách chúng thành application-facing Rust API trước, sau đó `egui` gọi các service/API này trực tiếp qua task bridge. Các DTO hiện ở `crates/tauri-app/src/dto.rs` được chuyển thành wire/domain view models dùng chung.

## 1.1. Định hướng visual: native nhưng mang chất Codex hiện đại

UI native không được trông như một form CRUD cổ điển. Mục tiêu là một database workspace có cảm giác **sắc nét, tối giản, premium và keyboard-first**, lấy cảm hứng từ các IDE/AI workspace hiện đại như Codex nhưng không sao chép logo, branding, layout hoặc visual asset của sản phẩm khác.

### Visual principles

- **Dark-first, high signal:** nền graphite/charcoal nhiều lớp, tương phản vừa đủ, không dùng màu đen tuyệt đối cho mọi surface.
- **Calm density:** compact để hiển thị nhiều thông tin nhưng vẫn có breathing room; không nhồi quá nhiều border và control.
- **One strong accent:** dùng một accent chính cho action/focus/active state; màu trạng thái success/warning/error chỉ dùng cho semantic feedback.
- **Layered surfaces:** app background → panel → elevated panel → popover/modal; phân cấp bằng tone, shadow nhẹ và hairline border.
- **Typography rõ ràng:** Geist/Inter-like sans cho UI, JetBrains Mono cho SQL, value và metadata; số liệu phải dễ scan.
- **Progressive disclosure:** sidebar và panel hiển thị vừa đủ; advanced option nằm trong popover/section mở rộng, không làm UI mặc định nặng.
- **Motion có mục đích:** transition rất ngắn cho panel/dialog/tab và query progress; không animation gây chậm thao tác database.
- **Keyboard-first nhưng discoverable:** mọi action quan trọng có shortcut, tooltip và command palette; focus state luôn nhìn thấy.
- **Data is the hero:** query editor, result grid và schema phải là vùng nổi bật nhất; chrome không được lấn át dữ liệu.

### Token baseline đề xuất

Các token này là điểm xuất phát, cần được kiểm tra bằng screenshot trên màn hình thật trước khi chốt:

```text
surface-app       #0B0D10
surface-panel     #11151A
surface-elevated  #171C23
surface-hover     #1D2530
border-subtle     #252D38
text-primary      #F3F5F7
text-secondary    #9AA5B1
text-muted        #697586
accent            #8B8CFF hoặc #A78BFA
success           #35C48A
warning           #E7B65A
danger            #F06A7A
focus-ring        accent với alpha 45%
```

- Không hard-code token trong widget; gom vào `DbProTheme` và map sang `egui::Visuals`.
- Cung cấp light theme sau khi dark theme ổn định, nhưng mọi component phải có semantic token thay vì đảo màu thủ công.
- Tạo token snapshot/gallery cho shell, tab, dialog, grid, editor, error và empty state.
- Icon dùng một bộ nhất quán, nét mảnh, kích thước nhỏ; không trộn nhiều phong cách icon.

### Layout và component direction

- App shell giống một professional workspace: activity rail mảnh, sidebar có hierarchy, topbar nhẹ, content canvas rộng.
- Tab có active indicator tinh tế, pinned/preview state dễ phân biệt nhưng không làm tab bar rối.
- Buttons ưu tiên `primary`, `secondary`, `ghost`, `destructive`; tránh quá nhiều pill/button màu.
- Dialog có title, context, primary action và Escape/Cancel rõ ràng; destructive dialog phải hiển thị target và SQL classification.
- Empty/loading/error state dùng illustration/icon nhỏ và copy ngắn, không để vùng trống vô nghĩa.
- Query result có toolbar gọn, sticky metadata, column type indicator và status bar giống công cụ chuyên nghiệp.
- Command Palette và Quick Open là first-class surface: search nhanh, fuzzy match, keyboard navigation, recent context.
- Agent panel nếu giữ lại phải cùng visual system, không biến thành chat UI tách rời khỏi database context.

### Visual acceptance gate

Mỗi milestone UI phải có screenshot review ở ba kích thước 1280×800, 1440×900 và 1920×1080, ở cả trạng thái bình thường và loading/error/empty. Không chấp nhận milestone nếu:

- layout trông như form admin mặc định hoặc widget demo của egui;
- typography, spacing, icon hoặc focus state không nhất quán;
- active/hover/disabled/error state không phân biệt được;
- panel borders quá nặng, accent dùng quá nhiều hoặc data bị chrome lấn át;
- shortcut/command palette và resize behavior không cho cảm giác desktop-native mượt.

## 2. Hiện trạng cần migrate

### Những phần sẽ được giữ lại

- `crates/core`: domain types, validation, application services, ports.
- `crates/infrastructure`: PostgreSQL, SQLite, metadata store, keyring, SSH, backup.
- `frontend/src/modules/*/services`, SQL classifier/generator, pure utilities và các business rule có thể viết lại thành Rust hoặc giữ tạm dưới dạng fixture/reference.
- Các hợp đồng tính năng: connection lifecycle, query cancellation, typed cells, bounded result, schema introspection, editable rows, export, backup, audit/confirmation.

### Những phần sẽ bị thay thế

- `frontend/src/App.tsx`, TanStack Router, React providers và toàn bộ component tree.
- shadcn/Radix/Tailwind/CSS tokens.
- Zustand/TanStack Query bằng một Rust `AppState` và event reducer.
- `@monaco-editor/react` bằng editor native (giai đoạn đầu có thể dùng editor đơn giản; không nên cố tái tạo toàn bộ Monaco ngay ngày đầu).
- `@tanstack/react-virtual` bằng virtualized grid native.
- Cytoscape/React Flow bằng renderer graph native hoặc tạm hoãn ER diagram đến sau cutover.
- Tauri window/frontend bridge và `frontendDist` trong `tauri.conf.json`.

### Các bề mặt UI chính

| Bề mặt hiện tại | Component/source tiêu biểu | Đích egui |
|---|---|---|
| App shell | `commons/components/shell/*` | `ui/shell`, activity bar, topbar, sidebar, status bar |
| Workspace | `workspace-content.tsx`, `workspace-tab-bar.tsx` | tab model + central panel renderer |
| Connections | `modules/connection/components/*` | connection list/editor/dialog |
| Query | `modules/query/components/*` | SQL editor, toolbar, history, result tabs |
| Data | `modules/data-grid`, `modules/unified-grid` | virtual grid, cell editor, copy, staged mutations |
| Schema | `modules/schema/components/*` | explorer, object workbench, DDL, indexes, relations |
| ER diagram | `modules/er-diagram/*` | custom painter/graph view, hoặc phase sau |
| Export/backup/users | tương ứng trong `modules/*` | native modal/progress/forms |
| Agent preview | `commons/components/ide/agent-panel.tsx` | panel + confirmation/audit surfaces |

## 3. Nguyên tắc không được phá vỡ

1. **Không đưa credential về UI state.** Secret vẫn chỉ được resolve trong backend/service layer.
2. **Không bypass application service.** egui không gọi PostgreSQL/SQLite trực tiếp.
3. **Không dùng string-only result.** Giữ `CellValue`, metadata cột, null/numeric/binary/temporal/JSON.
4. **Không stream row lớn vào một global store vô hạn.** Dùng bounded channel, page/batch và cancellation theo request ID.
5. **Không đổi behavior khi chưa có parity test.** Migration trước hết là thay presentation, không phải rewrite business rules.
6. **Mọi destructive action vẫn có confirmation** gồm connection, target, operation class và SQL preview.
7. **UI thread không block.** Query, introspection, export, backup, connect/test đều chạy async task; UI chỉ poll/nhận event.
8. **State là một chiều:** `UserIntent -> Command -> Service -> AppEvent -> Reducer -> repaint`.
9. **Native accessibility và keyboard-first** phải được thiết kế lại, không giả định ARIA/DOM còn tồn tại.
10. **Persist state có version/migration:** tab, panel width, theme, editor preferences, recent connections, workspace.

## 4. Kiến trúc đích chi tiết

### 4.1 Workspace mới

Tạo workspace members:

```text
crates/core/                 # giữ domain/application/ports
crates/infrastructure/       # giữ adapters
crates/ui/                   # egui widgets, screens, AppState, reducer
crates/app/                  # bootstrap, runtime, task bridge, native window
```

Tên crate có thể là `db-pro-ui` và `db-pro-app`; không cần đổi crate ngay trong spike. `crates/tauri-app` được giữ trong giai đoạn transitional để React và egui cùng dùng một backend, sau đó xoá khi cutover.

### 4.2 Task bridge

Định nghĩa typed command/event:

```rust
pub enum UiCommand {
    LoadConnections,
    TestConnection(ConnectionConfigInput),
    Connect(ConnectionId),
    ExecuteQuery(ExecuteQueryRequest),
    CancelQuery(RequestId),
    Introspect(ConnectionId),
    FetchTablePage(FetchTablePageRequest),
    UpdateRow(UpdateRowRequest),
    Export(ExportRequest),
}

pub enum UiEvent {
    Loading { request_id: RequestId, operation: Operation },
    ConnectionsLoaded(Vec<ConnectionSummary>),
    QueryBatch(QueryBatch),
    QueryCompleted(QueryCompleted),
    Failed { request_id: RequestId, error: DbErrorDto },
    Notification(Notification),
}
```

- `UiCommand` từ egui thread gửi qua `tokio::sync::mpsc`.
- worker gọi application service và gửi `UiEvent` qua bounded channel.
- mỗi frame drain một số lượng event có giới hạn rồi gọi `ctx.request_repaint()` khi còn task pending.
- stream batch phải có sequence, request ID và giới hạn byte/row.
- không dùng `Arc<Mutex<AppState>>` như một nơi để mọi task mutate tuỳ ý; chỉ reducer/UI thread được mutate state hiển thị.

### 4.3 AppState

Chia nhỏ state thay vì một struct khổng lồ:

```text
AppState
├── connection_state
├── explorer_state
├── workspace_state
├── query_state
│   ├── editors per query tab
│   ├── execution state per request
│   └── result pages
├── schema_state
├── grid_state
├── overlay_state       # dialog, command palette, quick open, confirm
├── settings_state
├── agent_state
└── diagnostics
```

Mỗi state có `UiAction`/`UiEvent` rõ ràng, trạng thái `Loading/Ready/Empty/Error`, và không giữ dữ liệu nhạy cảm.

### 4.4 Native window/bootstrap

`eframe::run_native` khởi tạo window, theme, fonts, icon và Tokio runtime. Cần xử lý:

- close request và dirty-tab guard;
- minimize/maximize/fullscreen theo từng OS;
- high-DPI/scale factor;
- clipboard, file picker, open/save path;
- native menu/shortcut;
- persistence path dùng app data directory hiện tại;
- graceful shutdown: cancel query, flush workspace/meta state, đóng connection.

## 5. Lộ trình theo phase

### Phase 0 — Chốt phạm vi và baseline (3–5 ngày)

**Mục tiêu:** khóa behavior hiện tại trước khi viết UI mới.

Công việc:

- Chạy toàn bộ frontend/Rust quality gates hiện tại và lưu baseline.
- Lập catalog command từ `crates/tauri-app/src/lib.rs` và `commands/*`.
- Đánh dấu command nào cần chuyển thành service API dùng chung, command nào chỉ là adapter DTO.
- Chụp baseline workflow bằng fixture: connect, query, cancel, schema, grid edit/delete, export, backup.
- Chốt scope MVP native: connection + shell + query/result + schema explorer; grid edit, ER, agent/users/backup có thể follow-up.
- Viết ADR xác nhận bỏ Tauri WebView và chọn `eframe`/`egui` version.
- Đặt performance budgets: startup, frame time, query first-result, memory với 100k rows.
- Review và chắt lọc visual direction từ `prototype-wireframe.html`, `db-pro-concept-c-20260805.html` và các demo hiện có; prototype chỉ là reference, không bê nguyên CSS/DOM sang egui.
- Chốt một visual moodboard/token sheet theo hướng Codex-inspired: graphite surfaces, lavender/indigo accent, compact typography, subtle borders và high information density.

**Exit criteria:** có parity matrix, danh sách blocker, quyết định scope và benchmark baseline; có visual token sheet, component inventory và ít nhất một shell mockup được review.

### Phase 1 — Tách backend khỏi Tauri (1–2 tuần)

**Mục tiêu:** React và egui có thể chạy trên cùng application/service layer.

Công việc:

- Tạo `db-pro-runtime`/`db-pro-app` chứa bootstrap registry, metadata store, secret store, cancellation registry và service wiring hiện đang ở `tauri-app/src/lib.rs`.
- Di chuyển DTO dùng chung ra `crates/core` hoặc crate `db-pro-contracts`; DTO UI không phụ thuộc Tauri.
- Chuyển `CommandError` thành stable `DbErrorDto`/`UiError` dùng được ngoài IPC.
- Tạo facade typed: `ConnectionApi`, `QueryApi`, `SchemaApi`, `TableDataApi`, `ExportApi`, `BackupApi`, `UserApi`.
- Giữ Tauri command mỏng, chỉ map JSON input/output vào facade để React không vỡ trong giai đoạn transition.
- Thêm unit/integration tests cho facade và cancellation.

**Exit criteria:** Tauri commands chỉ còn adapter; service facade test chạy không cần WebView/Tauri.

### Phase 2 — egui shell spike (1 tuần)

**Mục tiêu:** chứng minh window/event loop/layout/density/keyboard.

Công việc:

- Tạo binary native chạy được trên Linux/macOS/Windows target chính.
- Dựng dark theme, color tokens, typography, spacing, icon strategy, focus ring.
- Implement activity bar, topbar, statusbar, sidebar, splitter resize, central workspace.
- Implement `UiCommand`/`UiEvent`, reducer, async task bridge, error snackbar/log panel.
- Implement menu/shortcut registry: `Cmd/Ctrl+P`, `Cmd/Ctrl+Shift+P`, F5, Escape, close tab.
- Chứng minh clipboard và file picker native.

**Exit criteria:** shell responsive, resize không block, shortcuts hoạt động trên Linux/macOS/Windows; không có service call trên UI thread.

### Phase 3 — Connection vertical slice (1 tuần)

**Mục tiêu:** user có thể quản lý và connect database bằng egui.

Công việc:

- Connection list, create/edit dialog, test/connect/disconnect/reconnect/delete.
- Driver-specific form PostgreSQL/SQLite, SSL/SSH path, readonly, tags/color.
- Loading/empty/error/restricted states.
- Secret input không đưa password vào log/persisted UI state.
- Startup restore/reconnect và orphan workspace behavior.

**Exit criteria:** pass connection lifecycle parity tests và manual smoke với PostgreSQL + SQLite.

### Phase 4 — Query editor và execution (2–3 tuần)

**Mục tiêu:** hoàn thiện vertical slice quan trọng nhất.

Công việc:

- Chọn editor:
  - **Giai đoạn 1:** editor text native tối thiểu (multiline, selection, undo/redo, cursor, line numbers, search).
  - **Giai đoạn 2:** syntax highlight SQL, diagnostics, completion, hover, snippets, zoom.
- Port query tab, saved queries, query history, run config, current selection/run all policy.
- Port command bar, explain, stop/Escape và request identity.
- Query task bridge nhận `Started/Batch/Completed/Failed`.
- Result grid read-only trước, typed cell renderer và column metadata.
- Persist content/selection/zoom theo tab khi cần.

**Exit criteria:** read-only SQL flow parity; cancel không để stale result ghi đè tab khác; query/result benchmark đạt budget.

### Phase 5 — Virtualized data grid và mutation (2 tuần)

**Mục tiêu:** thay `TanStack Virtual` + React grid bằng native renderer.

Công việc:

- Thiết kế row/column virtualization theo visible rect, overscan và fixed/variable row height.
- Column resize, sort/filter/pagination, row selection, copy cell/row/column.
- Typed cell editor: null/bool/number/text/UUID/date/JSON/bytes.
- Staged update/delete, same-row patch composition, PK requirement, readonly no-PK.
- Dirty/conflict/revision-safe partial failure states.
- Keyboard navigation và accessibility fallback cho cell hiện tại.

**Exit criteria:** không lag khi 100k rows bounded/page; mutation parity với tests hiện tại; không gửi full row nếu chỉ đổi một số column.

### Phase 6 — Explorer/schema/workspace parity (2 tuần)

**Mục tiêu:** migrate navigation và object workbench.

Công việc:

- Tree explorer: connection/schema/table/view, targeted refresh, loading/error/empty.
- Workspace tab model: preview/permanent/pinned, collision title, orphan recovery, close guard.
- DB Object: Data, Columns, Indexes, Relations, DDL, triggers/CRUD actions.
- Quick Open và Command Palette dùng cùng registry command.
- Persist workspace migration/version.

**Exit criteria:** mở object từ tree thành đúng tab, refresh không mất context, close guard đầy đủ.

### Phase 7 — Export, backup, users, agent (1–2 tuần)

**Mục tiêu:** migrate tính năng phụ sau core ổn định.

- Native file picker + export CSV/JSON/Excel progress/cancel.
- Backup/restore progress và confirmation.
- User/privilege views.
- Agent panel, action confirmation, audit buffer, cancellation identity.
- Không mở rộng agent capability trong migration; chỉ giữ preview parity.

### Phase 8 — ER diagram và editor nâng cao (2–3 tuần, có thể tách release)

**Mục tiêu:** thay Cytoscape/React Flow mà không làm trễ core release.

- Tách layout algorithm hiện có khỏi React-facing code; tái dùng các utility thuần nếu có thể.
- Worker layout bằng Rust task hoặc thread riêng.
- Native painter: pan/zoom, node/edge LOD, spatial index, search-first, neighborhood.
- Kiểm tra scale với large schema.
- Nếu chi phí vượt budget, ship native MVP với schema lists và defer ER diagram; không nhúng WebView chỉ cho một màn hình nếu mục tiêu là native thuần.

### Phase 9 — Cutover, xoá React/Tauri (1 tuần)

- Chạy parity suite song song trên React và egui cho đến khi đạt ngưỡng.
- Đổi release binary sang native app.
- Xoá `frontend/`, `tauri.conf.json` frontend build fields, Tauri plugins không còn dùng, command adapters cũ.
- Cập nhật README, CI, packaging, release checklist, crash reporting/logging.
- Kiểm tra clean checkout build được trên Linux/macOS/Windows.

## 6. Mapping state và component

| React hiện tại | egui/native tương ứng | Ghi chú |
|---|---|---|
| TanStack Query | task bridge + request registry + reducer | cache server state explicit, invalidation bằng event |
| Zustand stores | `AppState` sub-state + persistence | không mutate từ worker |
| React Router | `Screen`/`WorkspaceTabKind` + command navigation | desktop app không cần URL routing |
| Radix Dialog | `egui::Window`/custom modal stack | focus trap và Escape phải tự kiểm thử |
| Sonner | notification queue | timeout và severity rõ ràng |
| Monaco | native editor | đây là rủi ro lớn nhất; scope riêng |
| TanStack Virtual | visible-range painter | benchmark trước khi port toàn bộ grid |
| Cytoscape | custom `egui::Painter` | defer được |
| `invoke` | typed facade + channel | bỏ JSON/IPC overhead nội bộ |
| Tauri events | typed `UiEvent` | lifecycle vs data stream phân biệt rõ |
| CSS variables | `DbProTheme`/`egui::Visuals` | token snapshot test nếu cần |

## 7. Testing strategy

### Unit/property tests

- Reducer: cùng một chuỗi `UiEvent` luôn cho state cuối xác định.
- Request registry: stale request không được mutate tab hiện hành.
- Workspace migration, tab collision, close guard.
- SQL selection/splitting/classification giữ behavior backend.
- Cell codec, formatting, sorting/filtering, patch composition.
- Virtualization visible range và hit-testing.
- Shortcut conflict và focus routing.

### Integration tests

- Connection lifecycle PostgreSQL/SQLite.
- Query typed result, stream batches, timeout/cancel, multi-tab isolation.
- Schema introspection và invalidation.
- PK update/delete, no-PK readonly, partial failure.
- Export/backup cancellation và path policy.
- Secret redaction và không leak credential vào debug/log.

### Native/manual tests

Ma trận tối thiểu: Linux X11/Wayland, macOS, Windows; scale 100/125/150/200%; window resize/minimize/reopen; keyboard-only; clipboard; file picker; dark theme; large schema; 100k-row result.

Mỗi phase phải có screenshot/screen recording parity cho flow tương ứng và một smoke checklist chạy được từ clean checkout.

## 8. Performance budgets đề xuất

Các số này cần đo lại ở Phase 0, nhưng nên dùng làm guardrail ban đầu:

- cold start đến interactive: `<= 1.5s` không tính connect database;
- steady-state frame time: p95 `<= 16 ms` ở shell/grid bình thường;
- scroll grid 100k rows bounded: không allocation theo toàn bộ row set, không dropped frame kéo dài;
- first query result: không chậm hơn React baseline quá 10% trên cùng fixture;
- memory idle sau startup: đo baseline native; mục tiêu giảm đáng kể bundle/webview footprint;
- cancellation acknowledgment: UI phản ánh stopped trong `<= 250 ms` sau event backend.

## 9. Rủi ro và cách giảm thiểu

| Rủi ro | Mức | Giảm thiểu |
|---|---:|---|
| Native SQL editor không đạt Monaco | Cao | ship editor MVP trước, completion/diagnostic tách phase; không tự hứa parity 100% |
| egui ecosystem thiếu widget desktop phức tạp | Cao | lập design system nhỏ, ưu tiên widgets cần thiết; tránh port mọi shadcn primitive |
| Grid/ER renderer tốn thời gian | Cao | benchmark spike sớm; defer ER; render visible range |
| egui immediate mode gây mất state | Cao | state model explicit, không đặt dữ liệu quan trọng trong widget call; reducer tests |
| Accessibility/native IME kém | Cao | test keyboard/IME từng OS từ Phase 2, abstraction cho text input |
| Async task race/stale result | Cao | request ID, generation/tab ID, reducer-only mutation |
| Native packaging khó hơn Tauri | Trung bình | CI matrix sớm từ Phase 2; kiểm tra system deps và signing |
| Rewrite business logic hai lần | Trung bình | tách facade trước, giữ core; không port service vào UI |
| Scope creep do “native hóa” | Cao | freeze MVP; ER/agent/editor nâng cao là phase riêng |
| Regression security | Cao | giữ service boundary, redaction tests, review capability không còn qua Tauri |

## 10. Definition of Done cho cutover

Chỉ xoá React/Tauri frontend khi tất cả điều kiện sau đạt:

- Native binary build/package được trên ba OS mục tiêu.
- Connection, read-only query, typed result, cancel, schema explorer và workspace đạt parity acceptance.
- Grid edit/delete, export/backup và confirmation/audit đã có scope rõ: migrated hoặc chính thức defer trong release notes.
- Không còn service call trực tiếp từ UI renderer; không có blocking I/O trên UI thread.
- Rust fmt/check/clippy/test và native UI tests đều xanh trên CI.
- Manual smoke keyboard, DPI, clipboard, file picker, close/reopen, crash/restart hoàn tất.
- Security review xác nhận không leak password/private key/query params nhạy cảm.
- Startup, frame time, memory và large-result benchmark đạt budget hoặc có exception được ghi trong ADR.
- Workspace/meta schema có migration/version; người dùng hiện tại không mất connections, history, settings, tabs nếu compatibility được cam kết.
- README, docs architecture, release checklist và developer setup đã cập nhật; clean checkout không cần Node/pnpm.

## 11. Backlog implementation đề xuất

Các PR nên nhỏ và có thể rollback:

1. `native: add runtime facade and shared contracts`
2. `native: scaffold eframe app and task bridge`
3. `native: implement shell and theme tokens`
4. `native: migrate connection workflow`
5. `native: migrate query execution and typed results`
6. `native: add native sql editor MVP`
7. `native: add virtualized result grid`
8. `native: migrate workspace/explorer/schema`
9. `native: migrate export/backup/agent overlays`
10. `native: migrate ER diagram or explicitly defer`
11. `native: native CI/package matrix`
12. `native: remove React/Tauri frontend after parity sign-off`

Mỗi PR phải có: scope/known gaps, tests, screenshot hoặc video ngắn của flow UI, benchmark nếu ảnh hưởng render, và ghi chú migration/persistence.

## 12. Câu hỏi cần chốt trước Phase 1

1. “Native” có nghĩa là **egui/eframe thuần, không WebView** không? Kế hoạch này giả định là có.
2. SQL editor giai đoạn đầu chấp nhận thiếu completion/diagnostics so với Monaco không?
3. ER diagram có bắt buộc trong native MVP hay được defer sau cutover?
4. Có giữ Windows/macOS parity ngay từ đầu, hay Linux-first như baseline hiện tại?
5. Có cần backward compatibility với workspace/meta data của bản React 0.1.0 không?
6. Native UI sẽ giữ dark-first visual language hiện tại hay cho phép visual redesign nhỏ trong lúc migrate?

Nếu các câu hỏi trên chưa được chốt, nên chỉ bắt đầu Phase 0/1 và không bắt đầu port hàng loạt màn hình.
