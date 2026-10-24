---
name: handoff-retrospective
description: "Bàn giao task chưa hoàn tất giữa phiên/agent/người làm và tổng kết cải tiến sau feature, release hoặc incident. Dùng khi cần tiếp tục công việc an toàn hoặc rút bài học hành động được."
---
# Bàn giao và retrospective

## Khi nào dùng
- Task chưa hoàn tất, đổi phiên/agent, cần tiếp tục điều tra hoặc bàn giao cho người khác.
- Sau feature lớn, thay đổi phức tạp, release hoặc incident khi tổng kết đem lại cải tiến cụ thể.

## Không dùng khi
- Task nhỏ đã hoàn tất, không có phần dang dở hoặc bài học đáng lưu; chỉ cần summary ngắn.

## Quy trình bàn giao
1. Ghi mục tiêu, phạm vi, trạng thái hiện tại và thay đổi đã thực hiện.
2. Nêu file/đường dẫn quan trọng, lệnh đã chạy và kết quả thực tế.
3. Nêu điều chưa xong, blocker, giả định, rủi ro và bước tiếp theo có thể hành động ngay.
4. Liệt kê các thay đổi chưa commit hoặc file người dùng đã sửa mà người tiếp theo phải giữ nguyên.
5. Ghi memory/ADR liên quan nếu có, không sao chép nguyên memory vào handoff.
6. Dùng `.agents/template/software/handoff.md` khi người tiếp theo cần tiếp tục không đồng bộ.

## Quy trình retrospective
1. Dựa trên dữ kiện: mục tiêu, kết quả, sai lệch, thời gian chờ, bug, test và rollout.
2. Ghi điều hiệu quả, điều chưa hiệu quả và nguyên nhân hệ thống; tránh quy trách nhiệm cá nhân.
3. Chuyển bài học thành action item cụ thể có owner, mức ưu tiên và điều kiện hoàn tất.
4. Tạo retrospective khi có giá trị; đưa kiến thức bền vững sang memory thay vì lưu nhật ký dài hạn.

## Quy tắc
- Không ghi “đã xong” nếu còn bước chưa xác minh.
- Không giả định agent/phiên kế tiếp có thể truy cập hội thoại hiện tại; bàn giao đủ context cần thiết.
- Không đưa secret vào handoff.

## Điều kiện hoàn tất
- Người nhận biết chính xác trạng thái, việc tiếp theo và các rủi ro; retrospective có hành động theo dõi rõ ràng.

## Template liên quan
- `.agents/template/software/handoff.md`
- `.agents/template/software/retrospective.md`
