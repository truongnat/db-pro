# Functions — Thiết kế hàm

Hàm là đơn vị tổ chức nhỏ nhất. Hàm sạch = nhỏ, làm một việc, tên nói đúng việc đó.

## 1. Nhỏ (Small)

| Ngưỡng | Ý nghĩa |
|--------|---------|
| ≤ 20 dòng | Lý tưởng |
| 20–50 dòng | Chấp nhận nếu ở một mức trừu tượng, không lồng sâu |
| > 50 dòng | **P2** — phải có lý do rõ ràng, thường phải tách |
| > 100 dòng | **Chặn** — tách trước khi merge |

Không tính dòng trống/comment. Với React component, "hàm" bao gồm toàn bộ body trước `return`.

```tsx
// ✗ Sai — component 180 dòng: fetch, transform, sort, filter, render, xử lý keyboard
export function SchemaTree({ connectionId }: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState("");
  const { data } = useQuery(...);
  const nodes = useMemo(() => {
    // 40 dòng build tree
  }, [data]);
  const filtered = useMemo(() => {
    // 25 dòng filter
  }, [nodes, filter]);
  const handleKeyDown = (e: KeyboardEvent) => {
    // 30 dòng điều hướng bằng phím
  };
  return ( /* 60 dòng JSX */ );
}

// ✓ Đúng — mỗi trách nhiệm một hook / một hàm thuần
export function SchemaTree({ connectionId }: Props) {
  const nodes = useSchemaTreeNodes(connectionId);
  const { filter, setFilter, visibleNodes } = useSchemaTreeFilter(nodes);
  const { expanded, toggle } = useExpandedSet();
  const handleKeyDown = useTreeKeyboardNavigation(visibleNodes, expanded, toggle);

  return (
    <div role="tree" onKeyDown={handleKeyDown}>
      <SchemaTreeFilter value={filter} onChange={setFilter} />
      <SchemaTreeList nodes={visibleNodes} expanded={expanded} onToggle={toggle} />
    </div>
  );
}
```

`buildSchemaTreeNodes(data)` và `filterTreeNodes(nodes, filter)` trở thành hàm thuần → unit test được không cần DOM.

## 2. Làm một việc (Do One Thing)

Cách kiểm tra: mô tả hàm bằng một câu **không có chữ "và"**. Nếu có "và", tách.

Một hàm nên ở **một mức trừu tượng**: hoặc điều phối (gọi các hàm con), hoặc làm chi tiết — không trộn.

```rust
// ✗ Sai — trộn parsing, validation, IO, mapping lỗi
pub async fn export_table(req: ExportRequestDto) -> Result<String, CommandError> {
    let format = match req.format.as_str() {
        "csv" => ExportFormat::Csv,
        "json" => ExportFormat::Json,
        _ => return Err(CommandError::new("bad format")),
    };
    if req.table.is_empty() { return Err(CommandError::new("table required")); }
    let rows = fetch_rows(&req.connection_id, &req.table).await.map_err(CommandError::from)?;
    let mut out = String::new();
    for row in rows {
        // 30 dòng serialize theo format
    }
    std::fs::write(&req.path, &out).map_err(|e| CommandError::new(e.to_string()))?;
    Ok(req.path)
}

// ✓ Đúng — hàm điều phối chỉ gọi các bước đã đặt tên
pub async fn export_table(req: ExportRequestDto) -> Result<String, CommandError> {
    let options = ExportOptions::try_from(req)?;          // parse + validate
    let rows = fetch_rows(&options.connection_id, &options.table).await?;
    let payload = serialize_rows(&rows, options.format);  // thuần, test được
    write_export_file(&options.path, &payload)?;
    Ok(options.path)
}
```

### Step-down rule

Đọc file từ trên xuống như đọc báo: hàm public ở trên, hàm nó gọi ở ngay bên dưới, chi tiết nhất ở cuối.

## 3. Tham số (Arguments)

