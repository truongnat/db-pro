# Clean Code Review Checklist

Copy vào PR description hoặc dùng khi self-review. Đánh dấu mức: **P1** chặn merge · **P2** sửa trong PR · **P3** follow-up.

## 0. Tự động

- [ ] `bash .skills/clean-code/scripts/clean-code-scan.sh` — không có `✗`, mọi `⚠` được giải thích
- [ ] Quality gates trong `AGENTS.md` đã **thực sự chạy** và pass

## 1. Đặt tên — `naming.md`

- [ ] Tên nói đúng ý định, không viết tắt vô nghĩa, đọc thành tiếng được
- [ ] Không magic number/string; hằng số có tên
- [ ] Đúng case theo ngôn ngữ (camel/Pascal/UPPER_SNAKE — snake/Pascal/SCREAMING_SNAKE)
- [ ] Không lặp ngữ cảnh (`connection.connectionHost`)
- [ ] Một khái niệm — một từ (`fetch` vs `get` vs `load` nhất quán)
- [ ] Hàm = động từ; boolean = `is/has/can/should`; test = mô tả hành vi

## 2. Hàm — `functions.md`

- [ ] ≤ 30 dòng (P2 nếu > 50; P1 nếu > 100 không lý do)
- [ ] Làm một việc, một mức trừu tượng; mô tả được không có chữ "và"
- [ ] ≤ 3 tham số; không boolean flag; không output argument
- [ ] Không side effect ẩn; query không mutate (CQS)
- [ ] Lồng ≤ 3 cấp; dùng guard clause / early return
- [ ] `switch(provider)` không lặp — registry / trait
- [ ] React: component ≤ 150 dòng, logic tách hook/hàm thuần, không component trong component

## 3. Comment — `comments.md`

- [ ] Trả lời "tại sao", không "cái gì"
- [ ] Không code comment-out
- [ ] `TODO(#issue|owner): việc cụ thể`
- [ ] Doc comment cho `pub` trong `crates/core` và export dùng chung trong `commons/`
- [ ] Comment cũ liên quan đã cập nhật theo code mới

## 4. Định dạng — `formatting.md`

- [ ] Prettier / rustfmt pass; không trộn commit format với logic
- [ ] File trong ngưỡng (TS ≤ 400, Rust ≤ 800 dòng cảnh báo)
- [ ] Thứ tự: imports → types/consts → public → private → tests
- [ ] Điều kiện dài đã đặt tên; khoảng trắng dọc tách khái niệm

## 5. Đối tượng & dữ liệu — `objects-and-data.md`

- [ ] Type là Object (ẩn field) hoặc Data Structure (DTO) — không hybrid
- [ ] DTO chỉ ở ranh giới IPC; chuyển đổi một chỗ (`From`/`TryFrom`/zod)
- [ ] Không train wreck `a.b().c().d()`; "Tell, don't ask"
- [ ] Không mutate state/cache dùng chung; ưu tiên `readonly` / bất biến
- [ ] Không `any`, `as unknown as`, `!`, `@ts-ignore` mới (P1 ở ranh giới IPC)
- [ ] Không `unwrap()`/`expect()` mới trên đường dữ liệu (P1 trong handler/adapter)
- [ ] Thư viện ngoài bọc sau adapter (`sqlx` không rò rỉ vào `core`)

## 6. Xử lý lỗi — `error-handling.md`

- [ ] **P1**: không `catch {}` trống, `let _ = fallible`, `.ok()`, `unwrap_or_default()` che lỗi
- [ ] Bỏ qua lỗi có chủ ý → comment lý do + log
- [ ] Hàm có `try` chỉ xử lý lỗi; logic thật ở hàm khác
- [ ] Lỗi có ngữ cảnh; phân loại theo `code`/variant, không `message.includes`
- [ ] Không exception cho luồng bình thường; `Option` vs `Result` đúng nghĩa
- [ ] Promise được await/catch; không set state sau unmount
- [ ] **P1**: tuân Database safety trong `AGENTS.md` (không claim atomicity không transaction, `affectedRows=0` ≠ success, capability-gate)

## 7. Nguyên tắc thiết kế — `design-principles.md`

- [ ] SRP: module có một lý do thay đổi
- [ ] OCP: thêm provider/format = thêm file, không sửa switch rải rác
- [ ] LSP: impl trait giữ contract, không `unimplemented!()` runtime
- [ ] ISP: interface/props chỉ chứa thứ client dùng
- [ ] DIP: `crates/core` không import driver; component không `invoke` trực tiếp
- [ ] DRY theo *kiến thức*; không gom thứ tình cờ giống nhau
- [ ] KISS: dev mới đọc 30s hiểu; không clever
- [ ] YAGNI: không tham số/trait/config/generic không dùng

## 8. Sức khỏe & workflow — `code-health.md`

- [ ] Không `console.*` / `dbg!` / `println!` debug / `debugger`
- [ ] Không `.only` / `.skip` / `#[ignore]` không lý do
- [ ] Không `eslint-disable` / `#[allow]` không lý do
- [ ] Không secret hardcode
- [ ] Refactor có test bảo vệ; test mới đặt tên theo hành vi
- [ ] Debt ngoài phạm vi ghi vào `FINDINGS.md` / issue, không mở rộng scope
- [ ] Tuyên bố "sạch hơn" có bằng chứng cụ thể (trước/sau)

## Mẫu finding

```text
[P2] functions — frontend/src/modules/export/export-dialog.tsx:42-131
Smell: handleExport 90 dòng trộn validate + build options + invoke + toast.
Hệ quả: không unit test được phần build options; sửa toast phải đọc cả hàm.
Đề xuất: tách buildExportOptions(form): ExportOptions (thuần, test) và
         useExportMutation() (side effect); handleExport còn ~10 dòng điều phối.
```
