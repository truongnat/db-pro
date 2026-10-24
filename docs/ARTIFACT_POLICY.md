# Artifact policy — db-pro

**Mục đích:** Tránh commit bản sao/regenerate vào git; giữ evidence audit có disposition rõ ràng.

## 1. Ba tầng

| Tầng | Định nghĩa | Ví dụ |
|------|------------|--------|
| **Source** | Sửa tay, review PR | `crates/`, `docs/plans/**/PLAN.md`, `.agents/skills/` |
| **Generated** | Tái tạo bằng script; không commit | Memory mirrors, `KNOWLEDGE_*.json`, `cargo` `target/` |
| **Evidence** | Bằng chứng run/release; commit có chủ đích | RQM `result.json`, readiness PNG/log, coordinator `RUN_INVALIDATION.md` |

## 2. Agents memory (generated)

| Path | Git | Regenerate |
|------|-----|------------|
| `.agents/memory/entries/migrated/by-source-file/` | ❌ | `scripts/bootstrap-agents-memory.sh` (purge orphans + collision check trước migrate) |
| `.agents/memory/entries/migrated/from-plans/` | ❌ | `migrate-plans-to-memory.py` (v1 excerpt) |
| `KNOWLEDGE_*.json`, `MIGRATION_*.json` | ❌ | cùng bootstrap |
| `APPROVED_EXCLUSIONS.json`, `SOURCE_SCOPE_MANIFEST.json` | ✅ | sửa tay + bootstrap |
| `entries/{environment,patterns,workflows}/` curated | ✅ | sửa tay |

Chi tiết: `docs/plans/active/agents-memory-framework/VERIFICATION.md` §3, §9.

## 3. Release qualification runs (`runs/<run_id>/`)

| Path | Git | Ghi chú |
|------|-----|---------|
| `coordinator/*`, `run-manifest.json` | ✅ | Luôn |
| `workers/**/result*.json`, retained PNG/JSON trong `RUN_INVALIDATION.md` | ✅ | Frozen / invalidated run |
| `workers/**/package-*.log`, `artifact-ready.marker` | ✅ | Hash / path tới binary |
| `artifacts/**/*.tar.gz`, `.../MacOS/db-pro-native` | ❌ | Local; metadata `README-INSTALL.txt`, `Info.plist` có thể ✅ |
| `expected-instances.json` | ✅ nếu plan/RUNBOOK yêu cầu freeze | ~2MB; không ignore blanket |

Mẫu gitignore: `docs/plans/_template/runs.gitignore` → copy vào `runs/.gitignore` khi tạo run mới.

## 4. Release readiness (`docs/release/`)

| Path | Git |
|------|-----|
| `*.md` report | ✅ |
| `evidence/readiness-YYYY-MM-DD/` được report **link** | ✅ (toàn bundle) |
| Scratch / chưa link trong report | ❌ hoặc không tạo |

**Không** dùng `evidence/readiness-*/` global gitignore.

## 5. Plan UI evidence (PNG)

**Contract (root `AGENTS.md`):** UI thay đổi cần runtime evidence tại **1280×800, 1440×900, 1920×1080** (cộng theme/state mà plan/VERIFICATION yêu cầu). Cleanup **không** được xóa viewport lớn để “gọn repo” nếu gate vẫn còn áp dụng.

| Trạng thái plan | Quy tắc |
|-----------------|--------|
| `active` / `RUNTIME_VERIFY` | Giữ mọi PNG (hoặc artifact tương đương) được `VERIFICATION.md` / checklist liệt kê cho từng viewport + theme |
| `completed` | Giữ nguyên bộ evidence đã dùng để đóng plan; không bulk-delete theo kích thước file |

**Giảm dung lượng (chỉ sau audit từng plan):**

- **Lossless** (git LFS, `pngcrush`/tương đương) — không đổi pixel, không bỏ viewport.
- **Dedup / thay thế** — chỉ khi plan có **manifest** (path, logical size, theme, surface) chứng minh artifact mới vẫn đáp **cùng** gate VERIFICATION; owner cập nhật VERIFICATION + disposition trong FINDINGS.

**Không làm:** xóa 1920×1080 (hoặc chỉ giữ một viewport) như chiến lược mặc định; không archive `_archive/` hàng loạt không có disposition per plan.

## 6. Local only (không git)

- `target/`, `target-build/` (đã trong `.gitignore`)
- RQM tarball / extracted app (xem `runs/.gitignore`)
- `~/.agents` catalog — chỉ inventory JSON local

## 7. Checklist trước `gitignore` / `git rm`

1. `grep` link trong `VERIFICATION.md`, `AGENT_EVIDENCE.md`, `RUN_INVALIDATION.md`, `docs/release/*.md`.
2. Mục “retain” trong invalidation → **phải** còn tracked hoặc doc **local-only** + path rebuild (`package-macos.log`).
3. Chạy `scripts/verify-artifact-policy.sh` (readiness + RQM frozen run).
4. Sau đổi memory sources: `scripts/bootstrap-agents-memory.sh`.

## 8. Bootstrap sau clone (và CI)

Mirrors nằm trong `.gitignore` → checkout sạch **không** có `entries/migrated/**`. Chỉ chạy `reconcile-knowledge-sources-v2.py` sẽ thiếu mirror và báo sai.

**Local / CI (đúng):**

```bash
scripts/bootstrap-agents-memory.sh
```

Tương đương: `build-knowledge-inventory.py` → `migrate-pending-knowledge.py` → reconcile → `classify-knowledge-groups.py` → migrate → reconcile (VERIFICATION §9).

**CI tùy chọn:** job chạy **toàn bộ** `bootstrap-agents-memory.sh`; fail nếu `scope_complete` false. **Không** commit JSON/mirror artifacts. **Không** dùng reconcile-only trên tree không bootstrap.

**Không** thêm CI memory gate nếu chưa sẵn sàng chi phí generate ~1.5k mirrors mỗi run — khi đó bỏ hẳn bước đó, không reconcile rỗng.
