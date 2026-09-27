# Comments — Sử dụng nhận xét

> "Don't comment bad code — rewrite it." — Kernighan & Plauger

Comment là **thất bại trong việc diễn đạt bằng code**, chỉ nên dùng khi code, kiểu dữ liệu hoặc cấu trúc không thể diễn đạt đủ ý định. Comment không được compiler kiểm tra nên có xu hướng trở nên sai theo thời gian.

## Ngôn ngữ và thứ tự quyết định

- Tài liệu hướng dẫn này viết bằng tiếng Việt; comment trong source code phải viết bằng **tiếng Anh**, trừ khi repository có quy ước rõ ràng khác.
- Trước khi thêm comment: cải thiện tên, kiểu, cấu trúc hoặc tách hàm để code tự diễn đạt. Nếu thông tin có thể encode an toàn trong code, ưu tiên code.
- Chỉ giữ comment giải thích lý do không hiển nhiên: ràng buộc nghiệp vụ/kỹ thuật, invariant khó biểu diễn bằng type, trade-off, workaround, thuật toán bất ngờ hoặc nguy cơ từ một refactor tưởng như hợp lý. **Ngoại lệ có chủ đích:** mọi khai báo constant phải có comment riêng, theo quy tắc bên dưới.
- Đặt comment sát quyết định/code liên quan và cập nhật hoặc xóa cùng commit khi hành vi đổi. Dẫn nguồn ổn định khi cần; không biến comment thành bản sao RFC hay lịch sử thảo luận.

## Constant declarations — bắt buộc có comment

Mỗi khai báo Rust `const`, `static` và associated constant phải có một comment tiếng Anh ngắn, đặt ngay cạnh khai báo. Comment phải nói ý nghĩa ngữ nghĩa của giá trị, đơn vị/miền giá trị, invariant hoặc lý do tồn tại; không chỉ lặp lại identifier hay diễn giải cú pháp/giá trị. Đây là ngoại lệ có chủ đích đối với nguyên tắc chung chống phình to comment: luôn chú thích mỗi constant, nhưng giữ nội dung súc tích và giàu ý nghĩa.

```rust
// Maximum number of rows fetched per page to bound memory use during browsing.
const MAX_PAGE_SIZE: usize = 500;

// Milliseconds allowed for a connection attempt before the UI reports a timeout.
static CONNECTION_TIMEOUT_MS: u64 = 3_000;

impl RetryPolicy {
    // Number of retries after the initial attempt; zero disables retries.
    const MAX_RETRIES: u8 = 4;
}
```

```rust
// ✗ BAD: repeats the name without explaining what this limit means.
const MAX_PAGE_SIZE: usize = 500;

// ✗ BAD: translates the syntax/value rather than adding semantic meaning.
const MAX_RETRIES: u8 = 4; // Maximum retries is four.
```

## Comment hợp lệ (giữ lại / nên viết)

### 1. Giải thích *tại sao* (intent), không phải *cái gì*

```rust
// Keep the permission check before the generic SQLSTATE mapping: 42501 belongs
// to class 42 too, but callers must receive the actionable authorization error.
if code == "42501" {
    return DbError::PermissionDenied(message);
}
```

Comment nêu lý do quyết định; code đã nói điều gì đang diễn ra. Cảnh báo hệ quả cụ thể cũng hợp lệ khi có giới hạn thực tế, không chỉ là diễn giải luồng điều khiển.

```rust
// SAFETY: `ptr` came from Box::into_raw and is consumed exactly once here.
unsafe { drop(Box::from_raw(ptr)) }
```

### 2. Invariant, ràng buộc và thuật toán không hiển nhiên

Dùng comment cho invariant không thể biểu diễn bằng type, hành vi provider cụ thể, trade-off, workaround hoặc thuật toán gây bất ngờ. Nêu điều kiện cần giữ và vì sao; tránh kể lại từng bước hiển nhiên.

```rust
// `columns` and `values` stay aligned because RowBuilder::build validates their lengths.
struct Row { columns: Vec<String>, values: Vec<Value> }
```

### 3. Bảo vệ khỏi refactor gây hại

Ghi rõ điều gì không được thay đổi và lý do nếu một refactor có vẻ hợp lý có thể phá hợp đồng, tính đúng đắn hoặc an toàn. Đặt sát invariant; không dùng cảnh báo chung chung.

```rust
// Authorize the original schema name before normalization; normalizing first can
// make this check apply to a different database object.
```

### 4. TODO / FIXME / HACK / WORKAROUND có hành động

```rust
// TODO(#188): replace this fallback with provider capability detection when MySQL support lands.
// FIXME(#241): preserve the original error context when the retry API is available.
// HACK(#310): retain this ordering until the upstream parser handles quoted semicolons.
// WORKAROUND(#355): remove after driver 4.2 fixes pooled-connection cancellation.
```

`TODO` phải có owner hoặc issue và việc cụ thể: `TODO(<issue|owner>): <hành động>`. Dùng `FIXME` cho lỗi đã biết, `HACK` cho giải pháp mong manh, `WORKAROUND` cho né tránh hạn chế có căn cứ; nêu điều kiện gỡ bỏ. Không để ghi chú mồ côi: gắn tham chiếu/owner hoặc tạo issue, nếu không thì xử lý ngay.

### 5. Doc comment cho API

Doc comment giải thích **hợp đồng ngữ nghĩa** mà người gọi cần biết: hành vi, điều kiện biên, lỗi/panic có thể xảy ra, tác dụng phụ hoặc đảm bảo quan trọng. Không lặp chữ ký hay chỉ đổi tên hàm thành câu. Áp dụng theo quy ước API của module; với API public ở `crates/core`, ghi đủ hợp đồng cần thiết.

