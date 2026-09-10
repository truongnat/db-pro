# Naming — Tiêu chuẩn đặt tên

Tên là hình thức tài liệu rẻ nhất và bền nhất. Một cái tên tốt loại bỏ nhu cầu comment.

## Quy ước theo ngôn ngữ

### TypeScript / React

| Loại | Quy ước | Ví dụ |
|------|---------|-------|
| Biến, hàm, method | `camelCase` | `activeConnectionId`, `fetchTableColumns()` |
| Type, interface, enum, class, component | `PascalCase` | `ConnectionProfile`, `SchemaTreeNode`, `QueryEditor` |
| Hằng số module-level | `UPPER_SNAKE_CASE` | `MAX_TAB_COUNT`, `DEFAULT_PAGE_SIZE` |
| Custom hook | `use` + PascalCase | `useConnectionStatus()` |
| Event handler prop | `on` + Event | `onRowSelect`, `onClose` |
| Handler implementation | `handle` + Event | `handleRowSelect` |
| Boolean | `is` / `has` / `can` / `should` | `isReadOnly`, `hasUnsavedChanges`, `canExecute` |
| Generic type param | 1 chữ hoa có nghĩa hoặc PascalCase | `TRow`, `TResult` (tránh `T` khi > 1 param) |
| File component | `kebab-case.tsx` (theo repo) | `schema-tree.tsx` |
| Interface | **không** prefix `I` | `Connection` chứ không `IConnection` |

### Rust

| Loại | Quy ước | Ví dụ |
|------|---------|-------|
| Hàm, biến, module, field | `snake_case` | `introspect_schema`, `connection_id` |
| Struct, enum, trait, type alias | `PascalCase` | `DbError`, `TableMetadata`, `Connector` |
| Enum variant | `PascalCase` | `DbError::ConnectionTimeout` |
| const / static | `SCREAMING_SNAKE_CASE` | `DEFAULT_POOL_SIZE` |
| Lifetime | ngắn, chữ thường | `'a`, `'conn` |
| Trait mô tả khả năng | tính từ/danh từ | `Connector`, `SchemaIntrospector`, `Cancellable` |
| Constructor | `new`, `from_*`, `with_*`, `try_*` | `Pool::with_capacity`, `DbError::from_sqlx` |
| Conversion | `as_*` (rẻ, borrow), `to_*` (tốn, clone), `into_*` (consume) | theo Rust API Guidelines |

## Nguyên tắc

### 1. Tên thể hiện ý định (Intention-revealing)

```ts
// ✗ Sai
const d = Date.now() - t; // elapsed?
const list = rows.filter((r) => r[3] === 1);

// ✓ Đúng
const elapsedMs = Date.now() - startedAt;
const activeConnections = connections.filter((connection) => connection.isActive);
```

```rust
// ✗ Sai
fn proc(c: &Conn, s: &str) -> Result<Vec<T>, E>

// ✓ Đúng
fn list_tables_in_schema(connection: &Connection, schema_name: &str) -> Result<Vec<TableMetadata>, DbError>
```

### 2. Đọc thành tiếng được (Pronounceable)

```ts
// ✗ Sai
const genymdhms = createTimestamp();
const cnnMgr = new ConnectionManager();

// ✓ Đúng
const generationTimestamp = createTimestamp();
const connectionManager = new ConnectionManager();
```

### 3. Tìm kiếm được — không dùng magic number / string

```ts
// ✗ Sai
if (tabs.length > 50) closeOldest();
setTimeout(reconnect, 5000);
if (status === "rdy") { ... }

// ✓ Đúng
const MAX_OPEN_TABS = 50;
const RECONNECT_DELAY_MS = 5_000;
if (tabs.length > MAX_OPEN_TABS) closeOldest();
setTimeout(reconnect, RECONNECT_DELAY_MS);
if (status === ConnectionStatus.Ready) { ... }
```

```rust
// ✗ Sai
if code_str == "23505" { ... }

// ✓ Đúng
const SQLSTATE_UNIQUE_VIOLATION: &str = "23505";
if code_str == SQLSTATE_UNIQUE_VIOLATION { ... }
// hoặc tốt hơn: match thành enum ConstraintType (xem crates/infrastructure/src/error.rs)
```

