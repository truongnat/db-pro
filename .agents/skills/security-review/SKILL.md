---
name: security-review
description: "Đánh giá rủi ro bảo mật cho thay đổi liên quan xác thực, phân quyền, dữ liệu nhạy cảm, upload, file, tích hợp bên ngoài, secret hoặc xử lý đầu vào không tin cậy."
---
# Review bảo mật

## Khi nào dùng
- Thay đổi authn/authz, session/token, tenant isolation, quyền truy cập dữ liệu, upload/download, webhook, secret, logging hoặc dependency có bề mặt tấn công.
- Trước release khi mức độ rủi ro yêu cầu kiểm tra bảo mật riêng.

## Không dùng khi
- Thay đổi thuần nội dung không có dữ liệu/luồng quyền tin cậy liên quan; áp dụng rule bảo mật chung vẫn bắt buộc.

## Quy trình
1. Đọc `.agents/rules/security.md`, yêu cầu và phần code liên quan.
2. Xác định tài sản cần bảo vệ, actor/đối tượng tấn công, trust boundary, dữ liệu nhạy cảm và quyền cần được áp dụng.
3. Kiểm tra xác thực và phân quyền ở phía server, kiểm tra quyền trên từng tài nguyên, tenant boundary và fail-closed behavior.
4. Kiểm tra validation/normalization input, injection, file/path handling, SSRF hoặc deserialization khi phù hợp với luồng thực tế.
5. Kiểm tra secret management, log/telemetry, error response, CORS/CSRF/cookie/token policy và dependency theo bề mặt liên quan.
6. Với thay đổi schema/permission, kiểm tra dữ liệu cũ, default permission, backfill và khả năng rollback.
7. Ghi findings có vị trí, điều kiện, tác động và mitigation. Nếu cần lưu tài liệu, dùng template security review.
8. Kiểm thử các case từ chối truy cập và input không hợp lệ; không chỉ kiểm tra happy path.

## Quy tắc
- Không đưa secret thật vào prompt, log, tài liệu hoặc fixture.
- Không coi ẩn nút trên UI là cơ chế phân quyền.
- Không chạy kiểm thử xâm nhập trên hệ thống không được phép.
- Không tuyên bố hệ thống an toàn tuyệt đối; chỉ nêu phạm vi đã kiểm tra và rủi ro còn lại.

## Điều kiện hoàn tất
- Không còn finding nghiêm trọng chưa xử lý hoặc chưa được người có thẩm quyền chấp nhận; phạm vi và giới hạn review rõ ràng.

## Template liên quan
`.agents/template/software/security-review.md`
