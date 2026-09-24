# Sidebar Explorer — Feature Research and Upgrade Report

**Research date:** 2026-09-24  
**Feature folder:** `docs/design/sidebar-explorer/`  
**Current source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`

## 1. Mục tiêu

Báo cáo này so sánh Explorer/Database Navigator hiện tại với các capability được mô tả trong tài liệu chính thức của DBeaver, JetBrains DataGrip và pgAdmin. Mục tiêu là xác định:

- feature hiện tại đã có;
- feature còn thiếu hoặc mới chỉ có một phần;
- feature nào nên nâng cấp trước;
- action cụ thể cần bổ sung vào sidebar.

Đây là research/design report, chưa phải quyết định triển khai và chưa phải runtime verification.

## 2. Nguồn chính thức đã tham khảo

### DBeaver

- [Database Navigator](https://dbeaver.com/docs/dbeaver/Database-Navigator/)
- [Filtering database objects](https://dbeaver.com/docs/dbeaver/Filtering-database-objects/)
- [ER Diagrams](https://dbeaver.com/docs/dbeaver/ER-Diagrams/)
- [Data Editor](https://dbeaver.com/docs/dbeaver/Data-Editor/)
- [Compare/Migrate](https://dbeaver.com/docs/dbeaver/Compare-Migrate/)

### JetBrains DataGrip

- [Database Explorer](https://www.jetbrains.com/help/datagrip/database-explorer.html)
- [Viewing reference information](https://www.jetbrains.com/help/datagrip/viewing-reference-information.html)
- [Schema comparison](https://www.jetbrains.com/help/datagrip/schema-comparison.html)

### pgAdmin

- [Object browser](https://www.pgadmin.org/docs/pgadmin4/latest/browser.html)
- [ERD tool](https://www.pgadmin.org/docs/pgadmin4/latest/erd_tool.html)
- [Query tool](https://www.pgadmin.org/docs/pgadmin4/latest/query_tool.html)

## 3. Hiện trạng đã có

Theo source baseline ở commit `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`, Explorer đã có:

- connection catalog và connect/disconnect/reconnect;
- database → schema → tables hierarchy;
- filter tables;
- refresh schema;
- table data/structure/DDL entry points;
- SQL generation cho SELECT/INSERT/UPDATE/DELETE;
- table columns, foreign keys, indexes;
- views, functions/procedures và triggers folders;
- open/modify/drop/copy action cho một phần schema object;
- ER diagram entry ở connection level;
- Agent entry ở connection/table level;
- Schema Workbench entry cho modify/drop;
- delete confirmation cho connection/object flows;
- guard staged changes và open transaction;
- capability gate cho functions;
- offscreen rendering và navigation cache cho table list.

## 4. Các gap chính cần nâng cấp

### F1 — Object filter chưa đủ sâu

**Hiện tại:**

- Có một ô `Filter objects…`.
- Logic hiện tại chủ yếu lọc table trong schema active.
- Views/functions/triggers vẫn được render theo schema, chưa có filter model thống nhất cho từng loại object.

**Capability tham khảo:**

DBeaver có tài liệu riêng cho filtering database objects và mô hình lọc object trong navigator. DataGrip Database Explorer cũng có object filter theo data source/schema/object scope.

**Nên bổ sung:**

- Filter theo loại object: Tables, Views, Functions, Triggers, Sequences, Types, Extensions.
- Filter theo tên, schema, owner, comment, system/user object.
- Filter mode rõ ràng: contains, starts with, exact, regex.
- Hiển thị số kết quả theo từng folder.
- Nút clear filter và trạng thái filter đang áp dụng.
- Persist filter theo connection/schema.
- Không tự động bỏ object folder chỉ vì không match; hiển thị count `0` và lý do.

**Action đề xuất:**

```text
SetExplorerFilter { query, scope, mode }
ClearExplorerFilter
SetObjectTypeVisibility { kind, visible }
OpenObjectFilterDialog
ResetObjectFilters
```

**Ưu tiên:** P1 — tác động trực tiếp đến khả năng làm việc với database lớn.

---

### F2 — Tổ chức connection và object còn phẳng

**Hiện tại:**

- Connection được render theo catalog.
- Chưa thấy UI folder/group rõ ràng trong Explorer baseline.
- Đã có dữ liệu favorite/group trong model connection ở các phần khác, nhưng chưa được thể hiện đầy đủ trong cây Explorer.

**Capability tham khảo:**

DBeaver Database Navigator và DataGrip Database Explorer đều hỗ trợ tổ chức data source/object theo project, folder, group hoặc màu sắc để phân biệt môi trường.

**Nên bổ sung:**

- Connection folders: Development, Staging, Production, Local.
- User-defined group/folder.
- Favorite/pinned connections.
- Favorite/pinned tables và views.
- Recent objects.
- Environment badge và màu connection.
- Collapse/expand folder độc lập.
- Drag-and-drop connection vào folder nếu UX native cho phép.

**Action đề xuất:**

```text
CreateConnectionFolder
RenameConnectionFolder
MoveConnectionToFolder
ToggleConnectionFavorite
ToggleObjectPin
OpenRecentObjects
```

**Ưu tiên:** P1 — đặc biệt quan trọng khi có nhiều database/tenant/environment.

---

### F3 — Thiếu quick documentation/metadata preview

**Hiện tại:**

- Table có thể mở detail columns/FK/index khi đã select và có metadata.
- View/function/trigger chủ yếu chỉ hiển thị tên, event hoặc routine type.
- Chưa có hover/side preview thống nhất cho object metadata.

**Capability tham khảo:**

DataGrip có cơ chế viewing reference information/quick documentation. DBeaver cũng cung cấp metadata/documentation thông qua navigator và object editor.

**Nên bổ sung:**

- Hover hoặc keyboard shortcut mở quick documentation.
- Preview cho table:
  - columns, data types, nullability, PK/FK;
  - indexes;
  - row estimate nếu provider hỗ trợ;
  - comment/description;
  - created/modified metadata nếu provider hỗ trợ.
- Preview cho view:
  - definition SQL;
  - columns;
  - dependencies.
- Preview cho function/procedure:
  - signature;
  - return type;
  - definition/source nếu capability cho phép.
- Preview cho trigger:
  - event;
  - timing;
  - target table;
  - definition.

**Action đề xuất:**

```text
ShowQuickDocumentation { object }
OpenObjectMetadata
CopyObjectDefinition
OpenObjectDependencies
```

**Ưu tiên:** P1 — giảm số lần phải rời Explorer để inspect object.

---

### F4 — Schema object coverage còn thiếu

**Hiện tại:**

Explorer chỉ có các folder chính:

```text
Tables / Views / Functions / Triggers
```

**Capability tham khảo:**

DBeaver và pgAdmin object browser thể hiện nhiều nhóm object hơn, tùy provider. Với PostgreSQL, object browser thường bao gồm sequences, types, extensions, schemas, catalogs, foreign tables, materialized views, policies, publications/subscriptions và nhiều nhóm khác.

**Nên bổ sung theo capability provider:**

#### PostgreSQL

- Sequences.
- Materialized Views.
- Types, enums, domains, composites.
- Extensions.
- Foreign Tables và Foreign Data Wrappers.
- Policies/RLS.
- Publications/subscriptions.
- Event triggers.
- Collations.
- Full-text search objects nếu scope cho phép.

#### SQLite

- Indexes như folder cấp schema hoặc table.
- Triggers.
- Views.
- SQLite pragmas/virtual tables nếu capability hỗ trợ.
- Không hiển thị PostgreSQL-only folders.

**Action đề xuất:**

```text
OpenObjectFolder { kind }
RefreshObjectFolder { kind }
CreateSchemaObject { kind }
OpenSchemaObjectDefinition { kind, name }
```

**Ưu tiên:** P1 cho PostgreSQL; P2 cho các object nâng cao chưa có nhu cầu rõ ràng.

**Điều kiện:** Mỗi object type phải capability-gated riêng; không emit SQL unsupported cho provider khác.

---

### F5 — Create action chưa cân đối với Modify/Drop

**Hiện tại:**

- Connection context menu có `Create Table (DDL)…`, thực chất mở một query template.
- Table có Modify/Drop.
- View có Modify/Drop.
- Trigger có Drop.
- Function/procedure chưa có modify/drop action.
- Chưa có create action trực tiếp theo folder/schema.

**Capability tham khảo:**

DBeaver navigator và pgAdmin object browser đều dùng context menu/object editor để tạo và chỉnh sửa nhiều loại database object.

**Nên bổ sung:**

- Context menu folder: `Create Table`, `Create View`, `Create Function`, `Create Trigger`.
- Context menu schema: `Create Schema Object…` với object type selector.
- Mở form/workbench typed theo object type, thay vì chỉ tạo query text.
- Preview DDL trước khi apply.
- Dependency warning trước Drop.
- Explicit distinction giữa:
  - generate SQL;
  - plan mutation;
  - execute mutation.

**Action đề xuất:**

```text
CreateTable
CreateView
CreateFunction
CreateTrigger
OpenCreateObjectWorkbench
PreviewObjectDdl
PlanDropObject
ShowDropDependencies
```

**Ưu tiên:** P1 cho table/view; P2 cho function/trigger nếu provider support chưa đầy đủ.

---

### F6 — Compare/Migrate chưa gắn với Explorer selection

**Hiện tại:**

- Có Schema Compare activity riêng.
- Explorer connection menu có ER Diagram nhưng chưa có compare action theo selected connection/schema/table.
- Chưa thấy flow chọn source/target trực tiếp từ Explorer.

**Capability tham khảo:**

DBeaver có Compare/Migrate workflow. DataGrip có schema comparison để so sánh database objects/schema và tạo diff/script.

**Nên bổ sung:**

- Compare two connections.
- Compare two schemas.
- Compare selected table/view/function.
- Chọn source và target từ context menu.
- Preview diff trước khi generate/apply.
- Generate migration SQL/script.
- Chỉ apply khi user xác nhận rõ source/target.
- Hiển thị provider/capability mismatch.

**Action đề xuất:**

```text
SelectAsCompareSource
SelectAsCompareTarget
CompareConnection
CompareSchema
CompareObject
GenerateMigrationScript
OpenSchemaCompare
```

**Ưu tiên:** P1 — đây là hướng compare feature được yêu cầu tiếp theo.

**Safety:** Không được mặc định apply migration; chỉ generate/preview trước, apply phải qua confirmation policy.

---

### F7 — ER Diagram chỉ mở ở connection level

**Hiện tại:**

- Connection context menu có `View ER Diagram`.
- Chưa có action rõ ràng trên schema/table để mở diagram giới hạn trong selection.

**Capability tham khảo:**

DBeaver ER Diagrams và pgAdmin ERD đều hỗ trợ làm việc với quan hệ giữa bảng/object trong phạm vi được chọn hoặc schema.

**Nên bổ sung:**

- `Open ER Diagram` trên schema.
- `Open ER Diagram` trên selected table.
- `Add table to diagram` từ context menu.
- `Show related tables` dựa trên FK.
- Save diagram scope/layout.
- Export diagram.

**Action đề xuất:**

```text
OpenSchemaDiagram
OpenTableDiagram
AddToDiagram
ShowRelatedTables
SaveDiagram
ExportDiagram
```

**Ưu tiên:** P1 cho selected-table diagram; P2 cho layout/export nâng cao.

---

### F8 — Refresh/loading/error state chưa granular

**Hiện tại:**

- Có refresh schema ở connection/toolbar/context menu.
- Có loading/error feedback cho schema tree.
- Refresh chủ yếu là request theo connection, chưa phải refresh độc lập từng folder/object type.

**Capability nâng cấp:**

- Refresh connection.
- Refresh schema.
- Refresh folder.
- Refresh selected object metadata.
- Loading state theo node/folder thay vì chỉ một schema loading flag.
- Cancel introspection.
- Retry riêng node failed.
- Hiển thị timestamp của metadata snapshot.
- Phân biệt stale cache với request đang chạy.

**Action đề xuất:**

```text
RefreshConnection
RefreshSchema
RefreshObjectFolder
RefreshObjectDetails
CancelIntrospection
RetryIntrospection
```

**Ưu tiên:** P1 — cần thiết để Explorer đáng tin cậy khi schema lớn hoặc provider chậm.

---

### F9 — Missing dependencies/usage navigation

**Hiện tại:**

- Foreign keys được hiển thị trong table details.
- Chưa có navigation rõ ràng tới objects tham chiếu và objects phụ thuộc.
- Chưa có action tìm nơi table/view/function được sử dụng.

**Capability tham khảo:**

Database IDEs thường expose references/dependencies từ object metadata và quick documentation.

**Nên bổ sung:**

- Go to referenced table từ FK row.
- Show incoming references.
- Show outgoing dependencies.
- Find usages cho table/view/function.
- Dependency graph trước Drop/Alter.
- Open dependent object trực tiếp.

**Action đề xuất:**

```text
OpenReferencedObject
ShowIncomingReferences
ShowOutgoingDependencies
FindUsages
OpenDependencyGraph
```

**Ưu tiên:** P1 cho table/view; P2 cho routine dependency graph.

---

### F10 — Keyboard navigation và command discoverability

**Hiện tại:**

- Có một số shortcut: `F5`, `F4`, `Del`, shortcut New Query/Command Palette.
- Chưa thấy keyboard navigation hoàn chỉnh cho toàn bộ tree.

**Nên bổ sung:**

- Arrow keys để expand/collapse và di chuyển node.
- Enter mở object.
- Space toggle folder.
- Context-menu shortcut.
- `F2` rename/edit khi object hỗ trợ.
- `Ctrl/Cmd+F` focus filter.
- `Escape` clear/close filter.
- Shortcut help hiển thị theo context.
- Focus restoration sau refresh hoặc mở workspace.

**Ưu tiên:** P2, nhưng nên thiết kế cùng với mọi action mới.

---

### F11 — Persisted navigation state chưa đầy đủ

**Hiện tại:**

- Một số collapsing state dùng persistent egui id.
- Sidebar width được lưu.
- Chưa thấy đầy đủ persisted state cho filter, expanded path, favorite/pin, recent object và selected compare source/target.

**Nên bổ sung:**

- Persist expanded connection/database/schema/folder path.
- Persist filter theo connection/schema.
- Persist selected object và last workspace destination.
- Persist pinned/favorite/recent objects.
- Persist compare source/target trong session, không nhất thiết lâu dài.
- Reset state khi connection id/provider/database thay đổi.

**Ưu tiên:** P2.

## 5. Đề xuất roadmap theo feature

### Phase 1 — Explorer usability baseline

1. F1 — object filter sâu hơn.
2. F2 — folders/favorites/recent objects.
3. F3 — quick documentation.
4. F8 — granular refresh/loading/error.
5. F10 — keyboard navigation cơ bản.

### Phase 2 — Compare-oriented Explorer

1. F6 — compare source/target từ Explorer.
2. F5 — typed create/modify/drop workbench.
3. F9 — dependency/reference navigation.
4. F7 — schema/table scoped ER diagram.

### Phase 3 — Provider depth

1. F4 — PostgreSQL object folders.
2. F4 — SQLite-specific objects/capabilities.
3. Provider-specific DDL preview and mutation policy.
4. Runtime matrix độc lập PostgreSQL/SQLite.

## 6. Khuyến nghị scope cho compare feature trước mắt

Nếu mục tiêu kế tiếp là **compare feature**, không nên bắt đầu bằng việc mở rộng toàn bộ object browser. Scope nhỏ và có giá trị nhất:

```text
Explorer
  → chọn connection/schema/table
  → Select as compare source
  → chọn connection/schema/table khác
  → Select as compare target
  → Open Schema Compare
  → preview diff
  → generate migration script
  → user-confirmed apply (future scope)
