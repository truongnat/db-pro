# Đánh giá mã (code review): <PR/thay đổi>

## Mục đích và phạm vi

- Hành vi mong đợi:
- Spec/issue liên quan:
- Nội dung chủ động không đưa vào phạm vi:

## Tính đúng đắn

- [ ] Tiêu chí chấp nhận và chuyển trạng thái được đáp ứng.
- [ ] Đã xem xét edge case, trạng thái rỗng/lỗi/loading, retry và concurrency.
- [ ] Contract API/schema/event và các caller vẫn tương thích.

## Chất lượng và bảo mật

- [ ] Tuân theo kiến trúc/pattern hiện có hoặc giải thích rõ ngoại lệ.
- [ ] Phân quyền, validation, secret và log nhạy cảm được xử lý an toàn.
- [ ] Xử lý lỗi, khả năng quan sát và hiệu năng phù hợp.
- [ ] Không có refactor ngoài phạm vi, code chết, sai sót với file sinh tự động hoặc secret vô tình bị thêm.

## Kiểm chứng

- Test/type check/lint/build đã chạy:
- Kiểm tra chưa chạy và lý do:
- Bằng chứng thủ công:

## Phát hiện

| Mức độ | File/dòng | Phát hiện và ảnh hưởng | Thay đổi cần thiết / đề xuất |
|---|---|---|---|

## Kết luận

- [ ] Chấp thuận
- [ ] Yêu cầu chỉnh sửa
- [ ] Chờ bổ sung bằng chứng

## Việc tiếp theo

- [ ] Cập nhật tài liệu/memory/monitoring khi có lý do phù hợp.
