# Danh sách kiểm tra phát hành: <bản phát hành/thay đổi>

- **Version/build:**
- **Môi trường đích:**
- **Người phụ trách phát hành:**
- **Thời gian dự kiến:** YYYY-MM-DD HH:mm TZ

## Trước phát hành

- [ ] Tiêu chí chấp nhận và test quan trọng đã đạt.
- [ ] Build/artifact/cấu hình đã được review.
- [ ] Đã kiểm tra migration và tương thích dữ liệu.
- [ ] Đã kiểm tra feature flag, quyền, secret và chính sách truy cập.
- [ ] Đã cập nhật changelog/runbook/ghi chú hỗ trợ.
- [ ] Đã xác nhận phương án backup hoặc khôi phục khi cần.

## Triển khai

- [ ] Các bước deploy:
- [ ] Ràng buộc thứ tự/migration:
- [ ] Smoke test:
- [ ] Health indicator/log/metric cần theo dõi:

## Rollback

- **Điều kiện kích hoạt:**
- **Các bước rollback:**
- **Lưu ý về dữ liệu/schema:**
- **Người quyết định:**

## Sau phát hành

- [ ] Đã kiểm tra luồng người dùng cốt lõi.
- [ ] Đã kiểm tra error rate/latency/tín hiệu nghiệp vụ.
- [ ] Đã ghi lại lỗi đã biết và người phụ trách theo dõi.
- [ ] Cập nhật memory nếu release phát hiện kiến thức vận hành có thể tái sử dụng.
