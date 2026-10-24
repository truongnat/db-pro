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

| `purge-orphan-mirror-files.py` | Xóa file mirror không khớp `mirror_filename` nguồn (legacy) |
| `check-mirror-filename-collisions.py` | Fail nếu hai nguồn trùng tên mirror |

**Bootstrap (sau clone):** `scripts/bootstrap-agents-memory.sh` từ repo root — purge + collision check + full §9 + `validate-skills.sh`.

**Policy / verify:** `docs/ARTIFACT_POLICY.md`, `scripts/verify-artifact-policy.sh`.
