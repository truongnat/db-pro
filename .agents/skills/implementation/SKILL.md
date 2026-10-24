---
name: implementation
description: "Triển khai feature hoặc thay đổi phần mềm theo yêu cầu, thiết kế và convention hiện tại; giữ diff nhỏ, an toàn và có thể kiểm chứng. Dùng khi đã đủ ngữ cảnh để sửa code."
---
# Triển khai thay đổi phần mềm

## Khi nào dùng
- Thêm/sửa hành vi, API, UI, data access, integration hoặc tooling.
- Tiếp tục công việc sau khi yêu cầu và quyết định chính đã rõ.

## Không dùng khi
- Chưa hiểu lỗi hoặc chưa tái hiện được vấn đề → `bug-investigation`.
- Chỉ cần đánh giá diff hiện có, không sửa code → `code-review`.

## Quy trình
1. Tìm memory liên quan và đọc file/luồng hiện có trước khi chỉnh sửa.
2. Xác định acceptance criteria, contract và phạm vi file chịu ảnh hưởng.
3. Lập kế hoạch ngắn cho thay đổi nhiều bước; dùng `implementation-plan.md` khi hữu ích.
4. Thực hiện thay đổi nhỏ nhất nhưng hoàn chỉnh; giữ phong cách, naming, kiến trúc và dependency hiện hữu.
5. Xử lý loading/error/empty state, validation, phân quyền, retry và idempotency khi phù hợp với loại task.
6. Không để mock, TODO hoặc fallback giả làm hành vi production nếu chưa được yêu cầu rõ.
7. Chạy kiểm tra cục bộ sau mỗi thay đổi rủi ro; cập nhật test cùng với hành vi.
8. Dùng `testing-validation` để hoàn tất kiểm chứng, sau đó xem diff.

## Quy tắc
- Không tự tiện xóa thay đổi chưa commit của người dùng hoặc ghi đè file ngoài phạm vi.
- Không thêm secret vào source, log, test fixture hoặc memory.
- Không tuyên bố đã chạy lệnh/test nếu chưa thực thi.
- Nếu môi trường thiếu dependency/quyền truy cập, làm tối đa phần có thể và nêu rõ phần không xác minh được.
- Nếu yêu cầu đổi đáng kể sau khi bắt đầu, cập nhật phạm vi và kế hoạch trước khi tiếp tục.

## Điều kiện hoàn tất
- Thay đổi đáp ứng acceptance criteria, có test/kiểm chứng thích hợp, diff không chứa nội dung ngoài phạm vi và các giới hạn được công bố rõ.
