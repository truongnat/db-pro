# Đối soát nhóm nguồn kiến thức (repo)

**Cập nhật:** 2026-10-24
**`filesystem_scope_complete`:** `True`
**`content_scope_complete` (v2 hash):** `True`
**`scope_complete` (cả hai):** `True`

## Đối soát nội dung mirror (v2 — bắt buộc cho bảo toàn byte)

Không dùng `mirror_present` / `migrated_full_mirror` (v1) làm bằng chứng.

| Chỉ số | Giá trị |
|---|---:|
| `mirrored_full_verified` | 1562 |
| `mirrored_truncated_verified` | 0 |
| `excluded_approved` + artifact | 21 |
| `unresolved_count` | 0 |

Artifact: `.agents/memory/KNOWLEDGE_RECONCILIATION_V2.json` — lệnh: `python3 .agents/scripts/reconcile-knowledge-sources-v2.py`.

## Filesystem vs inventory

| Chỉ số | Giá trị |
|---|---:|
| File text trong `docs/` (không `_archive`) | 1239 |
| Dòng inventory `docs_plans` | 980 |
| Dòng inventory `docs_other` | 259 |
| Thiếu trong inventory | 0 |

**Giải thích số cũ ~327 / ~1141:** v1 chỉ mirror 4 tên/plan → phần còn lại **không** ‘out of scope’ mà đã vào **v2 `by-source-file`** (`content_verified_full_mirror` sau `reconcile-knowledge-sources-v2.py`). Toàn `docs/` ngoài `_archive` được quét vào inventory (plans + docs_other + …).

## Bảng nhóm

| Nhóm | Số | durable (chính) | disposition (chính) |
|---|---:|---|---|
| `agents_memory_curated` | 3 | durable_high | canonical_in_place |
| `agents_memory_curated_readme` | 6 | durable_medium | canonical_in_place |
| `agents_memory_meta` | 4 | structural | inventory_artifact |
| `agents_other` | 6 | durable_medium | mirrored-full-verified |
| `agents_rules_template` | 21 | durable_high | mirrored-full-verified |
| `agents_skills_vendored` | 268 | durable_medium | mirrored-full-verified |
| `cursor` | 15 | durable_medium | mirrored-full-verified |
| `docs_architecture_numbered` | 1 | durable_high | content_verified_full_mirror |
| `docs_other` | 135 | durable_medium | content_verified_full_mirror |
| `docs_release` | 123 | durable_high | content_verified_full_mirror |
| `plans_audit_slice` | 15 | durable_high | content_verified_full_mirror |
| `plans_checklist` | 191 | durable_medium | content_verified_full_mirror |
| `plans_evidence_path` | 5 | durable_low | content_verified_full_mirror |
| `plans_json_sidecar` | 24 | ephemeral | content_verified_full_mirror |
| `plans_lifecycle_core` | 624 | durable_high | content_verified_full_mirror |
| `plans_other_md` | 60 | durable_medium | content_verified_full_mirror |
| `plans_rqm_run_artifacts` | 14 | ephemeral | content_verified_full_mirror |
| `plans_rqm_run_docs` | 5 | durable_medium | content_verified_full_mirror |
| `plans_supplement` | 36 | durable_medium | content_verified_full_mirror |
| `plans_template` | 6 | structural | excluded_template |
| `repo_canonical` | 3 | durable_high | canonical_in_place |
| `repo_config` | 2 | durable_high | canonical_in_place |
| `skills_dot` | 16 | durable_medium | mirrored-full-verified |

Chi tiết JSON: `.agents/memory/KNOWLEDGE_GROUP_RECONCILIATION.json`

## Durable ưu tiên (không mirror mù)

- `durable_high`: lifecycle core, plans root, architecture numbered, release docs, rules — đọc trực tiếp hoặc dual excerpt+full.
- `durable_medium`: CHECKLIST, supplement, vendored skills.
- `ephemeral`: RQM `runs/**` JSON — mirror để search; không promote `active` hàng loạt.
- `excluded_template`: `_template` only (5 files).
