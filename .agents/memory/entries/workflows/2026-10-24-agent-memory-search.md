---
schema_version: 1
id: "MEM-20261024-agent-memory-search"
title: "Quy trình agent: tìm project memory trước global trước khi sửa code"
type: workflow
scope: project
status: active
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "2026-10-24"
confidence: high
tags: [agent, memory, workflow]
applies_to:
  repo: "db-pro"
  stack: "any"
  versions: "agents-memory blueprint"
sources:
  - ".agents/AGENTS.md"
  - ".agents/rules/memory-policy.md"
related: []
supersedes: null
---

# Tóm tắt

Với bug/feature/refactor không tầm thường: chạy `search-memory.sh`, đọc entry `active`, đối chiếu code hiện tại, rồi mới implement. Chỉ ghi memory khi đáp ứng memory-policy.

## Khi nào áp dụng

- Mọi phiên agent trên repo này (Cursor, Orca, CLI).

## Cách xử lý / quy trình đề xuất

1. Tóm tắt từ khóa: lỗi, subsystem, file.
2. `.agents/scripts/search-memory.sh "<query>"` — project `.agents/memory` trước `~/.agents/memory`.
3. Nếu trống: `--archive` hoặc diễn đạt khác; không bịa đã tìm.
4. Trước handoff: kiểm chứng test/lint; cập nhật memory nếu có bài học tái sử dụng.

## Kiểm chứng

- Script exit 0; kết quả rg có đường dẫn entry.

## Lịch sử thay đổi

- 2026-10-24 — Áp dụng blueprint `agents-memory` vào db-pro.
