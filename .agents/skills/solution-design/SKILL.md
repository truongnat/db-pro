---
name: solution-design
description: "Thiết kế giải pháp kỹ thuật, luồng dữ liệu, API contract, thay đổi schema, trade-off và quyết định kiến trúc trước khi implement. Dùng cho feature nhiều thành phần, thay đổi kiến trúc, tích hợp hoặc migration có rủi ro."
---
# Thiết kế giải pháp

## Khi nào dùng
- Thay đổi nhiều module/service hoặc ranh giới giữa FE, API và dữ liệu.
- Thêm dependency, đổi data model, API contract, auth flow, async flow hoặc kiến trúc.
- Có nhiều phương án với trade-off đáng kể.

## Không dùng khi
- Sửa lỗi cục bộ hoặc thay đổi hiển nhiên không cần lựa chọn kiến trúc.

## Quy trình
1. Đọc memory/ADR hiện có và xác minh các quyết định liên quan còn đúng.
2. Khảo sát kiến trúc, pattern và contract đang dùng; tránh thiết kế dựa trên tên file mà chưa đọc code.
3. Xác định các thành phần, luồng request/event, nguồn dữ liệu, trạng thái, lỗi, timeout/retry, idempotency và ranh giới tin cậy khi liên quan.
4. So sánh tối thiểu phương án hợp lý, tiêu chí lựa chọn và trade-off. Không tạo bảng so sánh hình thức nếu chỉ có một phương án thực tế.
5. Dùng `technical-design.md` cho thiết kế ảnh hưởng nhiều thành phần. Dùng `adr.md` khi quyết định có giá trị dài hạn hoặc khó đảo ngược.
6. Dùng `implementation-plan.md` nếu cần chia thành bước nhỏ có thứ tự, dependency và tiêu chí xác minh.
7. Xác định chiến lược test, rollout, migration và rollback tương ứng với rủi ro.
8. Tiếp tục implement sau khi các quyết định trọng yếu đã rõ; task nhỏ không cần đợi tài liệu hình thức.

## Quy tắc
- Bám convention và dependency hiện có, trừ khi có bằng chứng cần thay đổi.
- Mọi đề xuất thay đổi contract phải xác định bên tiêu thụ và tương thích ngược.
- Với dữ liệu, phân biệt schema, dữ liệu hiện hữu, backfill và hành vi ứng dụng trong giai đoạn chuyển tiếp.
- Không tự thêm công nghệ/dependency chỉ để giải quyết vấn đề nhỏ.

## Điều kiện hoàn tất
- Luồng chính, failure mode, rủi ro, quyết định và cách kiểm chứng đã đủ rõ để triển khai.
- Mọi ADR cần thiết đã được ghi nhận hoặc lý do không tạo ADR được nêu rõ.

## Template liên quan
- `.agents/template/software/technical-design.md`
- `.agents/template/software/adr.md`
- `.agents/template/software/implementation-plan.md`
