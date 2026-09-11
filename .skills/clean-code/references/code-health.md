# Code Health & Workflow — Sức khỏe mã nguồn

Clean code không phải trạng thái đạt được một lần — là **thói quen** được tự động hoá hết mức có thể.

## 1. Refactoring liên tục

### Red → Green → Refactor

1. **Red**: viết test fail mô tả hành vi mong muốn.
2. **Green**: làm cho test pass bằng cách **đơn giản nhất** (kể cả xấu).
3. **Refactor**: dọn sạch **ngay** khi test còn xanh — không để "làm sau" (sau = không bao giờ).

Refactor = thay đổi cấu trúc **không** thay đổi hành vi. Test phải xanh trước và sau. Nếu chưa có test cho vùng
cần refactor → viết characterization test trước.

### Boy Scout Rule

> Rời khỏi file sạch hơn lúc bạn đến.

Áp dụng có kỷ luật:

- Sửa **nhỏ** (đổi tên biến, trích 1 hàm, xoá comment rác) trong file mình đang sửa → làm luôn, **commit riêng**.
- Refactor **lớn** (tách file, đổi API) → PR riêng, tham chiếu từ PR hiện tại. Không phình diff review.

`AGENTS.md`: *"Do not silently expand scope."*

### Catalogue refactoring hay dùng

| Smell | Refactoring |
|-------|-------------|
| Hàm dài | Extract Function; Replace Temp with Query; Decompose Conditional |
| Danh sách tham số dài | Introduce Parameter Object; Preserve Whole Object |
| Boolean flag | Split Function; Replace Parameter with Explicit Methods |
| `switch` lặp | Replace Conditional with Polymorphism; Registry map |
| Feature Envy (hàm dùng data của class khác nhiều hơn của mình) | Move Function |
| Data Clumps (3 biến luôn đi cùng nhau) | Extract Class / struct |
| Primitive Obsession (`string` cho id, sql, path) | Newtype / branded type |
| Comment giải thích khối code | Extract Function với tên = nội dung comment |
| Shotgun Surgery (một thay đổi đụng 10 file) | Move/Inline để gom về một chỗ |
| Divergent Change (một file đổi vì 10 lý do) | Split theo trách nhiệm (SRP) |
| Speculative Generality | Inline Class; Remove Parameter; Collapse Hierarchy |
| Dead code | Xoá (Git nhớ) |

### Tách một hàm Rust > 100 dòng — quy trình an toàn

`crates/ui` là boundary egui: hầu hết hàm dài là `draw_*` lồng nhiều closure. Kinh nghiệm từ đợt
đóng gate FOLLOW-UP #3 (20 hàm > 100 dòng → 0, `explorer_view.rs` 1.471 → 582 dòng):

1. **Đọc theo khối ≤ 150 dòng**, không đọc cả file — file lớn làm loãng ngữ cảnh và dễ sửa nhầm.
2. **Viết khối thay thế ra `/tmp/*.rs`** rồi splice bằng Python có `assert` kiểm tra biên (dòng đầu
   đúng chữ ký hàm, dòng cuối đúng `}`), thay từ **dưới lên trên** để số dòng phía trên không đổi.
   Dùng `Edit` cho hàm 100+ dòng rất dễ lệch một ký tự rồi fail match.
3. **Giữ nguyên hành vi tuyệt đối.** Ví dụ: `sql_layouter` tô màu số ở giữa chuỗi nhưng **không** tô
   màu số ở cuối input — bản tách phải giữ đúng sự bất đối xứng đó, không "sửa luôn cho đẹp".
4. **Vòng lặp kiểm chứng sau mỗi file**: `cargo fmt --all` → `cargo clippy -p <crate> --all-targets -- -D warnings`
   → `cargo test -p <crate> --offline` → commit riêng cho từng file.
5. **Gom tham số thành struct** khi clippy báo `too_many_arguments` hoặc scan báo > 3 tham số — repo
   đã dùng `CategoryFolder`, `GridCell`, `GridRows`, `TableDataPaging`, `ClosableTab`.
6. **Xoá trùng lặp thật, đừng chỉ di chuyển code.** Giá trị nằm ở chỗ 8 hint-row giống nhau → 1
   `draw_hint_row`; 6 category folder → 1 `draw_category_folder`; 26 chỗ `let _ = task_bridge.send`
   → 1 `DbProApp::dispatch_command` có doc-comment nêu lý do.

Với `let _ = <fallible>`: đừng copy-paste cùng một comment 26 lần. Tạo **một** helper có doc-comment
giải thích lý do, rồi thay mọi call-site bằng helper đó — scan vẫn xanh vì dòng `let _` duy nhất nằm
ngay dưới comment lý do.

### Heuristic của `clean-code-scan.sh` — điều cần biết

- `--diff` mới biến ngưỡng kích thước thành `✗`; quét toàn repo chỉ báo `⚠` (đó là nợ hiện có).
- Hàm khởi tạo chỉ liệt kê field trong struct literal (`Self { .. }`, không rẽ nhánh) được gắn
  `[data-literal]` — cảnh báo, không chặn, vì Rust không cho lấy derived `Default` khi đã tự viết
  `impl Default`.
- `#[cfg(test)]` chỉ bật chế độ bỏ qua phần còn lại của file khi nó gắn vào một `mod`. (Trước đây
  gắn vào `use` cũng bật, khiến `fn main` của `crates/native-app` bị che khỏi gate.)
- `let _ = <fallible>` chỉ được coi là "có lý do" khi **dòng ngay trên** là comment `//` hoặc `///`.
- Regex truyền cho `awk` phải đi qua `ENVIRON`, **không** dùng `awk -v`: `-v` xử lý escape sequence
  nên `\(` biến thành dấu mở group và `pub(super) fn` không bao giờ khớp.
