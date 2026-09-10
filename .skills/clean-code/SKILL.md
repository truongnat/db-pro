---
name: clean-code
description: "Clean Code standards and review checklist for the DB Pro codebase (TypeScript + React 19 frontend, Rust backend). Use when the user asks to write clean code, review code quality, refactor, rename, split a large function/file, remove code smells, apply SOLID / DRY / KISS / YAGNI, fix error handling, clean up comments or logs, or run a code-health scan before a PR. Triggers on: /clean-code, clean code, code review, refactor, code smell, naming, SOLID, DRY, KISS, YAGNI, readability, maintainability, code health scan."
---

# Clean Code

Bộ tiêu chuẩn Clean Code từ cơ bản đến thực chiến, áp dụng cho toàn bộ mã nguồn DB Pro
(`frontend/` — TypeScript/React, `crates/` — Rust). Mỗi mục bên dưới có một file tham chiếu
chi tiết với ví dụ **Sai vs Đúng** trong `references/`.

## Quick Start

Quét tự động các code smell phổ biến trước khi mở PR:

```bash
bash .skills/clean-code/scripts/clean-code-scan.sh --diff          # chỉ file thay đổi so với main (dùng trước PR)
bash .skills/clean-code/scripts/clean-code-scan.sh                 # toàn repo (baseline nợ kỹ thuật)
bash .skills/clean-code/scripts/clean-code-scan.sh frontend        # chỉ TypeScript/React
bash .skills/clean-code/scripts/clean-code-scan.sh rust            # chỉ Rust
bash .skills/clean-code/scripts/clean-code-scan.sh --with-linters  # kèm eslint/prettier/tsc/fmt/clippy
bash .skills/clean-code/scripts/clean-code-scan.sh --diff --ci     # exit 1 nếu có ✗ (dùng trong CI)
```

Ý nghĩa kết quả:

- `✗` — smell nghiêm trọng (debug log, `catch {}` trống, `todo!()`, core import driver…): sửa trước khi mở PR.
- `⚠` — cần nhìn kỹ (hàm/file dài, nhiều tham số, `unwrap()`, `let _ =`…): giải thích trong PR nếu giữ.
- Ngưỡng kích thước hàm/file chỉ tính là `✗` ở chế độ `--diff` (gate code mới); quét toàn repo chỉ báo `⚠` vì đó là nợ hiện có.
- Script là heuristic grep/awk — dùng làm bộ lọc, không phải phán quyết.

Khi review hoặc refactor thủ công, dùng `references/review-checklist.md`.

## Khi nào dùng skill này

- Trước khi mở PR: chạy scan + rà checklist cho phần diff.
- Khi được yêu cầu review, refactor, đặt tên lại, tách hàm/file lớn.
- Khi thêm module mới: thiết kế theo nguyên tắc trong mục 6 và 8.
- Khi sửa lỗi: kiểm tra mục 7 (Error Handling) để không "nuốt lỗi".

Skill này **không** thay thế các quality gate trong `AGENTS.md` (typecheck, lint, format,
test, clippy). Nó bổ sung tầng đánh giá về khả năng đọc / bảo trì mà linter không bắt được.

## 1. Ý nghĩa của Clean Code

Bốn thuộc tính đo lường một đoạn code "sạch":

| Thuộc tính | Câu hỏi kiểm tra |
|------------|------------------|
| **Readability** — dễ đọc | Người mới đọc lần đầu có hiểu *ý định* mà không cần chạy code không? |
| **Maintainability** — dễ bảo trì | Sửa một logic cũ có phải đụng tới nhiều file không liên quan không? |
| **Extensibility** — dễ mở rộng | Thêm provider/format/command mới có phải sửa code cũ (thay vì thêm mới) không? |
| **Testability** — dễ kiểm thử | Có unit test được hàm này mà không cần DB thật, Tauri runtime hay DOM không? |

Nguyên tắc vàng: **code được đọc nhiều hơn được viết gấp ~10 lần** — tối ưu cho người đọc.

## 2. Tiêu chuẩn đặt tên (Naming)

