---
name: incident-response
description: "Điều phối xử lý sự cố đang ảnh hưởng dịch vụ hoặc người dùng: xác định phạm vi, giảm tác động, cập nhật trạng thái, khôi phục và thực hiện postmortem."
---
# Ứng phó sự cố

## Khi nào dùng
- Production/service đang lỗi, suy giảm, mất dữ liệu, lỗi giao dịch hoặc tác động người dùng đang tiếp diễn.

## Không dùng khi
- Bug chưa phát hành và không ảnh hưởng hệ thống đang chạy → `bug-investigation`.

## Quy trình
1. Xác định tác động, thời điểm bắt đầu, hệ thống/phân khúc bị ảnh hưởng và mức độ tin cậy của dữ liệu hiện có.
2. Ưu tiên biện pháp giảm tác động an toàn, có thể đảo ngược: rollback, disable feature, traffic shift hoặc mitigation phù hợp.
3. Phân công vai trò/đầu mối nếu có; giữ timeline sự kiện, quyết định và cập nhật trạng thái rõ ràng.
4. Thu thập log/metric/trace đã lọc dữ liệu nhạy cảm; phân biệt xác nhận và giả thuyết.
5. Không thực hiện thay đổi phá hủy hoặc sửa dữ liệu hàng loạt nếu chưa có backup, kế hoạch và quyền phù hợp.
6. Xác minh dịch vụ đã khôi phục bằng các chỉ số và luồng nghiệp vụ liên quan, không chỉ dựa vào việc trang chủ mở được.
7. Sau khi ổn định, dùng `incident-postmortem.md` để ghi tác động, timeline, nguyên nhân, yếu tố góp phần và action item có owner/điều kiện hoàn tất.
8. Chuyển bài học lâu dài sang `memory-curation` khi nó hữu ích cho sự cố tương tự.

## Quy tắc
- Khôi phục dịch vụ và bảo vệ dữ liệu quan trọng hơn việc tìm thủ phạm.
- Không ghi secret, token hoặc dữ liệu người dùng không cần thiết vào báo cáo.
- Không khẳng định nguyên nhân cuối cùng trước khi có bằng chứng đủ mạnh.
- Action item phải kiểm tra được, tránh mục tiêu chung chung như “cẩn thận hơn”.

## Điều kiện hoàn tất
- Tác động đã được chặn/khôi phục hoặc trạng thái còn ảnh hưởng được công bố; postmortem và hành động tiếp theo có người phụ trách khi phù hợp.

## Template liên quan
`.agents/template/software/incident-postmortem.md`