```rust
/// Lists tables visible to the current connection.
///
/// An existing schema with no visible tables produces an empty result. Permission
/// failures are returned as [`DbError::PermissionDenied`].
pub async fn list_tables(&self, schema_name: &str) -> Result<Vec<TableMetadata>, DbError>
```

Metadata nhãn như `Feature` hoặc tên màn hình là tùy chọn, không phải yêu cầu cho component reusable; chỉ thêm khi thực sự giúp hiểu ngữ cảnh.

### 6. Giá trị khó đọc từ API ngoài

```rust
// The driver uses 1 for enabled; this numeric option is part of its external API.
editor.set_option("word_wrap", 1);
```

Nếu code do mình kiểm soát, ưu tiên hằng số/enum có tên thay comment (xem `naming.md`). Với regex/thuật toán, chỉ giải thích giới hạn hoặc ngữ nghĩa cần bảo toàn, không chú giải cú pháp từng ký tự.

## Comment rác (xoá / không viết)

### 1. Lẩm bẩm / diễn giải lại code

```rust
// ✗ Sai
// Increment the index.
i += 1;
// Get connections.
let connections = get_connections();
```

### 2. Code bị comment-out

```rust
// ✗ Sai — Git đã nhớ.
// let legacy = build_legacy_tree(data);
return build_tree(data);
```

**Quy tắc: xóa ngay.** Dùng lịch sử Git để tìm lại khi cần.

### 3. Comment bù đắp cho tên xấu

```rust
// ✗ Sai
// Check whether the connection is read-only.
fn chk(c: &Connection) -> bool { ... }

// ✓ Đúng — tên đã diễn đạt ý nghĩa.
fn is_read_only(connection: &Connection) -> bool { ... }
```

### 4. Banner / phân cách trang trí

```rust
// ✗ Sai
// ==========================================
// ============ HELPER FUNCTIONS ============
// ==========================================
```

Nếu cần banner để điều hướng, cân nhắc tách file theo trách nhiệm.

### 5. Ghi chú tác giả, ngày, lịch sử hoặc lời tường thuật công việc

Không ghi tên tác giả, ngày sửa, lịch sử thay đổi, “implemented by AI”, tiến độ hay tường thuật quá trình làm việc vào source comment. Git và tài liệu công việc phù hợp hơn.

### 6. Comment ở dấu đóng ngoặc

```rust
// ✗ Sai
} // end function
```

Hàm cần comment này có thể cần được chia nhỏ.

### 7. Comment sai lệch / lỗi thời

Khi sửa code, sửa hoặc xóa comment liên quan trong cùng thay đổi. Comment mâu thuẫn với hành vi là lỗi cần sửa.

### 8. Doc comment thừa

```rust
// ✗ Sai: không bổ sung ý nghĩa ngoài chữ ký
/// Returns the name.
fn name(&self) -> &str
```

### 9. Comment phình to hoặc nhận xét về lựa chọn hiển nhiên

Không dán RFC, bảng SQLSTATE đầy đủ, lịch sử thảo luận hoặc lý do chọn lựa thông thường vào comment. Chỉ ghi phần cần để bảo toàn ý định; liên kết tài liệu/issue nếu cần chi tiết.

## Thay comment bằng code

| Thay vì | Hãy |
|---------|-----|
| `// check if eligible` trước biểu thức dài | đặt tên biểu thức hoặc trích hàm `is_eligible()` |
| `// step 1 ... // step 2 ...` | tách thành hàm có tên theo từng bước có ý nghĩa |
| `// magic: 3600000 = 1h` | `const ONE_HOUR: Duration = Duration::from_secs(3600);` |
| `// null nếu không tìm thấy` | kiểu trả về `Option<T>` |
| `// phải gọi init() trước` | thiết kế constructor/builder/typestate để không thể gọi sai |
| `// không được dùng ngoài module này` | giữ visibility private |

## Lint hỗ trợ

- TODO/FIXME format có thể được kiểm tra bằng lint phù hợp nếu repo cấu hình.
- Clippy `missing_docs` có thể dùng cho crate public khi phù hợp với policy của crate.
- Script `clean-code-scan.sh` là heuristic; dùng kết quả làm gợi ý, không thay thế review ngữ nghĩa.

## Checklist nhanh

- [ ] Đã cải thiện code trước khi quyết định cần comment chưa (ngoại trừ yêu cầu comment bắt buộc cho mọi `const`, `static` và associated constant)?
- [ ] Mọi khai báo Rust `const`, `static` và associated constant đều có comment tiếng Anh sát bên, giải thích ý nghĩa ngữ nghĩa chứ không chỉ nhắc lại tên/giá trị?
- [ ] Mỗi comment source viết bằng tiếng Anh theo quy ước repo và giải thích lý do, không phải cái gì?
- [ ] Comment mô tả ràng buộc/invariant/trade-off/workaround/thuật toán bất ngờ hoặc refactor nguy hiểm cụ thể?
- [ ] TODO/FIXME/HACK/WORKAROUND có hành động, owner/issue và điều kiện xử lý?
- [ ] Doc comment giải thích hợp đồng ngữ nghĩa, không lặp chữ ký?
- [ ] Comment đặt cạnh code và còn đúng sau thay đổi?
- [ ] Không có comment-out, banner, lịch sử, tường thuật AI hoặc comment phình to?
