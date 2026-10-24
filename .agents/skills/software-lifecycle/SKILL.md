---
name: software-lifecycle
description: "Điều phối vòng đời phát triển phần mềm từ phân loại yêu cầu đến triển khai, kiểm chứng, review, release và đúc kết memory. Dùng khi người dùng yêu cầu xây feature, sửa lỗi, refactor, thay đổi kiến trúc hoặc hoàn thành một task phần mềm nhiều bước."
---
# Điều phối vòng đời phát triển phần mềm

## Khi nào dùng
- Task có nhiều bước hoặc có thể ảnh hưởng đến nhiều thành phần.
- Cần chọn đúng skill và template mà không nạp toàn bộ quy trình không cần thiết.

## Không dùng khi
- Câu hỏi thuần giải thích, không yêu cầu thay đổi repository.
- Một chỉnh sửa cực nhỏ đã có quy trình rõ; vẫn tuân thủ memory search, kiểm chứng và review diff phù hợp.

## Quy trình
1. Đọc `.agents/AGENTS.md`, rule áp dụng và `.agents/skills/README.md`.
2. Phân loại mức độ: **nhỏ** (đổi cục bộ, rủi ro thấp), **vừa** (nhiều file hoặc thêm hành vi), **lớn/rủi ro cao** (kiến trúc, auth, dữ liệu, migration, bảo mật, production).
3. Với task kỹ thuật đáng kể, tìm memory trước theo `.agents/rules/memory-policy.md`; tìm project trước, global sau. Xác minh entry với code và phiên bản hiện tại.
4. Chọn skill theo bảng trong `.agents/skills/README.md`. Chỉ nạp các skill liên quan; có thể xếp chuỗi nhiều skill theo thứ tự công việc.
5. Chọn template tương xứng. Template là nguồn để khởi tạo tài liệu, không phải file cần chỉnh trực tiếp. Nếu tạo tài liệu, sao chép template vào vị trí tài liệu theo quy ước repo.
6. Thực hiện thay đổi theo phạm vi; không tự mở rộng yêu cầu khi chưa có lý do.
7. Chạy kiểm chứng phù hợp, xem diff, xử lý phát hiện quan trọng rồi quyết định có cần memory/handoff/release không.
8. Báo rõ kết quả, lệnh đã chạy, kiểm tra chưa chạy, giả định và rủi ro còn lại.

## Quy tắc chọn quy trình
- Bug → `bug-investigation` → `testing-validation` → `code-review` → `memory-curation` nếu có bài học tái sử dụng.
- Feature → `requirements-discovery` → `solution-design` khi cần → `implementation` → `testing-validation` → `security-review` nếu có bề mặt rủi ro → `code-review`.
- Refactor → `refactoring` → `testing-validation` → `code-review`.
- Release → `release-deployment`; khi có sự cố đang diễn ra, ưu tiên `incident-response`.
- Task chưa xong hoặc chuyển phiên → `handoff-retrospective`.

## Cổng chất lượng
- Không chuyển từ yêu cầu sang triển khai nếu mục tiêu/tiêu chí chấp nhận còn mơ hồ theo cách có thể làm thay đổi giải pháp đáng kể.
- Không coi task đã xong chỉ vì code đã sửa; cần có bằng chứng kiểm chứng phù hợp hoặc nêu giới hạn.
- Không chạy lệnh phá hủy, migration production hay thao tác triển khai có ảnh hưởng thật khi chưa có quyền/ủy quyền rõ ràng.
- Không bắt buộc tạo tài liệu đặc tả cho sửa đổi nhỏ; tránh quy trình hình thức không tạo giá trị.

## Đầu ra
Code/tài liệu cần thiết, bằng chứng kiểm chứng, tóm tắt diff và memory/handoff nếu phù hợp. Skill này điều phối; nó không thay thế các skill chuyên biệt.
