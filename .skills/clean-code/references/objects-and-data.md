# Objects & Data Structures — Đối tượng và cấu trúc dữ liệu

## 1. Object vs Data Structure — biết mình đang viết cái gì

| | Object | Data Structure |
|---|---|---|
| Phơi bày | **Hành vi** (method) | **Dữ liệu** (field) |
| Ẩn | Dữ liệu / cách lưu trữ | Không có gì để ẩn |
| Dễ | Thêm **kiểu mới** (thêm impl) | Thêm **hành vi mới** (thêm hàm nhận struct) |
| Khó | Thêm hành vi mới (sửa mọi impl) | Thêm kiểu mới (sửa mọi hàm `match`) |
| Ví dụ trong repo | `Connector`, `SchemaIntrospector` (trait + impl theo provider) | `TableMetadata`, `*Dto`, `ConnectionConfig` |

**Hybrid** (nửa object nửa struct: field public + method nghiệp vụ) là tệ nhất của cả hai — tránh.

```rust
// ✗ Sai — hybrid: field pub để ai cũng sửa, nhưng lại có invariant qua method
pub struct QueryTab {
    pub sql: String,
    pub is_dirty: bool,
}
impl QueryTab {
    pub fn set_sql(&mut self, sql: String) { self.sql = sql; self.is_dirty = true; }
}
// ai đó viết tab.sql = "..." → is_dirty không cập nhật.

// ✓ Đúng — chọn một: Object (ẩn field)
pub struct QueryTab { sql: String, is_dirty: bool }
impl QueryTab {
    pub fn sql(&self) -> &str { &self.sql }
    pub fn is_dirty(&self) -> bool { self.is_dirty }
    pub fn set_sql(&mut self, sql: String) { self.sql = sql; self.is_dirty = true; }
}
```

DTO ở ranh giới IPC (`crates/tauri-app/src/dto.rs`, kiểu trong `frontend/src/commons/types`) là **data structure thuần**:
`pub` field, `Serialize/Deserialize`, không logic. Chuyển đổi sang domain qua `From`/`TryFrom` một chỗ.

## 2. Đóng gói (Encapsulation / Data Hiding)

Không phải "field private + getter/setter cho mọi field". Đó chỉ là data structure trá hình.
Đóng gói = phơi bày **thao tác trừu tượng**, giấu **cách biểu diễn**.

```ts
// ✗ Sai — getter/setter lộ biểu diễn
class FuelTank {
  getGallons(): number
  setGallons(g: number): void
}

// ✓ Đúng — trừu tượng, có thể đổi biểu diễn sang lít mà không đổi API
class FuelTank {
  percentFull(): number
  refill(amount: Fuel): void
}
```

### TypeScript

- `readonly` cho field/prop không đổi; `Readonly<T>` / `ReadonlyArray<T>` cho dữ liệu chia sẻ.
- Không export mutable module-level state (`export let cache = {}`); export hàm truy cập.
- Zustand store: expose **action có tên nghiệp vụ** (`closeTab(id)`, `markSaved(id)`), không `setState` tự do từ component.

```ts
// ✗ Sai
useWorkspaceStore.setState((s) => ({ tabs: s.tabs.map((t) => (t.id === id ? { ...t, isDirty: false } : t)) }));

// ✓ Đúng — invariant tập trung trong store
useWorkspaceStore.getState().markSaved(id);
```

### Rust

- Field private mặc định; `pub(crate)` khi chỉ nội bộ crate cần.
- Constructor trả `Result` khi có invariant cần validate (`ConnectionConfig::try_new`).
- Newtype cho giá trị có ràng buộc: `struct SchemaName(String)` với `TryFrom<&str>` validate → không thể tạo tên sai.

```rust
// ✓ Đúng — không thể có QualifiedIdentifier chưa validate
pub struct QualifiedIdentifier { schema: SchemaName, table: TableName }
impl QualifiedIdentifier {
    pub fn parse(schema: &str, table: &str) -> Result<Self, DbError> { ... }
    pub fn quoted_for(&self, provider: Provider) -> String { ... }
}
```

## 3. Law of Demeter — "chỉ nói chuyện với bạn thân"

Method `m` của object `O` chỉ nên gọi method của:

1. chính `O`
2. tham số của `m`
3. object do `m` tạo ra
4. field trực tiếp của `O`

**Không** gọi method trên object trả về từ một method khác (train wreck).

```ts
// ✗ Sai — biết quá sâu: connection → profile → ssh → tunnel
const port = connection.getProfile().getSsh().getTunnel().getLocalPort();

// ✓ Đúng — hỏi đúng thứ cần
const port = connection.localPort();
```

```ts
// ✗ Sai — component biết cấu trúc store + query + row
const value = queryResult.data?.rows[rowIndex]?.cells[columnIndex]?.value ?? null;

// ✓ Đúng — selector / accessor
const value = selectCellValue(queryResult, rowIndex, columnIndex);
```

