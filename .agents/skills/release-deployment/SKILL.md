---
name: release-deployment
description: "Chuẩn bị phát hành và triển khai an toàn: kiểm tra thay đổi, migration, cấu hình, rollout, smoke test, giám sát và rollback. Dùng trước khi phát hành hoặc deploy môi trường dùng chung/production."
---
# Chuẩn bị release và triển khai

## Khi nào dùng
- Chuẩn bị tag/release/deploy, thay đổi schema hoặc cấu hình runtime, rollout feature flag hay cần kế hoạch rollback.

## Không dùng khi
- Chỉ sửa code local chưa tới bước phát hành.
- Đang xảy ra incident cần giảm tác động ngay → `incident-response`.

## Quy trình
1. Xác định môi trường, phạm vi release, commit/build artifact và người có thẩm quyền phê duyệt.
2. Đọc diff, kết quả test, security findings, feature flag và compatibility requirement.
3. Kiểm tra biến môi trường/config, secret reference, dependency, health check và observability.
4. Với migration, xác nhận thứ tự rollout, tương thích code cũ/mới, backfill, lock/downtime và cách rollback/forward-fix.
5. Xây dựng rollout steps, smoke test, tín hiệu theo dõi, ngưỡng dừng và rollback plan.
6. Dùng `release-checklist.md` cho release có ảnh hưởng đáng kể.
7. Chỉ thực hiện thao tác deploy thật khi môi trường, quyền và ủy quyền đã rõ; nếu không, dừng ở bước chuẩn bị và báo lại.
8. Sau deploy, kiểm tra health, log/metrics, luồng nghiệp vụ quan trọng và lỗi tăng bất thường.
9. Ghi lại kết quả phát hành, sai lệch và memory có thể tái sử dụng.

## Quy tắc
- Không deploy production, xóa dữ liệu hoặc chạy migration phá hủy khi chưa có phê duyệt rõ ràng.
- Không coi rollback code là đủ nếu schema/data đã thay đổi không tương thích.
- Phải có cách nhận biết release xấu và điều kiện dừng; không chỉ ghi “monitor logs”.

## Điều kiện hoàn tất
- Release có kiểm tra trước/sau, phương án ứng phó và trạng thái triển khai được báo chính xác.

## Template liên quan
`.agents/template/software/release-checklist.md`