| Số tham số | Đánh giá |
|------------|----------|
| 0 (niladic) | Lý tưởng |
| 1 (monadic) | Tốt |
| 2 (dyadic) | Ổn, cẩn thận thứ tự (`assertEquals(expected, actual)`) |
| 3 (triadic) | Cần cân nhắc — gom object? |
| ≥ 4 | **P2** — gom thành options object / struct |

```ts
// ✗ Sai — 6 tham số, boolean flag, dễ nhầm thứ tự
function exportRows(rows, format, path, includeHeader, delimiter, quoteAll) { ... }
exportRows(rows, "csv", "/tmp/a.csv", true, ",", false);

// ✓ Đúng
interface CsvExportOptions {
  path: string;
  includeHeader?: boolean;
  delimiter?: "," | ";" | "\t";
  quoteAll?: boolean;
}
function exportRowsAsCsv(rows: Row[], options: CsvExportOptions) { ... }
exportRowsAsCsv(rows, { path: "/tmp/a.csv", includeHeader: true });
```

```rust
// ✗ Sai
fn connect(host: &str, port: u16, user: &str, password: &str, database: &str, ssl: bool, timeout_ms: u64) -> ...

// ✓ Đúng
pub struct ConnectionConfig { pub host: String, pub port: u16, /* ... */ }
fn connect(config: &ConnectionConfig) -> ...
// hoặc builder: ConnectionConfig::builder().host("..").port(5432).build()
```

### Không dùng boolean flag argument

Boolean param = hàm làm **hai việc**. Tách hoặc dùng enum.

```ts
// ✗ Sai
renderGrid(rows, true); // true là gì?

// ✓ Đúng
renderEditableGrid(rows);
renderReadOnlyGrid(rows);
// hoặc
renderGrid(rows, { mode: "readonly" });
```

### Không dùng output argument

```ts
// ✗ Sai — sửa tham số đầu vào
function appendFooter(report: Report) { report.lines.push(FOOTER); }

// ✓ Đúng — trả về giá trị mới hoặc là method của chính object
function withFooter(report: Report): Report { return { ...report, lines: [...report.lines, FOOTER] }; }
report.appendFooter();
```

## 4. Không side effect ẩn (No Side Effects)

Hàm chỉ làm điều tên nó nói. Side effect ẩn tạo **temporal coupling** (phải gọi đúng thứ tự mới chạy).

```ts
// ✗ Sai — checkPassword còn khởi tạo session (ẩn)
function checkPassword(user: string, password: string): boolean {
  const ok = verify(user, password);
  if (ok) Session.initialize(user); // ai ngờ?
  return ok;
}

// ✓ Đúng — tách, đặt tên đúng
function isPasswordValid(user: string, password: string): boolean { ... }
function startSession(user: string) { ... }
```

```ts
// ✗ Sai — hàm "tính" nhưng mutate store
function computeVisibleTabs(): Tab[] {
  const tabs = useWorkspaceStore.getState().tabs;
  useWorkspaceStore.setState({ lastComputedAt: Date.now() }); // ẩn
  return tabs.filter(isVisible);
}
```

Trong React: hàm gọi trong render **phải thuần**. Side effect vào `useEffect` / event handler.

## 5. Command–Query Separation (CQS)

- **Query**: trả về giá trị, không thay đổi state. Gọi nhiều lần cho cùng kết quả.
- **Command**: thay đổi state, trả về `void` / `Result<()>` (hoặc id vừa tạo).

```ts
// ✗ Sai — vừa set vừa trả bool
if (setAttribute("username", "bob")) { ... } // set thành công? hay đã tồn tại?

// ✓ Đúng
if (hasAttribute("username")) { setAttribute("username", "bob"); }
```