**Ngoại lệ**: chuỗi truy cập trên **data structure thuần** (`dto.connection.host`) không vi phạm Demeter —
struct không có hành vi để "biết quá sâu". Vấn đề là chuỗi *method call* trên *object*.

**Không** giải quyết bằng cách thêm delegate method vô nghĩa cho mọi tầng (`connection.getSshTunnelLocalPort()`)
— hãy hỏi tại sao caller cần thứ đó và đẩy hành vi xuống nơi có dữ liệu ("Tell, Don't Ask").

```ts
// ✗ Ask
if (tab.isDirty() && !tab.isPinned()) { tab.markClosed(); store.remove(tab.id); }

// ✓ Tell
store.closeTab(tab.id); // store/tab tự biết điều kiện
```

## 4. Ưu tiên bất biến (Immutability)

- TS: `const`, `readonly`, spread thay vì mutate, `as const` cho literal.
- Rust: mặc định bất biến; `mut` chỉ khi thực sự cần và trong phạm vi hẹp nhất.
- Zustand/TanStack: **không mutate** object trong state/cache; luôn tạo object mới để reference thay đổi.

```ts
// ✗ Sai — mutate mảng từ store → React không re-render, các subscriber khác thấy dữ liệu "ma"
const tabs = useWorkspaceStore.getState().tabs;
tabs.push(newTab);

// ✓ Đúng
useWorkspaceStore.setState((s) => ({ tabs: [...s.tabs, newTab] }));
```

## 5. Composition hơn kế thừa

- TS: không dùng `class extends` cho tái sử dụng logic UI; dùng hook + component composition.
- Rust: không có kế thừa; dùng trait + generic/`dyn Trait` + struct chứa struct.

```ts
// ✗ Sai
class PostgresGrid extends BaseGrid { ... }
class SqliteGrid extends BaseGrid { ... }

// ✓ Đúng
<DataGrid rows={rows} columns={columns} cellRenderer={providerCellRenderer(provider)} />
```

## 6. Không lách hệ thống kiểu

| Ngôn ngữ | Cấm (ngoài test / ranh giới không tránh được) | Thay bằng |
|----------|-----------------------------------------------|-----------|
| TS | `any` | `unknown` + narrowing, generic, union type |
| TS | `as unknown as X`, `as X` bừa bãi | type guard `isX(v): v is X`, `zod` parse ở ranh giới IPC |
| TS | `!` non-null assertion | `?.`, early return, hoặc chứng minh bằng type |
| TS | `@ts-ignore` | `@ts-expect-error` + comment lý do (và chỉ khi thật sự cần) |
| Rust | `unwrap()` / `expect()` trên đường dữ liệu | `?`, `ok_or_else`, `let-else`, `unwrap_or_default` có chủ ý |
| Rust | `as` cast số mất dữ liệu | `try_from`, `u64::from` |
| Rust | `unsafe` không có `// SAFETY:` | giải thích hoặc tìm cách an toàn |

`expect("...")` được chấp nhận khi **không thể** fail và message nói rõ vì sao (ví dụ regex literal compile-time,
lock poisoning ở init). Trong handler Tauri / provider adapter → luôn là `Result`.

## 7. Ranh giới (Boundaries) — bọc thư viện bên thứ ba

Không để kiểu của `sqlx`, `rusqlite`, `monaco`, `@xyflow` rò rỉ khắp codebase.

```text
frontend:  monaco-editor  →  modules/query/editor/monaco-adapter.ts  →  phần còn lại dùng QueryEditorApi
crates:    sqlx::Error    →  infrastructure/src/error.rs (from_sqlx)  →  core chỉ thấy DbError
```

Lợi ích: nâng version / đổi thư viện sửa một chỗ; unit test không cần thư viện thật.

## 8. Null / Option

- TS: tránh `null` và `undefined` lẫn lộn — chọn `undefined` cho "không có" (trừ khi API ngoài trả `null`), và nói rõ trong type.
- Không truyền `null` vào hàm; dùng overload / options object / default.
- Rust: `Option<T>` — đừng dùng `String::new()` / `-1` / `0` làm "không có".
- Trả về collection rỗng thay vì `None`/`null` khi ngữ nghĩa là "danh sách không có phần tử".

```rust
// ✗ Sai
fn find_primary_key(table: &TableMetadata) -> String   // "" nếu không có PK?

// ✓ Đúng
fn find_primary_key(table: &TableMetadata) -> Option<&ColumnMetadata>
```

## Checklist nhanh

- [ ] Mỗi type là **Object** (ẩn field) hoặc **Data Structure** (phơi field, không logic) — không hybrid
- [ ] DTO chỉ ở ranh giới IPC, chuyển sang domain ở một chỗ
- [ ] Không có train wreck `a.b().c().d()` trên object
- [ ] Không mutate state/cache dùng chung
- [ ] Không `any` / `unwrap()` / `!` trên đường dữ liệu
- [ ] Thư viện bên thứ ba được bọc sau adapter
- [ ] "Không có giá trị" biểu diễn bằng `Option` / `undefined` rõ ràng, không sentinel
