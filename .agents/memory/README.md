# Kho memory

Memory lưu **kiến thức đã chọn lọc và rút ra**, không phải nhật ký đầy đủ của cả phiên làm việc.

## Phân loại

- `entries/bugfixes/`: triệu chứng → nguyên nhân gốc → cách sửa → kiểm chứng.
- `entries/decisions/`: quyết định và trade-off có ảnh hưởng dài hạn.
- `entries/patterns/`: pattern kiến trúc/code đã được chứng minh trong phạm vi xác định.
- `entries/workflows/`: quy trình/lệnh có thể lặp lại.
- `entries/environment/`: đặc điểm môi trường, toolchain, triển khai hoặc dependency dễ quên.
- `archive/YYYY/`: kiến thức lịch sử, không nằm trong kết quả tìm kiếm mặc định.

## Quy trình ghi memory

1. Tìm chủ đề tương tự trước khi tạo file.
2. Dùng `../template/memory/memory-entry.md`.
3. Mỗi entry tập trung vào một bài học có thể tái sử dụng; mục tiêu khoảng 200–700 từ, trừ khi trường hợp cần dài hơn.
4. Nêu bằng chứng và phạm vi áp dụng; phân biệt sự thật đã xác nhận với giả thuyết.
5. Chỉ cập nhật `INDEX.md` khi entry có giá trị cao hoặc thường xuyên cần tra cứu.
6. Theo dõi thay đổi bằng Git và review như code.

## Tìm kiếm

Chạy từ thư mục gốc của project:

```bash
.agents/scripts/search-memory.sh "exact error or concept"
.agents/scripts/search-memory.sh --archive "exact error or concept"
```

Nếu repository không có script, chạy `"$HOME/.agents/scripts/search-memory.sh" "exact error or concept"`. Script tự nhận diện Git root hiện tại và tìm cả memory của project lẫn global.

## db-pro

- **196** entry mirror từ `docs/plans/` trong `entries/migrated/from-plans/` (`MIGRATION_MANIFEST.json`).
- Entry đã kiểm chứng tay: `environment/`, `patterns/`, `workflows/` — xem `INDEX.md`.
- Lifecycle owner: `docs/plans/active/agents-memory-framework/` (STATE: **REVIEW**, chưa COMPLETED).
