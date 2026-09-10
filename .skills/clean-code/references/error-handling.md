# Error Handling — Xử lý ngoại lệ

Xử lý lỗi **quan trọng**, nhưng nếu nó làm mờ logic chính thì đó là code bẩn.
Mục tiêu: luồng chính đọc mạch lạc, lỗi được xử lý ở đúng tầng, **không bao giờ bị nuốt**.

## 1. Dùng cơ chế lỗi của ngôn ngữ — không dùng error code / sentinel

```ts
// ✗ Sai — caller phải nhớ kiểm tra ngay, và thường quên
function connect(cfg: Config): number {
  if (!cfg.host) return -1;
  if (!reachable(cfg)) return -2;
  return openSocket(cfg);
}
const fd = connect(cfg);
useSocket(fd); // -1 bị dùng như fd thật

// ✓ Đúng — không thể tiếp tục mà không xử lý
function connect(cfg: Config): Socket {
  if (!cfg.host) throw new ConnectionConfigError("host is required");
  if (!reachable(cfg)) throw new ConnectionRefusedError(cfg.host);
  return openSocket(cfg);
}
```

```rust
// ✗ Sai — bool/-1/Option làm lỗi mất thông tin
fn execute(sql: &str) -> bool
fn find_table(name: &str) -> Option<Table>   // None = không có? hay lỗi kết nối?

// ✓ Đúng — Result với error type domain, dùng ?
fn execute(sql: &str) -> Result<ExecutionSummary, DbError>
fn find_table(name: &str) -> Result<Option<Table>, DbError>  // phân biệt "không có" và "lỗi"
```

## 2. Luồng chính trước, xử lý lỗi tách riêng

```ts
// ✗ Sai — try/catch lồng, logic chính chìm trong xử lý lỗi
async function runQuery(tabId: string) {
  try {
    const tab = getTab(tabId);
    try {
      const result = await invoke("execute_query", { sql: tab.sql });
      try {
        setResult(tabId, result);
      } catch (e) {
        console.error(e);
      }
    } catch (e) {
      if ((e as Error).message.includes("timeout")) toast("timeout");
      else toast("failed");
    }
  } catch {
    // tab không tồn tại
  }
}

// ✓ Đúng — thân hàm là luồng vui; lỗi ở một chỗ
async function runQuery(tabId: string) {
  try {
    await executeAndStore(tabId);
  } catch (error) {
    reportQueryFailure(tabId, error);
  }
}

async function executeAndStore(tabId: string) {
  const tab = requireTab(tabId);
  const result = await queryService.execute(tab.sql);
  setResult(tabId, result);
}

function reportQueryFailure(tabId: string, error: unknown) {
  const failure = toQueryFailure(error);           // phân loại một chỗ
  setResultError(tabId, failure);
  logger.error("query failed", { tabId, code: failure.code });
}
```

Quy tắc: một hàm có `try` thì `try` nên là **câu lệnh đầu tiên** và không có gì sau `catch`/`finally` —
tức là hàm đó chỉ làm việc "xử lý lỗi", còn việc thật nằm trong hàm được gọi.

Rust: `?` làm điều này tự nhiên. Nếu thấy `match` trên `Result` lồng 3 cấp → dùng `?` + hàm riêng.

## 3. Không nuốt lỗi — TUYỆT ĐỐI

### Các dạng nuốt lỗi

```ts
// ✗ catch trống
try { await save(); } catch {}

// ✗ catch chỉ log rồi tiếp tục như không có gì
try { await save(); } catch (e) { console.log(e); }
return { ok: true };

// ✗ Promise không await, không .catch
save(); // unhandled rejection

// ✗ optional chaining che lỗi logic
const name = result?.rows?.[0]?.name ?? "";   // result undefined vì lỗi fetch? hay thật sự rỗng?
```

```rust
// ✗ let _ trên Result không có lý do
let _ = tx.commit().await;

// ✗ .ok() để bỏ qua
conn.execute(sql).await.ok();

// ✗ unwrap_or_default che lỗi
let rows = fetch_rows().await.unwrap_or_default();   // lỗi kết nối thành "0 dòng"

// ✗ if let Ok bỏ nhánh Err
if let Ok(meta) = introspect().await { render(meta); }   // Err đi đâu?
```

### Khi *thật sự* muốn bỏ qua

Phải (a) là quyết định có chủ ý, (b) ghi rõ lý do, (c) tốt nhất vẫn log ở mức debug/warn:

```rust
// ✓ Đúng — best-effort, có lý do, có log
if let Err(error) = app.emit("backup-progress", payload) {
    // Progress event là best-effort: UI có thể đã đóng, không được làm hỏng backup.
    tracing::debug!(%error, "failed to emit backup progress");
}
```

```ts
// ✓ Đúng
try {
  await navigator.clipboard.writeText(sql);
} catch (error) {
  // Clipboard bị từ chối trong iframe / không có quyền — hiển thị fallback, không chặn user.
  logger.warn("clipboard write failed", { error });
  showCopyFallback(sql);
}
```

`AGENTS.md` liệt kê rõ: **"treat affectedRows=0 as mutation success"** là vi phạm. Đó là một dạng nuốt lỗi nghiệp vụ.

## 4. Lỗi phải có ngữ cảnh

Message lỗi trả lời: **đang làm gì**, **với đầu vào nào**, **thất bại vì sao**.

```rust
// ✗ Sai
Err(DbError::Internal("failed".into()))

// ✓ Đúng
Err(DbError::IntrospectionFailed {
    schema: schema_name.to_owned(),
    table: table_name.to_owned(),
    source: Box::new(error),
})
```

```ts
// ✗ Sai
throw new Error("Invalid");

// ✓ Đúng
throw new ExportValidationError(`cannot export ${rows.length} rows: exceeds limit ${MAX_EXPORT_ROWS}`);
```

Không log **cả** ở nơi ném **và** nơi bắt (log kép làm nhiễu). Quy ước: log ở tầng **bắt cuối cùng**
(Tauri command handler / React error boundary / mutation `onError`), kèm ngữ cảnh gom được trên đường đi.

## 5. Định nghĩa lỗi theo **nhu cầu của caller**

Không tạo một exception class cho mỗi trường hợp nếu caller xử lý giống nhau. Ngược lại, không gộp
tất cả vào `Error` nếu caller cần phân biệt.

Trong repo:

- Backend: `DbError` (domain, `crates/core`) — caller (UI) cần phân biệt `AuthFailed` / `PermissionDenied` /
  `ConstraintViolation` / `QuerySyntax` để hiển thị hành động khác nhau → mỗi cái là variant.
- IPC: `CommandError` (`crates/tauri-app/src/dto.rs`) — chuyển từ `DbError` một chỗ (`From<DbError>`),
  mang `code` ổn định để frontend `switch`.
- Frontend: parse `CommandError` thành union type có `code`; component `switch` theo `code`, không `message.includes(...)`.

```ts
// ✗ Sai — string matching mong manh
if (error.message.includes("permission")) { ... }

// ✓ Đúng
if (isCommandError(error) && error.code === "PERMISSION_DENIED") { ... }
```

## 6. Bọc lỗi của thư viện bên thứ ba ở ranh giới

`crates/infrastructure/src/error.rs` (`from_sqlx`, `from_rusqlite`) là ví dụ đúng: mọi lỗi `sqlx::Error`
được ánh xạ thành `DbError` **một lần**, phần còn lại của hệ thống không biết `sqlx` tồn tại.

Áp dụng tương tự cho frontend: lỗi từ `invoke()` (Tauri) đi qua **một** hàm `toAppError(unknown): AppError`
trước khi vào store / component.

## 7. Không dùng exception cho luồng điều khiển bình thường

```ts
// ✗ Sai — "không tìm thấy" là trường hợp bình thường, không phải lỗi
try {
  const tab = getTabOrThrow(id);
  focus(tab);
} catch {
  createTab(id);
}

// ✓ Đúng
const tab = findTab(id);
if (tab) focus(tab); else createTab(id);
```

Rust: `Option` cho "có thể không có", `Result` cho "có thể thất bại". Không `Err` cho việc "user bấm cancel"
nếu cancel là luồng kỳ vọng — dùng enum kết quả `Outcome::Cancelled`.

## 8. Không trả về / truyền `null`

Xem `objects-and-data.md` mục 8. Tóm tắt: trả `Option` / `undefined` rõ ràng, collection rỗng, hoặc Null Object;
không nhận `null` làm tham số.

## 9. Async / cancellation / cleanup

- TS: mọi Promise phải được `await` hoặc `.catch`. `useEffect` async → dùng `AbortController`/flag `cancelled` để tránh set state sau unmount.
- TanStack Query mutation: xử lý ở `onError` + rollback optimistic update (xem `docs/plans/active/qa-p2-favorite-rollback`).
- Rust: cleanup bằng `Drop` / RAII, không dựa vào caller nhớ gọi `close()`. Transaction: `?` sau `begin` phải đảm bảo rollback khi drop.
- Không giữ lock qua `.await`; không `unwrap()` lock (poisoned → trả lỗi).

## 10. `AGENTS.md` — các lỗi nghiệp vụ không được che

Nhắc lại các luật liên quan trực tiếp tới error handling:

- Không claim atomicity khi không có transaction.
- `affectedRows = 0` **không** là thành công của mutation.
- Không "silently coerce" giá trị nhạy cảm về độ chính xác (decimal, bigint, timestamp) — lỗi hoặc cảnh báo rõ.
- Operation không được provider hỗ trợ → capability-gate với lý do rõ, không emit SQL rồi để DB báo lỗi.

## Checklist nhanh

- [ ] Không `catch {}` trống, không `let _ = fallible`, không `.ok()`/`unwrap_or_default()` che lỗi
- [ ] Mọi chỗ bỏ qua lỗi có comment lý do + log ở mức phù hợp
- [ ] Hàm có `try` chỉ làm việc xử lý lỗi; logic thật ở hàm khác
- [ ] Lỗi mang ngữ cảnh (đang làm gì / đầu vào / vì sao)
- [ ] Phân loại lỗi theo `code`/variant, không `message.includes`
- [ ] Lỗi thư viện ngoài được ánh xạ ở ranh giới (`from_sqlx`, `toAppError`)
- [ ] Không dùng exception cho luồng bình thường; `Option` vs `Result` đúng ngữ nghĩa
- [ ] Mọi Promise được await/catch; không set state sau unmount
- [ ] Không `unwrap()` / `expect()` trên đường dữ liệu trong handler và adapter
