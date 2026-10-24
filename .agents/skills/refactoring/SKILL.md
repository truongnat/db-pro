---
name: refactoring
description: "Tái cấu trúc code nhằm tăng tính rõ ràng, giảm coupling hoặc cải thiện khả năng bảo trì mà không cố ý thay đổi hành vi bên ngoài. Dùng khi người dùng yêu cầu refactor, cleanup hoặc giảm nợ kỹ thuật."
---
# Refactor an toàn

## Khi nào dùng
- Tách module, cải thiện abstraction, loại bỏ duplication, đổi cấu trúc code hoặc giảm technical debt.

## Không dùng khi
- Thay đổi hành vi nghiệp vụ là mục tiêu chính → `requirements-discovery` và `implementation`.
- Chưa hiểu nguyên nhân lỗi → `bug-investigation`.

## Quy trình
1. Tìm memory, convention và test hiện có; xác định hành vi cần giữ nguyên.
2. Xác định vấn đề cấu trúc cụ thể cần cải thiện và cách đánh giá kết quả.
3. Chạy baseline tests/benchmark phù hợp trước khi đổi nếu có thể.
4. Chia refactor thành bước nhỏ, mỗi bước có thể review và kiểm chứng.
5. Giữ nguyên public contract, error semantics, ordering, persistence behavior và side effect trừ khi có yêu cầu thay đổi rõ.
6. Tránh kết hợp refactor rộng với thay đổi tính năng khiến khó phân biệt regression.
7. Chạy `testing-validation`, review diff và so sánh hành vi trước/sau.
8. Ghi ADR hoặc memory chỉ khi lựa chọn/pattern có giá trị lâu dài.

## Quy tắc
- “Code đẹp hơn” không đủ nếu tăng độ phức tạp hoặc thêm abstraction không cần thiết.
- Không xóa test chỉ vì implementation mới khiến test cũ thất bại; xác định test đang bảo vệ behavior hay chi tiết implementation.
- Nếu phát hiện cần đổi behavior, tách thành quyết định/commit/phần việc rõ ràng.

## Điều kiện hoàn tất
- Cấu trúc được cải thiện theo mục tiêu đã nêu, behavior được bảo toàn qua kiểm chứng và diff không trộn thay đổi ngoài phạm vi.