- `MAX_SHOW=15` giới hạn số dòng ví dụ in ra; khi cần xem đủ danh sách, tạm tăng biến này trong một
  bản copy (đừng sửa file trong repo chỉ để debug).

## 2. Dọn dẹp trước khi push

### Cấm trong production code

| Loại | Frontend | Rust |
|------|----------|------|
| Debug output | `console.log`, `console.warn`, `console.debug`, `debugger` | `println!`, `dbg!`, `eprintln!` (ngoài CLI/main) |
| Code chết | code comment-out, hàm/export không dùng, import thừa | `#[allow(dead_code)]` không lý do, `unused` warnings |
| Tắt lint | `// eslint-disable*` không lý do, `@ts-ignore` | `#[allow(clippy::...)]` không comment lý do |
| Test residue | `it.only`, `describe.only`, `it.skip` không issue | `#[ignore]` không lý do |
| Placeholder | `TODO` không owner/issue, `FIXME`, `XXX`, `HACK` | `todo!()`, `unimplemented!()` ngoài prototype |
| Bí mật | key/token/password hardcode | tương tự |

Logging đúng cách: dùng logger có level (`tracing` ở Rust; wrapper `logger` ở frontend nếu có, nếu chưa → tạo trong `commons/`)
để có thể tắt theo môi trường. `console.error` cho lỗi thực sự ở error boundary là chấp nhận được.

### Lệnh kiểm tra nhanh

```bash
bash .skills/clean-code/scripts/clean-code-scan.sh
```

Script này quét mọi mục trong bảng trên và in `✓ / ⚠ / ✗` kèm vị trí.

## 3. Quality gates (từ `AGENTS.md`)

Luôn chạy trước khi tuyên bố PR sẵn sàng. **Không claim đã chạy nếu chưa chạy.**

Từ 2026-09-11 UI là native Rust và frontend React đã được archive, nên không còn gate
Node/pnpm. Gate hiện hành:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --locked -p db-pro-native
```

Gate frontend cũ (`pnpm install/typecheck/lint/format:check/check:tokens/test/build`) đã bị
gỡ khỏi CI cùng với frontend archive.

## 4. Đề xuất CI gate bổ sung cho clean code

Thêm vào workflow khi team đồng thuận (không tự ý thêm trong PR feature):

```yaml
- name: Clean code scan
  run: bash .skills/clean-code/scripts/clean-code-scan.sh --ci

- name: Clippy pedantic (warn only)
  run: cargo clippy --workspace --all-targets -- -W clippy::pedantic -W clippy::unwrap_used
  continue-on-error: true
```

Tuỳ chọn nâng cao (ngoài phạm vi mặc định): SonarQube/SonarCloud cho cognitive complexity & duplication,
`cargo-geiger` (unsafe), `cargo-machete` (unused deps). Gate frontend (`knip`, eslint) đã bị gỡ cùng
với frontend archive 2026-09-11.

## 5. Định nghĩa "Done" cho khía cạnh Clean Code

Một thay đổi được coi là sạch khi:

- [ ] Scan không có `✗`; mọi `⚠` còn lại được giải thích trong PR.
- [ ] Không thêm hàm > 50 dòng, file > ngưỡng, tham số > 3 mà không có lý do trong PR.
- [ ] Tên mới tuân `naming.md`; không có `any`/`unwrap()` mới trên đường dữ liệu.
- [ ] Không có `console.*` / `dbg!` / code comment-out / `.only`.
- [ ] Test mới đặt tên theo hành vi; refactor có test bảo vệ.
- [ ] Comment mới trả lời "tại sao"; comment cũ liên quan đã cập nhật.
- [ ] Quality gates đã **thực sự chạy** và pass.

## 6. Đo lường — không tuyên bố suông

`AGENTS.md`: *"Never claim performance improved without measurement evidence."* — áp dụng tương tự cho clean code.

Khi nói "đã refactor cho sạch hơn", nêu bằng chứng cụ thể:

- "Hàm `exportTable` 140 dòng → 4 hàm ≤ 25 dòng, mỗi hàm có unit test."
- "Xoá 3 bản copy của `quoteIdentifier`, còn 1 ở `commons/sql/quote.ts`."
- "Scan: 12 `console.log` → 0; 5 `unwrap()` trong handler → 0."
- "`crates/core` không còn import `sqlx` (grep rỗng)."

## 7. Quy trình xử lý technical debt

1. Phát hiện smell ngoài phạm vi PR → **không** sửa ngay.
2. Ghi vào `FINDINGS.md` của plan hiện tại (mức P2/P3) hoặc tạo issue với label `tech-debt`.
3. Nêu: vị trí, smell, tác động, đề xuất refactoring, ước lượng.
4. Ưu tiên debt ở **hot path thay đổi** (file bị sửa thường xuyên) hơn code ổn định ít ai đụng.

Kiểm tra hot path: `git log --since="3 months ago" --name-only --pretty=format: | sort | uniq -c | sort -rn | head -20`

## 8. Review clean code — thái độ

- Review **code**, không review **người**. "Hàm này làm 3 việc" thay vì "bạn viết hàm quá dài".
- Mỗi comment review: **smell → hệ quả → đề xuất cụ thể**. Không chỉ "refactor cái này".
- Phân mức P1/P2/P3 rõ ràng (xem `SKILL.md`); không chặn merge vì P3.
- Tác giả có quyền phản biện với bằng chứng; reviewer có quyền yêu cầu test.
- Self-review không thay thế review độc lập (`AGENTS.md` — Review infrastructure).
