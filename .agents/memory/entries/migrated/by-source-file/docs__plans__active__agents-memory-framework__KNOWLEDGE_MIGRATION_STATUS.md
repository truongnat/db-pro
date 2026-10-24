---
schema_version: 1
id: "MEM-SRC-dd938d1b829d6f0d"
title: "Source mirror: docs/plans/active/agents-memory-framework/KNOWLEDGE_MIGRATION_STATUS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/agents-memory-framework/KNOWLEDGE_MIGRATION_STATUS.md"
source_sha256: "dd938d1b829d6f0d773dd5b488c3c256d8b012a2b3e21ebe1b0d985ca6a114c6"
mirror_body_sha256: "dd938d1b829d6f0d773dd5b488c3c256d8b012a2b3e21ebe1b0d985ca6a114c6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Trạng thái migration kiến thức (master)

**Cập nhật:** 2026-10-24  
**Nguồn sự thật:** `.agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json` + `.agents/memory/KNOWLEDGE_RECONCILIATION_V2.json` (`reconciled_at_utc` — không dùng bản ghi reconcile cũ sau khi sửa file này).  
**Lệnh:** `build-knowledge-inventory.py` → `migrate-pending-knowledge.py` → **`reconcile-knowledge-sources-v2.py`** → `classify-knowledge-groups.py` → `migrate-pending-knowledge.py` → **`reconcile-knowledge-sources-v2.py`** (classify ghi lại `GROUP_RECONCILIATION.md`)

## Đối soát nhóm (bắt buộc đọc)

**[`GROUP_RECONCILIATION.md`](GROUP_RECONCILIATION.md)** + `.agents/memory/KNOWLEDGE_GROUP_RECONCILIATION.json`

- Mọi file text trong `docs/` (trừ `_archive`) có dòng inventory — **0 gap**.
- Số cũ **~327** `docs/plans/*.md` “ngoài v1” → nhóm `plans_checklist`, `plans_supplement`, `plans_other_md`, … disposition **`migrated_full_mirror`** (không phải out-of-scope).
- Số cũ **~1141** `docs/**/*.md` → **1237** file text quét filesystem (`KNOWLEDGE_GROUP_RECONCILIATION.json` → `docs_tree_text_files`); **979** `docs_plans` + **258** `docs_other` + meta trong inventory (không bỏ qua cả cây `docs/`).

Lệnh: `python3 .agents/scripts/classify-knowledge-groups.py` (sau `build-knowledge-inventory`).

## Repo db-pro (`scope_complete` + `group_scope_complete`)

**Đối soát v2:** `.agents/memory/KNOWLEDGE_RECONCILIATION_V2.json` — so `source_sha256` ↔ nội dung mirror (sau frontmatter), `mirror_body_sha256`, cờ `truncated`.

| Trạng thái (sau reconcile v2) | Số dòng | Ý nghĩa |
|---|---:|---|
| `mirrored_full_verified` | **1561** | Hash mirror khớp nguồn (full byte, không cap 512 KB) |
| `excluded_approved` | **20** | Canonical / curated — `APPROVED_EXCLUSIONS.json` |
| **Tổng nguồn repo** | **1581** | Không đếm artefact pipeline (`.agents/memory/KNOWLEDGE_*.json`, `MIGRATION_*.json`, …) — `is_pipeline_memory_artifact()` |

**`scope_complete`:** chỉ `true` khi reconcile v2 báo `unresolved_count === 0` (không dùng “file mirror tồn tại” hay `pending === 0` đơn thuần).

### Vùng (`by_zone`)

| Zone | Số file |
|---|---:|
| `docs_plans` | 979 |
| `docs_other` | 258 |
| `agents_other` | 295 |
| `agents_memory` | 13 |
| `skills_dot` | 16 |
| `cursor` | 15 |
| `repo_canonical` | 3 |
| `repo_config` | 2 |

## User-level agent (`~/.agents`, `~/.cursor`)

| Zone | Số | Trạng thái |
|---|---:|---|
| `user_agents_skills` | 804 | **catalogued_user_home** — không copy vào repo |
| `user_agents_agents` | 16 | catalogued |
| `user_agents_memory` | 9 | **user_skeleton_only** (1) + catalogued — **không** có corpus db-pro global |
| `user_agents_scripts` | 1 | catalogued (`search-memory.sh`) |
| `user_cursor_skills` | — | **`not_exists`** trên máy này (skills Cursor nằm trong repo `.cursor/skills/`) |

**Quy tắc:** Kiến thức **db-pro** nằm trong git repo; `~/.agents/skills` là catalog chéo (804 skill) để agent biết còn gì **ngoài** `skills-lock.json` (41).

## Pipeline

```bash
python3 .agents/scripts/build-knowledge-inventory.py
python3 .agents/scripts/migrate-plans-to-memory.py        # v1 plan units (197)
python3 .agents/scripts/migrate-pending-knowledge.py      # ghi mirror (unverified)
python3 .agents/scripts/reconcile-knowledge-sources-v2.py # hash nguồn ↔ mirror
python3 .agents/scripts/classify-knowledge-groups.py      # GROUP_RECONCILIATION.md
python3 .agents/scripts/migrate-pending-knowledge.py      # mirror sau classify
python3 .agents/scripts/reconcile-knowledge-sources-v2.py
python3 .agents/scripts/reconcile-memory-migration.py     # v1 excerpt (song song)
```

## Blueprint 3 — skill gốc (2026-10-24)

Đã thêm **13** skill vòng đời từ `agents-memory-blueprint 3` vào `.agents/skills/`:

`software-lifecycle`, `requirements-discovery`, `solution-design`, `implementation`, `bug-investigation`, `refactoring`, `testing-validation`, `code-review`, `security-review`, `release-deployment`, `incident-response`, `memory-curation`, `handoff-retrospective`.

Script: `.agents/scripts/validate-skills.sh` — **69** `SKILL.md` pass + **14** tham chiếu `template/` (`validate-skill-references.py`). Template `security-review.md` đã sync blueprint 3.

## Chưa / không áp dụng

- Binary (PNG, log lớn, zip): không trong inventory text scan.
- `_archive/`: loại khỏi quét.
- **Curate** `needs-review` → `active`: việc con người/agent, không phải migration byte.

## Claim được phép

✅ “Toàn bộ **nguồn text repo** trong inventory đã mirror hoặc skipped có lý do; user home **đã catalog**.”

❌ “804 skill global đã import vào repo” — chỉ có catalog đường dẫn.