```rust
// ✗ Sai — "get" nhưng tạo nếu chưa có, còn tăng counter
fn get_pool(&mut self, id: &str) -> &Pool {
    self.hits += 1;
    self.pools.entry(id.into()).or_insert_with(Pool::new)
}

// ✓ Đúng
fn pool(&self, id: &str) -> Option<&Pool>
fn ensure_pool(&mut self, id: &str) -> &Pool   // tên nói rõ có thể tạo
```

## 6. Tránh lồng sâu — Guard clauses / Early return

```ts
// ✗ Sai — 4 cấp lồng
function saveConnection(conn?: Connection) {
  if (conn) {
    if (conn.name) {
      if (!isDuplicate(conn)) {
        store.add(conn);
      } else {
        throw new Error("duplicate");
      }
    } else {
      throw new Error("name required");
    }
  }
}

// ✓ Đúng — luồng chính ở cuối, phẳng
function saveConnection(conn?: Connection) {
  if (!conn) return;
  if (!conn.name) throw new ConnectionValidationError("name required");
  if (isDuplicate(conn)) throw new ConnectionValidationError("duplicate");
  store.add(conn);
}
```

```rust
// ✓ Rust — let-else / ? / early return
let Some(schema) = table.schema.as_deref() else {
    return Err(DbError::InvalidInput("schema required".into()));
};
```

Ngưỡng: độ sâu lồng > 3 → refactor. Mỗi `if` lồng thêm là một hàm tiềm năng.

## 7. Thay switch/if-chain lặp bằng polymorphism / bảng ánh xạ

```ts
// ✗ Sai — cùng switch xuất hiện ở 5 file
switch (provider) {
  case "postgres": return quotePg(id);
  case "sqlite": return quoteSqlite(id);
}

// ✓ Đúng — một chỗ duy nhất, mở rộng bằng thêm entry
const IDENTIFIER_QUOTERS: Record<Provider, (id: string) => string> = {
  postgres: quotePg,
  sqlite: quoteSqlite,
};
export const quoteIdentifier = (provider: Provider, id: string) => IDENTIFIER_QUOTERS[provider](id);
```

Rust: một `match` ở factory / `impl Trait for Provider` thay vì `match provider` rải rác.

## 8. DRY trong hàm

Hai hàm giống nhau > 5 dòng liên tiếp → trích hàm chung. Nhưng đọc `design-principles.md` mục DRY
để không gom nhầm hai thứ *tình cờ* giống nhau.

## 9. React-specific

- Component > 150 dòng (kể cả JSX) → tách sub-component / custom hook.
- Logic không liên quan đến render → custom hook hoặc hàm thuần ngoài component.
- Không định nghĩa component bên trong component.
- `useEffect` chỉ làm một việc; nhiều concern → nhiều `useEffect`.
- Handler dài > 10 dòng → đặt tên riêng thay vì inline arrow trong JSX.

## 10. Rust-specific

- Hàm `async` không nên giữ `MutexGuard` qua `.await`.
- Tránh `&mut self` khi `&self` đủ; tránh `self` (consume) khi `&self` đủ.
- Trả về `impl Iterator` / slice thay vì collect thành `Vec` nếu caller không cần sở hữu.
- Không `clone()` để "cho qua borrow checker" mà không hiểu vì sao — thường là dấu hiệu thiết kế sai.
- Hàm `pub` trong `crates/core` không nên phụ thuộc kiểu cụ thể của `sqlx`/`rusqlite` (giữ đúng ports/adapters).

## Checklist nhanh

- [ ] Mô tả được hàm bằng một câu không có "và"
- [ ] ≤ 30 dòng, lồng ≤ 3 cấp
- [ ] ≤ 3 tham số, không boolean flag
- [ ] Tên là động từ, đúng với việc hàm làm — không hơn, không kém
- [ ] Không sửa tham số đầu vào / state toàn cục ngoài ý tên hàm
- [ ] Query không mutate; command không "tiện thể" trả dữ liệu tính toán
- [ ] Có thể unit test không cần DB / Tauri / DOM
