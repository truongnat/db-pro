---
schema_version: 1
id: "MEM-YYYYMMDD-short-topic"
title: "Một câu ngắn gọn, rõ ràng và dễ tìm kiếm"
type: bugfix # bugfix | decision | pattern | workflow | environment
scope: project # project | global
status: needs-review # active | needs-review | superseded | archived
created: "YYYY-MM-DD"
updated: "YYYY-MM-DD"
verified_on: "YYYY-MM-DD" # dùng "unknown" nếu chưa xác minh
confidence: medium # high | medium | low
tags: [subsystem, symptom, technology]
applies_to:
  repo: "tên repository hoặc any"
  stack: "framework/runtime"
  versions: "phạm vi phiên bản đã biết hoặc unknown"
sources:
  - "path/to/file.ts"
  - "URL hoặc ID của PR/issue/commit/tài liệu"
related: []
supersedes: null
---

# Tóm tắt

Nêu bài học có thể tái sử dụng trong 1–3 câu. Kỹ sư/agent tương lai cần biết gì trước khi thực hiện công việc này?

## Khi nào áp dụng

- Triệu chứng, điều kiện kích hoạt, subsystem bị ảnh hưởng và môi trường/phiên bản liên quan.
- Điều kiện mà hướng dẫn này **không** áp dụng.

## Bối cảnh và bằng chứng

- **Quan sát được:** điều đã thấy, kèm đường dẫn file/error/metric khi phù hợp.
- **Đã xác nhận:** sự thật được hỗ trợ bởi việc tái hiện, test, source hoặc bằng chứng runtime.
- **Giả thuyết:** ghi rõ điều chưa xác nhận hoặc bỏ qua nếu không hữu ích.

## Nguyên nhân gốc / quyết định

Giải thích cơ chế gây ra vấn đề hoặc quyết định quan trọng. Không chỉ lặp lại mô tả tác vụ.

## Cách xử lý / quy trình đề xuất

1. Bước hoặc thay đổi cần thực hiện.
2. Bước hoặc thay đổi cần thực hiện.
3. Lưu ý quan trọng về tương thích, giới hạn hoặc rollback.

## Kiểm chứng

- **Lệnh/kịch bản:** `lệnh chính xác hoặc cách kiểm tra có thể lặp lại`
- **Kỳ vọng:** kết quả thành công cần có.
- **Kết quả:** pass/fail/chưa chạy, kèm ngày và phiên bản liên quan.

## Phương án thay thế và cạm bẫy

- Phương án đã thử/cân nhắc và lý do loại bỏ, nếu thông tin này giúp tránh lặp lại công sức vô ích.
- Giới hạn đã biết hoặc trường hợp giải pháp không an toàn/không phù hợp.

## Lịch sử thay đổi

- YYYY-MM-DD — Tạo mới hoặc cập nhật đáng kể; nêu nội dung đã kiểm chứng lại.
