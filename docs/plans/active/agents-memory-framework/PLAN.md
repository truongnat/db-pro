# Agents Memory Framework — Plan

State: **IMPLEMENTING** — repo inventory migration **done**; user-home **catalogued**; curation `active` là follow-up.

Branch: `main`

## Goal

**Migration hết** theo nghĩa có thể kiểm chứng:

1. Inventory **mọi** nguồn kiến thức text trong repo (và catalog user-level agent paths).
2. Mirror hoặc ghi rõ **không tồn tại** / **skipped** / **catalog only**.
3. `KNOWLEDGE_SOURCE_INVENTORY.json` → `repo_pending_count: 0` trước khi claim repo scope xong.

## Scope (đã mở rộng — không chỉ active/completed/_audit)

| Lớp | Xử lý |
|---|---|
| `docs/plans/**` (active, completed, archived, project, issue-ledger, audit, root) | v2 `by-source-file` + v1 plan units |
| `docs/**` (ngoài plans) | v2 full mirror |
| `.agents/**` (trừ mirror dirs) | v2 |
| `.skills/`, `.cursor/`, `AGENTS.md`, `RULES.md`, … | v2 |
| `docs/plans/_template/` | **skipped_template** (5) |
| `~/.agents/**`, `~/.cursor/skills` | **catalog** trong inventory (831 rows); không copy 804 skill vào repo |
| `_archive/`, binary | không quét |

Chi tiết số liệu: [`KNOWLEDGE_MIGRATION_STATUS.md`](KNOWLEDGE_MIGRATION_STATUS.md).

## Non-goals

- Import toàn bộ `~/.agents/skills` (804) vào repo.
- Đặt `status: active` hàng loạt cho mirror `needs-review`.
- Thay thế `docs/plans/` lifecycle.

## Acceptance criteria

- [x] `KNOWLEDGE_SOURCE_INVENTORY.json` với `repo_pending_count: 0`.
- [x] `build-knowledge-inventory.py` + `migrate-pending-knowledge.py` + v1 reconcile.
- [x] `KNOWLEDGE_MIGRATION_STATUS.md` + `evidence/skills-crosswalk.json`.
- [x] User home: catalogued; `user_cursor_skills` ghi `not_exists` nếu không có.
- [ ] Curated promotion (optional follow-up task).

## Provider matrix

N/A (infrastructure).
