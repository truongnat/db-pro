# Quy trình làm việc của Agent

Trước khi tạo hoặc cập nhật tài liệu/memory, phải áp dụng `rules/language.md` trong cùng bộ `.agents` (dùng đường dẫn tương đối từ file hướng dẫn này).

## 1. Trước khi sửa code: tìm memory trước

Với mọi bug, feature, refactor, migration, thay đổi kiến trúc hoặc tooling không tầm thường:

1. Tóm tắt vấn đề thành các từ khóa tìm kiếm: triệu chứng/lỗi, subsystem/file, nguyên nhân có thể có và hành vi mong muốn.
2. Tìm **memory của project trước**, sau đó mới tìm memory global. Nếu lượt đầu ít kết quả, thử ít nhất hai cách diễn đạt, ví dụ chuỗi lỗi chính xác và tên subsystem/triệu chứng.
3. Đọc toàn bộ entry phù hợp, không chỉ xem snippet. Ưu tiên entry có `status: active`, stack/version tương thích và ngày xác minh gần nhất.
4. Đối chiếu hướng dẫn với branch, code, cấu hình, phiên bản dependency và test hiện tại. Memory là bằng chứng tham khảo, không phải chân lý tuyệt đối.
5. Nếu không có kết quả phù hợp, tiếp tục điều tra bình thường. Không bịa kết quả hoặc tuyên bố đã tìm kiếm nếu chưa thực sự chạy tìm kiếm.

### Lệnh tìm kiếm

Chạy từ bất kỳ thư mục nào bên trong repository:

```bash
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
search_script="$repo_root/.agents/scripts/search-memory.sh"
if [[ ! -x "$search_script" ]]; then search_script="$HOME/.agents/scripts/search-memory.sh"; fi

# Tìm memory project trước, sau đó đến global.
"$search_script" "refresh token loop"

# Chỉ tìm archive khi corpus hiện hành không có kết quả hữu ích.
"$search_script" --archive "refresh token loop"
```

Nếu không có script, dùng `rg -n -i "term" .agents/memory "$HOME/.agents/memory"` và loại `archive/` khỏi lượt tìm đầu tiên.

## 2. Trong quá trình thực hiện

- Xác nhận yêu cầu liên quan và tiêu chí chấp nhận trước khi thay đổi lớn.
- Kiểm tra kiến trúc và pattern hiện có trước khi thêm abstraction hoặc dependency.
- Ưu tiên thay đổi nhỏ nhất nhưng giải quyết trọn vẹn nguyên nhân đã xác nhận.
- Không trình bày giả định như sự thật; ghi rõ điều chưa chắc chắn và xác minh khi có thể.
- Không sao chép máy móc giải pháp từ memory. Kiểm tra tính phù hợp và phiên bản liên quan.
- Không ghi secret, credential, private key, dữ liệu cá nhân hoặc log nhạy cảm chưa được che thông tin vào memory.

## 3. Trước khi hoàn thành: kiểm chứng và đúc kết

1. Chạy test, type check, lint, build hoặc kiểm tra thủ công có thể tái hiện phù hợp nhất. Báo rõ đã chạy và chưa chạy những gì.
2. Xem lại diff; bảo đảm không có thay đổi ngoài phạm vi hoặc dữ liệu nhạy cảm.
3. Đánh giá xem công việc có tạo ra kiến thức lâu dài hay không. Chỉ tạo/cập nhật memory khi đáp ứng `rules/memory-policy.md`.
4. Nếu chủ đề đã tồn tại, cập nhật entry hiện có hoặc đánh dấu entry cũ là `superseded` thay vì tạo bản gần trùng lặp.
5. Chỉ đặt `status: active` khi giải pháp/bằng chứng đã được xác minh. Nếu chưa đủ, dùng `needs-review` và nêu rõ phần còn thiếu.
6. Chỉ cập nhật `memory/INDEX.md` cho entry có giá trị cao hoặc thường xuyên cần tra cứu; toàn bộ entry vẫn được tìm qua script.
7. Khi bàn giao, tóm tắt phần triển khai, kiểm chứng, giới hạn và memory đã tạo/cập nhật (nếu có). Không tuyên bố đã lưu memory nếu file thực tế chưa được ghi.

## 4. Ưu tiên memory và xử lý xung đột

- Memory của project cụ thể hơn memory global đối với hành vi riêng của project.
- Code hiện tại, test, tài liệu chính thức và hành vi runtime quan sát được có độ tin cậy cao hơn memory đã lỗi thời.
- Nếu memory mâu thuẫn với bằng chứng hiện tại, hãy sửa/thay thế entry và cập nhật ngày xác minh.
- Memory không được ghi đè yêu cầu hiện tại của người dùng hoặc chỉ dẫn bảo mật/an toàn có mức ưu tiên cao hơn.

## 5. Quy chuẩn ngôn ngữ và template

- Tuân thủ `rules/language.md` cho mọi tài liệu và memory mới.
- Dùng số lượng template tương xứng với độ phức tạp. Sửa lỗi nhỏ không cần tài liệu đặc tả và thiết kế đầy đủ; feature ảnh hưởng nhiều thành phần, migration rủi ro hoặc incident production nên dùng template phù hợp.
