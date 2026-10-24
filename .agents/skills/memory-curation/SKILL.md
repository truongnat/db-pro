---
name: memory-curation
description: "Tìm, xác minh, tạo, cập nhật hoặc thay thế memory Markdown sau khi giải quyết vấn đề hoặc hoàn thành công việc có bài học tái sử dụng. Dùng trước task kỹ thuật đáng kể và khi chốt task."
---
# Quản trị memory kỹ thuật

## Khi nào dùng
- Trước bug/feature/refactor/migration để tìm giải pháp tương tự.
- Sau khi xác minh root cause, quyết định, workaround hoặc quy trình có thể tái sử dụng.
- Khi phát hiện memory trùng, lỗi thời hoặc mâu thuẫn.

## Không dùng khi
- Công việc không tạo ra kiến thức bền vững và chưa có memory phù hợp cần cập nhật.
- Chỉ có giả thuyết chưa kiểm chứng; có thể tạo entry `needs-review` nếu vẫn có giá trị, nhưng phải ghi rõ bằng chứng thiếu.

## Quy trình tìm trước
1. Tạo 2–4 cụm từ khóa từ triệu chứng, error message, subsystem, nguyên nhân và hành vi mong muốn.
2. Tìm memory project trước, rồi global. Trước tiên bỏ qua archive; tìm archive khi không có kết quả đủ tốt.
3. Đọc toàn bộ entry ứng viên; xác minh `status`, scope, stack/version, `verified_on`, source và điều kiện áp dụng.
4. So sánh với code, test và tài liệu hiện tại. Không áp dụng memory chỉ vì từ khóa giống nhau.

## Quy trình đúc kết
1. Chỉ lưu nội dung có giá trị dùng lại: root cause khó nhận biết, bug tái diễn, quyết định, cạm bẫy dependency/môi trường hoặc workaround có điều kiện.
2. Tìm entry hiện có trước; cập nhật entry đó nếu cùng vấn đề. Tránh tạo memory gần trùng lặp.
3. Dùng `.agents/template/memory/memory-entry.md`; ghi summary, điều kiện áp dụng, bằng chứng, root cause/decision, resolution, verification và liên kết nguồn.
4. Đặt `status: active` chỉ khi đã kiểm chứng; nếu còn phần quan trọng chưa xác minh, dùng `needs-review` và ghi rõ.
5. Cập nhật `updated`/`verified_on` theo sự kiện thực; không sửa ngày để làm entry trông mới.
6. Nếu entry cũ không còn đúng, đánh dấu `superseded` và liên kết entry thay thế. Chỉ archive khi cần loại khỏi tìm kiếm thường nhật.
7. Cập nhật `memory/INDEX.md` khi entry có giá trị cao hoặc hay cần tra cứu; không biến index thành bản sao toàn bộ corpus.
8. Không ghi secret, dữ liệu cá nhân, token, raw log nhạy cảm hoặc thông tin không được phép lưu.

## Đánh giá chất lượng entry
- Có thể hiểu được mà không cần đọc toàn bộ hội thoại gốc.
- Nêu rõ áp dụng khi nào và không áp dụng khi nào.
- Có bằng chứng/nguồn và bước xác minh tái lặp được.
- Có metadata hợp lệ và không trùng entry khác.

## Điều kiện hoàn tất
- Memory có giá trị đã được tạo/cập nhật ở đúng scope hoặc đã xác định có lý do để không lưu; không tuyên bố hoàn thành nếu file chưa được ghi.

## Template liên quan
- `.agents/template/memory/memory-entry.md`
- `.agents/template/software/memory-evaluation.md`