```

### Actions tối thiểu cần có

```text
SelectAsCompareSource
SelectAsCompareTarget
CompareConnection
CompareSchema
CompareTable
GenerateMigrationScript
OpenSchemaCompare
```

### Acceptance criteria cho compare UX

- Source và target luôn hiển thị rõ tên connection/database/schema/object.
- Không cho source và target trùng nhau nếu operation không có ý nghĩa.
- Không tự động apply mutation sau compare.
- Diff phải phân biệt add/change/remove.
- Có preview SQL/script trước khi copy hoặc apply.
- Có capability reason nếu PostgreSQL/SQLite không hỗ trợ operation.
- Connection/schema context không bị mất khi quay lại Explorer.

## 7. Các điểm cần tránh khi nâng cấp

- Không thêm folder PostgreSQL vào SQLite bằng fallback giả.
- Không emit SQL unsupported chỉ vì action đã xuất hiện trong context menu.
- Không gộp `Generate SQL`, `Plan mutation`, và `Execute mutation` thành một action.
- Không dùng một filter string cho tất cả object nếu semantics khác nhau.
- Không để Drop action bypass confirmation/dependency check.
- Không coi source inspection là runtime evidence.
- Không coi capability của DBeaver/DataGrip là bằng chứng provider runtime của DB Pro.

## 8. Kết luận

Explorer hiện đã là một database navigator có nền tảng tốt: connection lifecycle, schema tree, table actions, schema objects và workspace integration đã tồn tại. Các thiếu hụt lớn nhất không nằm ở việc thêm thật nhiều folder, mà nằm ở:

1. filter và tổ chức object ở quy mô lớn;
2. quick metadata/dependency navigation;
3. create/modify/drop workflow nhất quán;
4. compare source/target flow trực tiếp từ Explorer;
5. capability-gated provider coverage;
6. granular refresh và state persistence.

Đối với compare feature, nên triển khai trước action pipeline source/target → diff → script preview, sau đó mới mở rộng object coverage.

## 9. Evidence boundary

Các claim về DB Pro trong phần hiện trạng dựa trên source commit `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. Các claim so sánh dựa trên tài liệu chính thức được dẫn link ở mục 2. Chưa có UI runtime evidence hoặc provider runtime evidence trong report này.
