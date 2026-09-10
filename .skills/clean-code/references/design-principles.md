# Design Principles — SOLID, DRY, KISS, YAGNI

Các nguyên tắc này **không** là luật để tuân thủ máy móc — chúng là công cụ để trả lời câu hỏi:
*"Nếu yêu cầu thay đổi theo hướng X, tôi phải sửa bao nhiêu chỗ?"*

## S.O.L.I.D

### S — Single Responsibility Principle

> Một module chỉ nên có **một lý do để thay đổi** (một "actor" yêu cầu thay đổi).

Không phải "làm một việc" (đó là hàm) mà là "phục vụ một bên liên quan".

```ts
// ✗ Sai — ConnectionService thay đổi khi: format lưu trữ đổi, UI đổi, provider đổi
class ConnectionService {
  loadFromDisk(): Connection[]                // persistence
  toDisplayLabel(c: Connection): string       // presentation
  buildPostgresDsn(c: Connection): string     // provider
}

// ✓ Đúng — mỗi lý do thay đổi một module
connection-repository.ts     loadConnections(), saveConnections()
connection-labels.ts         toDisplayLabel()
providers/postgres/dsn.ts    buildDsn()
```

Trong repo, phân tầng `crates/core` (domain/application/ports) ↔ `crates/infrastructure` (adapters) ↔ `crates/tauri-app` (IPC)
chính là SRP ở cấp crate: thay đổi driver DB không đụng domain; thay đổi IPC không đụng adapter.

Dấu hiệu vi phạm: file/class có tên chung chung (`Manager`, `Service`, `Utils`) và import từ mọi tầng.

### O — Open/Closed Principle

> Mở để **mở rộng**, đóng để **sửa đổi**: thêm hành vi mới bằng cách thêm code, không sửa code cũ đã test.

```rust
// ✗ Sai — thêm MySQL phải sửa hàm này (và 20 hàm match tương tự)
fn quote_identifier(provider: Provider, name: &str) -> String {
    match provider {
        Provider::Postgres => format!("\"{}\"", name.replace('"', "\"\"")),
        Provider::Sqlite => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

// ✓ Đúng — trait; thêm MySQL = thêm một impl mới, không đụng code cũ
pub trait Dialect {
    fn quote_identifier(&self, name: &str) -> String;
    fn capabilities(&self) -> Capabilities;
}
pub struct PostgresDialect;
impl Dialect for PostgresDialect { ... }
```

Trong frontend: registry pattern (`Record<Provider, Handler>`, map `exporters[format]`) thay cho `switch` rải rác.

**Cảnh báo YAGNI**: chỉ áp dụng OCP cho trục thay đổi **đã biết** (provider, export format, tab kind).
Không tạo abstraction cho trục thay đổi tưởng tượng.

### L — Liskov Substitution Principle

> Nơi nào dùng `Base`, thay bằng `Derived` phải **không làm hỏng** kỳ vọng.

Với trait/interface: mọi impl phải giữ **contract** (precondition không chặt hơn, postcondition không lỏng hơn, không ném lỗi bất ngờ).

```rust
// ✗ Sai — impl SQLite "hỗ trợ" trait nhưng panic / trả lỗi generic cho method không có
impl SchemaIntrospector for SqliteIntrospector {
    async fn list_triggers(&self, schema: &str) -> Result<Vec<Trigger>, DbError> {
        unimplemented!()   // caller không biết trước, crash runtime
    }
}

// ✓ Đúng — capability-gated (đúng luật AGENTS.md): caller hỏi trước, hoặc lỗi có lý do rõ
impl SchemaIntrospector for SqliteIntrospector {
    fn capabilities(&self) -> Capabilities { Capabilities { triggers: Supported, schemas: Unsupported("SQLite has no schemas") } }
    async fn list_triggers(&self, _schema: &str) -> Result<Vec<Trigger>, DbError> { ... }
}
```

Frontend: một component nhận `ExportHandler` không được phải `if (handler instanceof CsvHandler)` để hoạt động đúng.

### I — Interface Segregation Principle