- **Ý nghĩa rõ ràng**: tên thể hiện *mục đích*, không viết tắt vô nghĩa (`d`, `tmp`, `data2`).
- **Đọc thành tiếng được**: `generationTimestamp` thay vì `genymdhms`.
- **Tìm kiếm được**: không dùng magic number/string; đưa vào hằng số có tên.
- **Đúng cú pháp theo ngôn ngữ**:
  - TypeScript: `camelCase` biến/hàm, `PascalCase` type/component, `UPPER_SNAKE` hằng số module-level.
  - Rust: `snake_case` hàm/biến/module, `PascalCase` type/trait/enum variant, `SCREAMING_SNAKE` const/static.
- **Không lặp ngữ cảnh**: `connection.host` thay vì `connection.connectionHost`.
- Hàm = động từ (`fetchSchema`, `build_query`), boolean = câu hỏi (`isReadOnly`, `has_capability`).

→ Chi tiết + ví dụ: `references/naming.md`

## 3. Thiết kế hàm (Functions & Methods)

- **Nhỏ**: mục tiêu < 20–30 dòng; > 50 dòng là tín hiệu phải tách.
- **Đơn nhiệm (Do One Thing)**: một hàm — một mức trừu tượng — một lý do thay đổi.
- **Ít tham số**: 0–2 lý tưởng; ≥ 3 → gom vào object/struct options (`interface XxxOptions`, `struct XxxOptions`).
- **Không side effect ẩn**: không sửa state toàn cục / tham số đầu vào khi tên hàm không nói vậy.
- **Command–Query Separation**: hàm trả lời câu hỏi (query) không được thay đổi dữ liệu (command).
- **Không boolean flag param**: `render(true)` → tách thành hai hàm hoặc dùng enum.
- Tránh lồng sâu (> 2–3 cấp): dùng early return / guard clause.

→ Chi tiết + ví dụ: `references/functions.md`

## 4. Sử dụng nhận xét (Comments)

- Code tốt tự giải thích; comment giải thích **tại sao (why)**, không giải thích **cái gì (what)**.
- **Comment hợp lệ**: lý do của quyết định thiết kế bất thường, cảnh báo hệ quả, invariant, `TODO(owner/issue)`,
  doc comment cho public API (`///` Rust, JSDoc cho hàm export dùng chung).
- **Comment rác**: diễn giải lại code, code cũ bị comment (dùng Git), banner trang trí, comment sai lệch với code,
  ghi chú tác giả/ngày (Git đã có).

→ Chi tiết + ví dụ: `references/comments.md`

## 5. Định dạng mã nguồn (Formatting & Layout)

Repo đã cố định formatter — **không tranh luận về style, chạy công cụ**:

| Công cụ | Cấu hình | Giới hạn dòng |
|---------|----------|---------------|
| Prettier (`frontend/.prettierrc`) | `semi`, double quote, trailing comma `all` | 100 ký tự |
| rustfmt (`.rustfmt.toml`) | edition 2021, 4 spaces | 120 ký tự |

- **Mật độ dọc**: các hàm liên quan nằm gần nhau; hàm gọi ở trên hàm được gọi (đọc từ trên xuống như báo).
- **Khoảng trắng dọc** tách các "đoạn ý" trong hàm; không có dòng trống vô nghĩa liên tiếp.
- **Kích thước file**: > 400 dòng → cân nhắc tách theo trách nhiệm.
- Thứ tự trong file: imports → types/consts → public API → private helpers.

→ Chi tiết: `references/formatting.md`

## 6. Đối tượng & cấu trúc dữ liệu

- **Đóng gói**: ẩn state, expose hành vi. Rust: field private + constructor; TS: không export mutable state thô.
- **Phân biệt Object vs Data Structure**: DTO/struct dữ liệu thì phơi field và không có logic; object có hành vi thì ẩn field.
- **Law of Demeter**: chỉ gọi method của `this`, tham số, object tự tạo, hoặc field trực tiếp. Tránh chuỗi `a.getB().getC().doD()`.
- Ưu tiên **composition** và **trait/interface** hơn kế thừa; ưu tiên immutable (`readonly`, không `mut` nếu không cần).
- Không dùng `any` / `as unknown as X` để "lách" type; Rust không `unwrap()` ngoài test/hằng số chứng minh được.

→ Chi tiết + ví dụ: `references/objects-and-data.md`

## 7. Xử lý ngoại lệ (Error Handling)

