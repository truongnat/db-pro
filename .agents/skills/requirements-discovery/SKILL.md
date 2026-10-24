---
name: requirements-discovery
description: "Làm rõ vấn đề, người dùng bị ảnh hưởng, phạm vi, ràng buộc và acceptance criteria trước khi triển khai feature hoặc thay đổi hành vi. Dùng khi yêu cầu chưa rõ, feature mới hoặc thay đổi có ảnh hưởng đến nghiệp vụ."
---
# Khám phá yêu cầu

## Khi nào dùng
- Feature mới, thay đổi hành vi nghiệp vụ hoặc yêu cầu có nhiều cách hiểu.
- Cần xác nhận phạm vi, ngoại lệ, dữ liệu đầu vào/đầu ra hoặc tiêu chí nghiệm thu.

## Không dùng khi
- Sửa lỗi đã tái hiện rõ và yêu cầu sửa đã cụ thể → `bug-investigation`.
- Task kỹ thuật rất nhỏ, rủi ro thấp và không làm thay đổi hành vi người dùng.

## Quy trình
1. Đọc yêu cầu gốc và tìm memory liên quan trước khi chốt hướng giải quyết.
2. Kiểm tra hành vi hiện tại trong code, UI, API, tài liệu và test nếu có.
3. Xác định: vấn đề cần giải quyết, nhóm người dùng/actor, luồng chính, trạng thái, dữ liệu, quy tắc nghiệp vụ, ngoại lệ, phạm vi không làm.
4. Viết acceptance criteria theo hành vi có thể kiểm tra; ưu tiên Given/When/Then khi phù hợp.
5. Ghi giả định và câu hỏi mở. Chỉ hỏi người dùng khi câu trả lời có thể đổi đáng kể thiết kế, phạm vi hoặc rủi ro; còn lại ghi giả định rõ ràng.
6. Dùng `.agents/template/software/feature-spec.md` nếu feature đủ lớn để cần một đặc tả có thể review.
7. Chuyển sang `solution-design` khi cần quyết định kiến trúc hoặc ảnh hưởng nhiều thành phần.

## Quy tắc
- Phân biệt yêu cầu đã xác nhận với giả định.
- Không biến giải pháp do người dùng gợi ý thành yêu cầu bắt buộc nếu mục tiêu thực sự chưa rõ.
- Xác định cả trạng thái lỗi, loading/empty state, phân quyền và khả năng retry/reload khi liên quan.
- Không viết đặc tả dài hơn mức cần để implement và kiểm thử.

## Điều kiện hoàn tất
- Có mục tiêu rõ, phạm vi/ngoài phạm vi, acceptance criteria và ràng buộc quan trọng.
- Điểm chưa rõ còn lại được ghi rõ hoặc đã hỏi đúng câu hỏi cần thiết.

## Template liên quan
`.agents/template/software/feature-spec.md`