> Client không nên bị buộc phụ thuộc vào method nó không dùng.

```rust
// ✗ Sai — trait "fat": để test QueryRunner phải mock cả backup, user management, ssh
pub trait Connector {
    async fn execute(&self, sql: &str) -> Result<QueryResult, DbError>;
    async fn introspect(&self) -> Result<Schema, DbError>;
    async fn backup(&self, opts: &BackupOptions) -> Result<(), DbError>;
    async fn list_users(&self) -> Result<Vec<User>, DbError>;
    async fn open_ssh_tunnel(&self) -> Result<Tunnel, DbError>;
}

// ✓ Đúng — trait nhỏ theo vai trò; struct có thể impl nhiều trait
pub trait QueryExecutor { async fn execute(...) }
pub trait SchemaIntrospector { async fn introspect(...) }
pub trait BackupProvider { async fn backup(...) }
```

Frontend: props interface của component chỉ chứa thứ component dùng. Không truyền cả object `connection` khi
chỉ cần `connection.name` và `connection.isReadOnly` — truyền hai giá trị đó (cũng giúp `memo` hiệu quả).

### D — Dependency Inversion Principle

> Module cấp cao không phụ thuộc module cấp thấp; cả hai phụ thuộc vào **abstraction**.
> Abstraction không phụ thuộc chi tiết; chi tiết phụ thuộc abstraction.

Đây là xương sống kiến trúc ports/adapters của repo:

```text
crates/core/src/ports/        ← trait (abstraction) do CORE sở hữu
crates/core/src/application/  ← use-case phụ thuộc vào trait
crates/infrastructure/        ← impl trait bằng sqlx/rusqlite (chi tiết)
crates/tauri-app/             ← composition root: nối impl vào use-case
```

Luật: `crates/core` **không được** `use sqlx::*` / `use rusqlite::*`. Kiểm tra: `grep -rn "sqlx\|rusqlite" crates/core/src` phải rỗng.

Frontend: `frontend/src/commons/di` là composition root. Module/hook nhận service qua DI/context, không `import { invoke } from "@tauri-apps/api"`
trực tiếp trong component → test được bằng fake service.

```ts
// ✗ Sai — component phụ thuộc chi tiết Tauri
function SchemaTree() {
  const { data } = useQuery({ queryFn: () => invoke("list_tables", { connectionId }) });
}

// ✓ Đúng — phụ thuộc abstraction
function SchemaTree() {
  const schemaService = useSchemaService();       // từ DI
  const { data } = useQuery({ queryFn: () => schemaService.listTables(connectionId) });
}
```

## D.R.Y — Don't Repeat Yourself

> Mỗi mẩu **kiến thức** phải có một biểu diễn duy nhất, rõ ràng, có thẩm quyền trong hệ thống.

DRY là về **kiến thức**, không phải **text**. Hai đoạn code giống hệt nhau nhưng thay đổi vì lý do khác nhau
**không** phải duplication — gom chúng lại tạo coupling sai (accidental duplication).

```ts
// Tình cờ giống nhau — KHÔNG gom
function validateConnectionName(s: string) { return s.length > 0 && s.length <= 64; }
function validateTabTitle(s: string)       { return s.length > 0 && s.length <= 64; }
// Ngày mai tab title cho phép 128 ký tự; connection name thì không.

// Thật sự trùng kiến thức — PHẢI gom
// "Cách quote identifier PostgreSQL" xuất hiện ở export.ts, ddl.ts, table-data.ts → một hàm duy nhất.
```

Các dạng duplication cần săn:

| Dạng | Ví dụ | Cách xử lý |
|------|-------|------------|
| Copy-paste | cùng 10 dòng ở 3 file | trích hàm/hook |
| Cấu trúc | `switch(provider)` lặp ở nhiều nơi | polymorphism / registry |
| Kiến thức | hằng số `MAX_ROWS` hardcode ở FE và BE khác giá trị | single source (config / generated type) |
| Type | interface FE tự viết tay trùng struct Rust DTO | sinh type từ Rust (ts-rs/specta) hoặc zod schema chung |
| Comment ↔ code | comment mô tả lại logic | xoá comment |
| Test ↔ prod | logic tính toán copy vào test để so sánh | test bằng giá trị cụ thể, không copy thuật toán |

