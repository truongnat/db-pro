# Quy chuẩn memory

## Nội dung nào đáng lưu?

Tạo hoặc cập nhật entry khi có ít nhất một điều sau:

- Đã xác nhận một nguyên nhân gốc khó nhận biết và cách sửa có khả năng tái sử dụng.
- Có một dạng lỗi hoặc workaround nhiều khả năng tái diễn.
- Có quyết định kiến trúc/sản phẩm tạo ảnh hưởng lâu dài.
- Một đặc điểm của framework, hạ tầng hoặc repository đã khiến việc điều tra tốn công đáng kể.
- Một quy trình đã được kiểm chứng và đáng lặp lại.

**Không** tạo entry cho chỉnh sửa thường ngày, thông tin hiển nhiên đã có ở nơi khác, phỏng đoán chưa xác minh, kết quả chỉ dùng một lần hoặc bản tóm tắt tác vụ chung chung không có bài học tái sử dụng.

## Tiêu chuẩn chất lượng bắt buộc

Mỗi memory ở trạng thái `active` phải trả lời được:

1. **Khi nào kiến thức này hữu ích?** Triệu chứng, điều kiện kích hoạt, phạm vi, môi trường và phiên bản.
2. **Đã thực sự biết được điều gì?** Nguyên nhân/quyết định đã xác nhận; phân biệt sự thật với giả thuyết.
3. **Nên làm gì?** Các bước ngắn gọn, cụ thể cùng phương án thay thế/rủi ro quan trọng.
4. **Bằng chứng là gì?** Test/lệnh/bằng chứng quan sát được và kết quả.
5. **Nguồn ở đâu?** Đường dẫn file, issue/PR/commit, tài liệu hoặc nguồn bền vững khác.

Nếu chưa trả lời được, đặt `status: needs-review` hoặc chưa lưu entry.

## Tên file và metadata

- Tên file: `YYYY-MM-DD-<type>-<short-topic>.md`.
- Mỗi file chỉ tập trung vào một chủ đề; ưu tiên cập nhật entry có sẵn thay vì tạo bản trùng lặp.
- Các trường frontmatter bắt buộc: `schema_version`, `id`, `title`, `type`, `scope`, `status`, `created`, `updated`, `verified_on`, `confidence`, `tags`, `applies_to`, `sources`, `related`, `supersedes`.
- Các giá trị `type` được phép: `bugfix`, `decision`, `pattern`, `workflow`, `environment`.
- Các giá trị `scope` được phép: `global`, `project`.
- Các giá trị `status` được phép: `active`, `needs-review`, `superseded`, `archived`.
- Các giá trị `confidence` được phép: `high`, `medium`, `low`.
- Ngày dùng định dạng ISO `YYYY-MM-DD`. Dùng `schema_version` khi cấu trúc template thay đổi; dùng Git để lưu lịch sử nội dung, không tự quản lý `v1`, `v2` trong mọi tên file.

## Vòng đời

- `active`: bằng chứng đã được kiểm tra và entry áp dụng trong phạm vi đã khai báo.
- `needs-review`: có gợi ý hữu ích nhưng chưa xác minh đủ hoặc chưa chắc còn áp dụng.
- `superseded`: entry mới thay thế entry cũ; cần liên kết đến entry thay thế.
- `archived`: giữ để tra cứu lịch sử nhưng loại khỏi tìm kiếm mặc định.
- Khi dependency, môi trường hoặc giả định kiến trúc thay đổi, rà soát các entry bị ảnh hưởng.
- Chỉ chuyển entry vào `archive/YYYY/` khi đã lỗi thời, bị thay thế hoặc không còn hữu ích trong công việc thường ngày; không archive chỉ vì thư mục lớn.

## Bảo mật và quyền riêng tư

Không lưu credential, secret, private key, access token, dữ liệu cá nhân khách hàng hoặc log nhạy cảm nguyên bản. Che thông tin trong ví dụ và dùng giá trị giả lập. File Markdown cục bộ vẫn là kho dữ liệu: áp dụng chính sách truy cập và repository như với code.

## Tìm kiếm và loại trùng lặp

- Tìm theo error/symbol chính xác, triệu chứng, subsystem/file và nguyên nhân có thể có.
- Tìm memory của project trước memory global.
- Đọc metadata và phần giải thích xung quanh trước khi áp dụng kết quả.
- Nếu phát hiện mới trùng chủ đề với entry hiện có, cải thiện entry đó và cập nhật `updated`/`verified_on` thay vì tạo file khác.
- Tránh tạo `INDEX.md` khổng lồ. Index chỉ là trang điều hướng cho entry giá trị cao; toàn bộ corpus phải có thể tìm tự động.
