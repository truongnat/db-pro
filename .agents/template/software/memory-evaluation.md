# Đánh giá khả năng truy xuất memory

- **Corpus / commit:**
- **Phương pháp tìm kiếm:** `rg` | script | hybrid | other
- **Ngày đánh giá:** YYYY-MM-DD
- **Người đánh giá:**

## Các case benchmark

Chuẩn bị 10–20 task/câu hỏi thực tế đã gặp. Ghi entry kỳ vọng trước khi chạy tìm kiếm để tránh đánh giá theo kết quả đã thấy.

| Case | Truy vấn/task | Memory ID/path kỳ vọng | Trong top 5? | Đã mở kết quả hữu ích? | Áp dụng đúng? | Kết quả |
|---|---|---|---|---|---|---|
| E-01 | | | Có/Không | Có/Không | Có/Không/Không áp dụng | |

## Chỉ số

- **Recall@5:** số entry kỳ vọng liên quan xuất hiện trong top 5 / số case có memory liên quan = ___ / ___ = ___%
- **Precision@5:** số kết quả hữu ích trong các kết quả top 5 đã xem / tổng kết quả đã xem = ___ / ___ = ___%
- **Adoption rate:** số task liên quan mà memory được áp dụng đúng / tổng task liên quan = ___ / ___ = ___%
- **Stale-memory rate:** số entry được truy xuất nhưng mâu thuẫn với bằng chứng hiện tại / tổng entry đã kiểm tra = ___ / ___ = ___%
- **Trùng lặp/xung đột phát hiện được:**
- **Thời gian tìm / thời gian sửa:**

## Phát hiện và hành động tiếp theo

- Lỗi truy xuất chính:
- Vấn đề chất lượng nội dung chính:
- Thay đổi taxonomy/tag/index nên thử trước:
- Bằng chứng cần có trước khi dùng embedding/vector search/database:
