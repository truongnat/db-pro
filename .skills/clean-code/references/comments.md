# Comments — Sử dụng nhận xét

> "Don't comment bad code — rewrite it." — Kernighan & Plauger

Comment là **thất bại trong việc diễn đạt bằng code**, được chấp nhận khi code không thể diễn đạt được.
Comment không được compiler kiểm tra nên **luôn có xu hướng trở nên sai** theo thời gian.

## Comment hợp lệ (giữ lại / nên viết)

### 1. Giải thích *tại sao* (intent), không phải *cái gì*

```rust
// ✓ Đúng — code nói "gì", comment nói "tại sao"
// Permission denied (42501) nằm trong nhóm 42xxx (syntax) của SQLSTATE,
// phải kiểm tra trước khi ánh xạ chung thành QuerySyntax.
if code_str == "42501" {
    return DbError::PermissionDenied(db_err.message().into());
}
```

```ts
// ✓ Đúng
// Monaco giữ reference tới model sau khi component unmount; dispose thủ công
// để tránh leak khi user mở/đóng hàng trăm tab (xem #142).
useEffect(() => () => model.dispose(), [model]);
```

### 2. Cảnh báo hệ quả

```ts
// ✓ Đúng
// WARNING: hàm này chạy trên main thread; với > 1000 bảng hãy dùng layoutInWorker().
export function layoutSync(nodes: Node[]) { ... }
```

```rust
// ✓ Đúng
// SAFETY: `ptr` được cấp phát bởi Box::into_raw ở dòng 42 và chỉ được free tại đây.
unsafe { drop(Box::from_raw(ptr)) }
```

### 3. Invariant / giả định không thể encode bằng type

```rust
// ✓ Đúng
// Invariant: `columns` và `values` luôn cùng độ dài — được đảm bảo bởi RowBuilder::build().
```

### 4. TODO / FIXME có chủ và có tham chiếu

```ts
// ✓ Đúng
// TODO(#188): thay thế bằng capability check khi provider MySQL được thêm.

// ✗ Sai — TODO mồ côi, không ai biết khi nào/ai làm
// TODO: fix later
```

Quy tắc: `TODO(<issue|owner>): <việc cụ thể>`. TODO không có tham chiếu > 1 sprint → xoá hoặc tạo issue.

### 5. Doc comment cho public API

- Rust: `///` cho mọi item `pub` trong `crates/core` (ports, domain types). Mô tả hành vi, lỗi trả về, panic (nếu có).
- TypeScript: JSDoc cho hàm/hook export dùng bởi nhiều module — đặc biệt trong `frontend/src/commons/`.

```rust
/// Introspects all tables visible to the current connection.
///
/// Returns an empty `Vec` when the schema exists but has no tables.
///
/// # Errors
/// - [`DbError::PermissionDenied`] when the role lacks `USAGE` on the schema.
/// - [`DbError::NotFound`] when `schema_name` does not exist.
pub async fn list_tables(&self, schema_name: &str) -> Result<Vec<TableMetadata>, DbError>
```

Doc comment **không** lặp lại chữ ký. `/// Returns the name.` cho `fn name()` là rác.

### 6. Làm rõ giá trị khó đọc (khi không đổi được API bên ngoài)

```ts
// ✓ Đúng — API bên thứ ba dùng số, ta không đổi được
editor.setOption("wordWrap", 1 /* on */);
```

Nếu code do mình kiểm soát → dùng hằng số/enum thay vì comment (xem `naming.md`).

### 7. Giải thích regex / thuật toán không hiển nhiên

```ts
// ✓ Đúng
// Khớp identifier PostgreSQL không quote: bắt đầu bằng chữ/_ rồi chữ/số/_/$; tối đa 63 byte.
const UNQUOTED_IDENTIFIER = /^[a-z_][a-z0-9_$]{0,62}$/;
```

## Comment rác (xoá / không viết)

### 1. Lẩm bẩm / diễn giải lại code

```ts
// ✗ Sai
// tăng i lên 1
i++;
// lấy danh sách connection
const connections = getConnections();
// nếu không có connection thì return
if (!connections.length) return;
```

### 2. Code bị comment-out

```ts
// ✗ Sai — Git đã nhớ. Không ai dám xoá vì "chắc còn dùng".
// const legacy = buildLegacyTree(data);
// return legacy;
return buildTree(data);
```

**Quy tắc: xoá ngay.** `git log -S "buildLegacyTree"` tìm lại được khi cần.

### 3. Comment bù đắp cho tên xấu

```ts
// ✗ Sai
// kiểm tra xem connection có phải read-only không
function chk(c: Conn) { ... }

// ✓ Đúng — không cần comment
function isReadOnly(connection: Connection) { ... }
```

### 4. Banner / phân cách trang trí

```ts
// ✗ Sai
// ==========================================
// ============ HELPER FUNCTIONS ============
// ==========================================
```

Nếu file cần banner để điều hướng → file quá lớn, tách file.

### 5. Ghi chú tác giả, ngày, lịch sử

```ts
// ✗ Sai
// Created by X on 2025-03-01
// Modified by Y on 2025-04-12: fix bug
```

`git blame` làm việc này tốt hơn.

### 6. Comment ở dấu đóng ngoặc

```ts
// ✗ Sai
    } // end if
  } // end for
} // end function
```

Hàm dài đến mức cần cái này → tách hàm.

### 7. Comment sai lệch / lỗi thời

Nguy hiểm nhất. Khi sửa code, **sửa hoặc xoá comment liên quan trong cùng commit**.
Reviewer: đọc comment rồi so với code — lệch nhau là **P2**.

### 8. Comment thừa trong JSDoc/doc

```ts
// ✗ Sai
/**
 * @param connectionId the connection id
 * @returns the result
 */
```

Nếu JSDoc không thêm thông tin ngoài chữ ký → xoá.

### 9. Comment "thông tin quá nhiều"

Không dán RFC, bảng SQLSTATE đầy đủ, lịch sử thảo luận vào comment. Đặt link tới doc / issue.

## Thay comment bằng code

| Thay vì | Hãy |
|---------|-----|
| `// check if eligible` trước một biểu thức dài | `const isEligible = ...;` hoặc `function isEligible()` |
| `// step 1 ... // step 2 ...` trong một hàm | tách mỗi step thành hàm có tên |
| `// magic: 3600000 = 1h` | `const ONE_HOUR_MS = 60 * 60 * 1000;` |
| `// null nếu không tìm thấy` | kiểu trả về `Option<T>` / `T \| undefined` |
| `// phải gọi init() trước` | thiết kế để không thể gọi sai (constructor, builder, typestate) |
| `// không được dùng ngoài module này` | không `export` / không `pub` |

## Lint hỗ trợ

- ESLint `no-warning-comments` (cảnh báo TODO/FIXME không đúng định dạng) — tuỳ chọn.
- Clippy: `missing_docs` cho crate public (bật `#![warn(missing_docs)]` trong `crates/core/src/lib.rs` nếu muốn ép).
- `scripts/clean-code-scan.sh` đếm code comment-out, TODO mồ côi và `eslint-disable`/`#[allow]` không lý do.

## Checklist nhanh

- [ ] Mỗi comment trả lời "tại sao", không phải "cái gì"
- [ ] Không có code comment-out
- [ ] TODO có `(#issue|owner)` và việc cụ thể
- [ ] Doc comment cho `pub` trong `crates/core` và hàm export trong `commons/`
- [ ] Comment còn đúng với code sau khi sửa
- [ ] Đã thử đổi tên / trích hàm trước khi viết comment
