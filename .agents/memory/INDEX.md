# Chỉ mục memory

> Giữ file này ngắn. Corpus đầy đủ: `search-memory.sh` + [`MIGRATION_MANIFEST.json`](MIGRATION_MANIFEST.json).

## Entry curated (ưu tiên tra cứu)

| Chủ đề | Loại | Đường dẫn | Xác minh | Ghi chú |
|---|---|---|---|---|
| Release rustc pin 1.95.0 | environment | `entries/environment/2026-10-24-release-compiler-pin-195.md` | 2026-10-24 | RQM qualify |
| UI theme token layers | pattern | `entries/patterns/2026-10-24-ui-theme-token-layers.md` | 2026-10-24 | Component audit |
| Agent memory search | workflow | `entries/workflows/2026-10-24-agent-memory-search.md` | 2026-10-24 | Blueprint contract |

## Corpus mirror wave `v1-plan-evidence-excerpt`

| Mục | Giá trị |
|---|---|
| Mirror entries | **197** (`needs-review`) |
| Source files mirrored | **605** (4 tên/plan + audit) |
| Truncated sources | **38** |
| Plans `.md` **not** in v1 | **~327** / 932 |
| Artifacts | `SOURCE_SCOPE_MANIFEST.json`, `MIGRATION_RECONCILIATION.json`, `MIGRATION_MANIFEST.json` |
| Owner doc | `docs/plans/active/agents-memory-framework/SCOPE_AND_COMPLETION.md` |

Tìm theo slug plan: `.agents/scripts/search-memory.sh "release-qualification-matrix"` hoặc tên feature.

## Phạm vi

- **Project:** `.agents/memory/` (repo db-pro).
- **Global:** `~/.agents/memory/` (cross-repo; skeleton, chưa có entry nội dung).