Ngưỡng thực dụng: **Rule of Three** — lần thứ nhất viết, lần thứ hai chịu đựng (ghi chú), lần thứ ba refactor.

## K.I.S.S — Keep It Simple, Stupid

> Giải pháp đơn giản nhất **mà vẫn đúng**.

- Ưu tiên hàm thuần + data thay vì class hierarchy.
- Ưu tiên `if/else` rõ ràng thay vì generic 3 tầng cho 2 trường hợp.
- Ưu tiên thư viện chuẩn / đã có trong repo thay vì thêm dependency.
- Không "clever": bit trick, ternary lồng, regex 200 ký tự, `reduce` trả về object 6 field.

```ts
// ✗ "Clever"
const grouped = rows.reduce((acc, r) => ((acc[r.schema] ??= []).push(r), acc), {} as Record<string, Row[]>);

// ✓ Đơn giản
const grouped = Object.groupBy(rows, (row) => row.schema);
// hoặc vòng for rõ ràng nếu target chưa hỗ trợ
```

```rust
// ✗ Generic cho một use case
pub struct Cache<K, V, H: BuildHasher, E: EvictionPolicy<K>, C: Clock> { ... }

// ✓ Đủ dùng
pub struct SchemaCache { entries: HashMap<ConnectionId, CachedSchema>, ttl: Duration }
```

Câu hỏi kiểm tra: "Một dev mới đọc trong 30 giây có hiểu không?" Nếu không → đơn giản hoá hoặc đặt tên/tách hàm.

## Y.A.G.N.I — You Aren't Gonna Need It

> Không viết code cho nhu cầu **tưởng tượng**.

Dấu hiệu vi phạm:

- Tham số / option không ai truyền.
- Trait chỉ có **một** impl và không có test double.
- Config flag không có UI/CLI nào set.
- "Để sẵn cho MySQL" khi roadmap chưa có MySQL.
- Generic type param luôn được gọi với cùng một kiểu.
- Abstraction layer mà mọi caller đều "xuyên qua" (`getInner()`).

Cách kiểm tra nhanh: `npx knip` / `npx ts-prune` (unused exports), `cargo +nightly udeps` hoặc `cargo machete` (unused deps),
clippy `dead_code` warning.

**Ngoại lệ hợp lý**: ranh giới đã biết chắc sẽ thay đổi (provider abstraction — repo đã có 2 provider),
và các luật an toàn (capability gating, transaction) — đó không phải "tưởng tượng".

## Cân bằng giữa các nguyên tắc

| Xung đột | Cách quyết |
|----------|------------|
| OCP (thêm abstraction) vs YAGNI | Chỉ abstract khi có ≥ 2 biến thể **thật** hoặc cần test double |
| DRY vs KISS | Gom chỉ khi cùng *kiến thức*; hàm chung 8 tham số boolean là tệ hơn 2 bản copy |
| SRP (tách nhỏ) vs KISS (ít file) | Tách khi có ≥ 2 lý do thay đổi thật; đừng tách "cho đẹp" thành 10 file 5 dòng |
| DIP vs KISS | DIP tại ranh giới tầng (core/infra/IPC, service/component); bên trong một module, gọi thẳng |

## Checklist nhanh

- [ ] Mỗi module có một lý do thay đổi, tên nói rõ trách nhiệm
- [ ] Thêm provider / format / tab kind mới = thêm file, không sửa `switch` rải rác
- [ ] Mọi impl của trait/interface giữ đúng contract; không `unimplemented!()` runtime
- [ ] Interface/props chỉ chứa thứ client dùng
- [ ] `crates/core` không import driver; component không import `invoke` trực tiếp
- [ ] Duplication là *kiến thức* mới gom; tình cờ giống nhau thì để yên
- [ ] Không có tham số / trait / config / generic không ai dùng
- [ ] Dev mới đọc 30 giây hiểu được
