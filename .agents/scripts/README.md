# Script

Script cần có hành vi xác định, tham số rõ ràng, báo lỗi cụ thể khi đầu vào không hợp lệ và mô tả tác động phụ. Review lệnh trước khi chạy. `search-memory.sh` là công cụ tìm kiếm chỉ đọc.

| Script | Mục đích |
|---|---|
| `search-memory.sh` | Tìm project memory → global |
| `build-knowledge-inventory.py` | **Master inventory** repo + user home |
| `migrate-plans-to-memory.py` | Mirror excerpt v1 (197 plan units) |
| `migrate-pending-knowledge.py` | Mirror **pending** repo files (full text) |
| `reconcile-memory-migration.py` | Đối soát v1 excerpt hashes |
| `validate-skills.sh` | Kiểm tra frontmatter `SKILL.md` + tham chiếu `template/` |
| `validate-skill-references.py` | Skill → `.agents/template/*` phải tồn tại |
| `classify-knowledge-groups.py` | Nhóm nguồn + durable + disposition; `GROUP_RECONCILIATION.md` |
