# Quy chuẩn ngôn ngữ

## Mặc định

- Ngôn ngữ diễn giải mặc định của bộ `.agents/` là **tiếng Việt**.
- Viết bằng tiếng Việt cho hướng dẫn, checklist, ghi chú điều tra, giải thích quyết định, kết luận kiểm chứng và nội dung memory do AI tạo.
- Tên file/thư mục, tên trường metadata, enum trong dữ liệu có cấu trúc, ID, command, tên API, class, function, biến và ký hiệu kỹ thuật phải giữ nguyên dạng chuẩn tiếng Anh.
- Không dịch hoặc tự ý đổi các giá trị có cấu trúc như `status: active`, `type: bugfix`, `scope: project` hay `schema_version: 1`.
- Ngược lại, nhãn lựa chọn dành cho con người trong template (ví dụ trạng thái công việc, mức độ rủi ro, `Yes/No`) phải dùng tiếng Việt, trừ khi chúng được đọc trực tiếp bởi code hoặc automation.
- Giữ nguyên thông báo lỗi, code, đường dẫn, tên cấu hình và lệnh CLI khi trích dẫn; phần giải thích xung quanh viết bằng tiếng Việt.
- Có thể giữ thuật ngữ chuyên ngành tiếng Anh khi cần độ chính xác. Khi thuật ngữ dễ gây hiểu nhầm, giải thích ngắn bằng tiếng Việt ở lần xuất hiện đầu tiên.

## Ngoại lệ theo project

- `rules/project.md` có thể khai báo ngôn ngữ tài liệu riêng của repository, ví dụ tiếng Anh cho tài liệu kỹ thuật.
- Ngoại lệ chỉ áp dụng trong phạm vi project và phải được ghi rõ; không tự suy ra ngoại lệ chỉ vì codebase có identifier tiếng Anh.
- Nội dung người dùng hoặc khách hàng cung cấp cần được giữ theo ngôn ngữ nguồn khi việc dịch có thể làm thay đổi ý nghĩa.

## Tự kiểm tra trước khi lưu

- [ ] Phần diễn giải dùng một ngôn ngữ nhất quán theo quy tắc trên.
- [ ] Không dịch tên trường, enum, API, code hoặc lệnh.
- [ ] Không trộn hai ngôn ngữ trong cùng câu nếu không có lý do kỹ thuật.
- [ ] Ví dụ, tiêu đề và placeholder hướng dẫn người dùng viết nội dung bằng ngôn ngữ mặc định.
