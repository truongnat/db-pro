---
name: testing-validation
description: "Chọn và thực thi kiểm thử phù hợp cho thay đổi phần mềm; xác nhận acceptance criteria, regression, typecheck, lint và build bằng bằng chứng thực tế. Dùng trước khi kết luận task hoàn tất."
---
# Kiểm thử và xác minh

## Khi nào dùng
- Sau khi triển khai hoặc sửa bug; khi cần thiết kế test strategy hay xác minh một thay đổi.

## Không dùng khi
- Chỉ giải thích khái niệm không gắn với code.
- Đang cần ổn định incident trước; xử lý theo `incident-response` rồi xác minh có kiểm soát.

## Quy trình
1. Đọc test hiện có, script trong package/config và quy ước dự án; không đoán lệnh.
2. Đối chiếu test với acceptance criteria và rủi ro. Kiểm tra happy path, boundary, negative path, trạng thái lỗi và regression theo bề mặt ảnh hưởng.
3. Với thay đổi nhỏ, chạy test mục tiêu trước. Với thay đổi rộng, mở rộng sang integration, typecheck, lint và build phù hợp.
4. Đối với API/UI, xác minh contract và state mapping; với async flow, xem xét retry, duplicate event, timeout và trạng thái sau reload khi liên quan.
5. Nếu test fail, xác định lỗi do thay đổi hay môi trường; quay về `bug-investigation` nếu nguyên nhân chưa rõ.
6. Ghi chính xác command, pass/fail, test bị bỏ qua và lý do. Không suy diễn rằng build thành công từ việc test unit pass.
7. Dùng `test-plan.md` khi phạm vi/rủi ro cần kế hoạch kiểm thử có thể review.

## Quy tắc
- Không làm test yếu đi hoặc xóa assertion chỉ để pipeline xanh nếu chưa có căn cứ.
- Không bỏ qua test thất bại mà không báo cáo.
- Không dùng snapshot đơn lẻ thay cho assertion nghiệp vụ quan trọng.
- Nếu không thể chạy test, thực hiện static inspection có thể làm và nêu rõ rủi ro chưa được xác minh.

## Điều kiện hoàn tất
- Các kiểm tra phù hợp đã chạy và kết quả được ghi đúng; phần không kiểm tra được có lý do/ảnh hưởng rõ ràng.

## Template liên quan
`.agents/template/software/test-plan.md`