- **Không trả về error code / null ngầm**: TS ném `Error` có type hoặc trả `Result`-like rõ ràng; Rust dùng `Result<T, E>` với `?`.
- **Luồng chính đọc được trước**: tách phần xử lý lỗi ra hàm riêng, tránh `try/catch` lồng nhau.
- **Không nuốt lỗi**: cấm `catch {}` trống, `let _ = fallible()` không lý do, `.ok()` để bỏ qua lỗi âm thầm.
- Lỗi phải mang **ngữ cảnh** (đang làm gì, với đầu vào nào) và được ánh xạ thành lỗi domain (`DbError`, `CommandError`).
- Không dùng exception cho luồng điều khiển bình thường.
- Không trả về / truyền `null` khi có thể dùng `Option`, mảng rỗng, hoặc Null Object.

→ Chi tiết + ví dụ: `references/error-handling.md`

## 8. Nguyên tắc thiết kế (Design Principles)

- **SOLID**: SRP, OCP, LSP, ISP, DIP — ví dụ ánh xạ vào kiến trúc ports/adapters của `crates/core` và DI trong `frontend/src/commons/di`.
- **DRY**: không lặp *kiến thức* (không chỉ là không lặp *text*). Hai đoạn giống nhau nhưng thay đổi vì lý do khác nhau thì **không** phải duplication.
- **KISS**: giải pháp đơn giản nhất còn đúng. Không generic hoá khi chỉ có một use case.
- **YAGNI**: không viết trước abstraction/feature chưa có yêu cầu.

→ Chi tiết + ví dụ: `references/design-principles.md`

## 9. Sức khỏe mã nguồn (Code Health & Workflow)

- **Boy Scout Rule**: rời khỏi file sạch hơn lúc tới, nhưng refactor trong PR riêng nếu diff lớn.
- **Red → Green → Refactor**: refactor ngay khi test xanh, không để "làm sau".
- **Dọn dẹp trước khi push**: xóa `console.log`/`console.warn` debug, `dbg!`, `println!`, code comment-out, `eslint-disable` không lý do, `#[allow(...)]` không giải thích.
- **Tự động hoá**: quality gates trong `AGENTS.md` + `scripts/clean-code-scan.sh`; đề xuất CI gate trong `references/code-health.md`.
- **Đo lường**: không tuyên bố "sạch hơn" nếu không chỉ ra được smell nào đã biến mất.

→ Chi tiết: `references/code-health.md`

## Quy trình review Clean Code cho một PR

1. Chạy `bash .skills/clean-code/scripts/clean-code-scan.sh` — sửa mọi `✗`, giải thích mọi `⚠` còn lại.
2. Đọc diff theo `references/review-checklist.md`, ghi nhận theo mức độ:
   - **P1** (chặn merge): nuốt lỗi, side effect ẩn, `unwrap()` trên đường dữ liệu, `any` ở ranh giới IPC, vi phạm Database safety trong `AGENTS.md`.
   - **P2** (nên sửa trong PR): hàm > 50 dòng, > 3 tham số, tên mơ hồ, comment rác, duplication.
   - **P3** (ghi nhận / follow-up): style nhỏ, cơ hội refactor lớn hơn phạm vi PR.
3. Với mỗi finding: trích đoạn code, nêu smell, đề xuất cụ thể (kèm tên mới / cách tách).
4. Không tự mở rộng phạm vi PR để refactor toàn cục — tạo follow-up.

## Resources

- `references/naming.md` — Đặt tên: quy ước theo ngôn ngữ + ví dụ Sai/Đúng
- `references/functions.md` — Thiết kế hàm: kích thước, tham số, side effect, CQS
- `references/comments.md` — Comment hợp lệ vs comment rác
- `references/formatting.md` — Layout, thứ tự trong file, giới hạn kích thước
- `references/objects-and-data.md` — Đóng gói, Law of Demeter, Object vs Data Structure
- `references/error-handling.md` — Result/Exception, ngữ cảnh lỗi, cấm nuốt lỗi
- `references/design-principles.md` — SOLID, DRY, KISS, YAGNI với ví dụ trong repo
- `references/code-health.md` — Refactoring workflow, dọn dẹp, CI gates
- `references/review-checklist.md` — Checklist copy-paste khi review PR
- `scripts/clean-code-scan.sh` — Script quét code smell tự động