Quy tắc ngón tay cái: hằng số xuất hiện **≥ 2 lần** hoặc **có ý nghĩa nghiệp vụ** → phải có tên.
Số `0`, `1`, `-1` trong ngữ cảnh index/loop hiển nhiên thì được phép.

### 4. Không lặp ngữ cảnh (Don't add gratuitous context)

```ts
// ✗ Sai
interface Connection {
  connectionId: string;
  connectionHost: string;
  connectionPort: number;
}
connection.connectionHost;

// ✓ Đúng
interface Connection {
  id: string;
  host: string;
  port: number;
}
connection.host;
```

```rust
// ✗ Sai
struct TableMetadata { table_name: String, table_schema: String }

// ✓ Đúng
struct TableMetadata { name: String, schema: String }
```

Ngoại lệ: khi tên field trần gây mơ hồ ở nơi *sử dụng* (ví dụ hàm nhận cả `schema.name` và `table.name`),
hãy đặt tên **biến cục bộ** rõ ràng (`schema_name`, `table_name`), không đổi tên field.

### 5. Một khái niệm — một từ (Pick one word per concept)

Trong toàn codebase, chọn một từ và dùng nhất quán:

| Khái niệm | Dùng | Không dùng lẫn lộn |
|-----------|------|--------------------|
| Lấy dữ liệu từ backend | `fetch*` | `get*`, `retrieve*`, `load*` |
| Lấy giá trị đã có trong bộ nhớ | `get*` | `fetch*` |
| Xoá | `remove*` (khỏi collection) / `delete*` (khỏi persistent store) | `destroy*`, `kill*` |
| Tạo | `create*` | `make*`, `build*` (trừ builder pattern) |
| Chuyển đổi | `to*` / `from*` | `convert*` |

Kiểm tra: `grep -rn "function \(get\|fetch\|load\|retrieve\)" frontend/src` — nếu thấy cùng một thứ có 3 tên khác nhau, đó là smell.

### 6. Không chơi chữ, không mã hoá kiểu (No encodings)

```ts
// ✗ Sai — Hungarian notation, prefix type
const strName = "";
const arrRows = [];
const iCount = 0;
interface IConnection {}

// ✓ Đúng — kiểu đã có TypeScript lo
const name = "";
const rows: Row[] = [];
const count = 0;
interface Connection {}
```

### 7. Tên hàm là động từ, tên class/type là danh từ

```ts
// ✗ Sai
function connection(id: string) { ... }        // danh từ — làm gì với connection?
class ExecuteQuery { ... }                     // động từ — là hành động hay vật?

// ✓ Đúng
function openConnection(id: string) { ... }
class QueryExecutor { ... }
```

### 8. Độ dài tên tỉ lệ với phạm vi (Scope-length rule)

- Biến sống 2 dòng trong closure: `i`, `row`, `acc` chấp nhận được.
- Biến ở module-level / export: tên đầy đủ, mô tả.
- Hàm private dùng một chỗ: tên dài, cụ thể được phép (`buildQualifiedIdentifierForPostgres`).
- Hàm public dùng nhiều nơi: ngắn, tổng quát (`quoteIdentifier`).

### 9. Tên test mô tả hành vi

```ts
// ✗ Sai
it("works", ...)
it("test1", ...)

// ✓ Đúng
it("returns empty list when schema has no tables", ...)
it("rejects identifier containing double quote", ...)
```

```rust
// ✓ Đúng
#[test]
fn from_sqlx_maps_28p01_to_auth_failed() { ... }
```

## Red flags khi review

- Tên chứa `data`, `info`, `manager`, `helper`, `util`, `misc`, `temp`, `stuff` mà không có danh từ cụ thể đi kèm.
- Tên có số thứ tự: `handler2`, `newParser`, `oldConfig`.
- Tên phủ định kép: `isNotDisabled` → `isEnabled`.
- Tên nói dối: `getUser()` nhưng thực chất tạo user nếu chưa có.
- Cùng một biến bị gán lại với ý nghĩa khác giữa chừng hàm.
