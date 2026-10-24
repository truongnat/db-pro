---
name: code-review
description: "Review thay đổi code dựa trên diff và ngữ cảnh xung quanh, ưu tiên bug, regression, bảo mật, dữ liệu và khả năng bảo trì. Dùng trước merge hoặc khi được yêu cầu đánh giá chất lượng code."
---
# Review code

## Khi nào dùng
- Trước khi merge/release, sau thay đổi đáng kể hoặc khi được yêu cầu review độc lập.

## Không dùng khi
- Chỉ cần implement một yêu cầu chưa xong; dùng `implementation` rồi review diff sau.

## Quy trình
1. Đọc yêu cầu/acceptance criteria và memory có liên quan; xác định phạm vi diff.
2. Đọc diff cùng phần code xung quanh, caller/callee, test, schema và contract liên quan.
3. Tìm lỗi logic, xử lý trạng thái, race condition, validation, quyền truy cập, rò rỉ dữ liệu, tương thích ngược và regression.
4. Kiểm tra test mới có thật sự bắt được lỗi/đảm bảo hành vi cần thiết không.
5. Xếp phát hiện theo mức độ ảnh hưởng; mỗi finding cần vị trí cụ thể, điều kiện xảy ra, tác động và hướng xử lý.
6. Ghi phát hiện vào `code-review.md` nếu review lớn hoặc cần lưu lịch sử. Nếu không phát hiện vấn đề, nêu rõ phạm vi đã xem và giới hạn.

## Quy tắc
- Chỉ báo cáo điều có cơ sở từ diff/code/test; không tạo finding dựa trên phỏng đoán không kiểm chứng.
- Ưu tiên finding có thể hành động, không tập trung vào sở thích formatting khi đã có formatter.
- Phân biệt bug chắc chắn với câu hỏi cần xác minh.
- Không khẳng định review đầy đủ nếu chỉ xem một phần file hoặc không thể chạy test.

## Điều kiện hoàn tất
- Findings được phân loại rõ; blocker/high severity được xử lý hoặc được chấp nhận có chủ đích trước merge.

## Template liên quan
`.agents/template/software/code-review.md`
