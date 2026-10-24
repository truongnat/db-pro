---
name: bug-investigation
description: "Điều tra lỗi theo bằng chứng, tái hiện, lập giả thuyết, xác nhận nguyên nhân gốc và tạo regression test. Dùng khi có bug, lỗi runtime, dữ liệu sai, hành vi không ổn định hoặc test thất bại."
---
# Điều tra và xử lý bug

## Khi nào dùng
- Có triệu chứng lỗi, test fail, crash, trạng thái sai hoặc lỗi khó tái hiện.

## Không dùng khi
- Yêu cầu là thêm hành vi mới chưa tồn tại → `requirements-discovery`.
- Sự cố đang ảnh hưởng production và cần điều phối giảm tác động ngay → `incident-response` trước.

## Quy trình
1. Tìm memory theo thông báo lỗi chính xác, triệu chứng, subsystem và các từ đồng nghĩa.
2. Thu thập bằng chứng: bước tái hiện, input, môi trường/phiên bản, log đã che dữ liệu nhạy cảm và hành vi mong đợi/thực tế.
3. Xác định mức độ tái hiện: luôn xảy ra, có điều kiện hay chưa tái hiện được.
4. Đưa ra các giả thuyết có thể phân biệt bằng phép thử; ưu tiên thử nghiệm nhỏ giúp loại trừ giả thuyết.
5. Truy nguyên qua luồng dữ liệu/trạng thái và tìm nguyên nhân gốc; không dừng ở triệu chứng nếu có thể xác định nguyên nhân.
6. Dùng `bug-investigation.md` khi việc điều tra phức tạp hoặc cần lưu lại kết luận.
7. Sửa nguyên nhân đã xác nhận và thêm regression test tái hiện lỗi trước đây.
8. Chạy test mục tiêu, test liên quan và kiểm tra lỗi phụ có thể phát sinh.
9. Gọi `memory-curation` nếu cách sửa có thể tái sử dụng hoặc lỗi dễ tái diễn.

## Quy tắc
- Không kết luận root cause chỉ vì một thay đổi khiến lỗi tạm biến mất.
- Phân biệt sự kiện quan sát được, giả thuyết và kết luận.
- Nếu không tái hiện được, ghi rõ điều kiện đã thử và bằng chứng còn thiếu.
- Không vô tình che lỗi bằng catch rộng, timeout tăng tùy tiện hoặc fallback làm sai dữ liệu.

## Điều kiện hoàn tất
- Có nguyên nhân gốc được hỗ trợ bằng bằng chứng hoặc ghi rõ chưa xác định được.
- Fix có kiểm chứng và test bảo vệ phù hợp; các giới hạn còn lại được nêu rõ.

## Template liên quan
`.agents/template/software/bug-investigation.md`
