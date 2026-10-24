# Phạm vi hoàn tất — wave `v1-plan-evidence-excerpt`

**Không** gọi là “migration toàn bộ kiến thức db-pro”.

## Đã hoàn tất (có đối soát)

Wave **`v1-plan-evidence-excerpt`** PASS khi `MIGRATION_RECONCILIATION.json` → `pass_v1.overall: true`.

| Chỉ số | Giá trị (2026-10-24) |
|---|---|
| Plan dirs quét (`active`+`completed`) | 200 |
| Audit slice `.md` (trừ INDEX/CAPTURE) | 13 |
| Đơn vị có ≥1 file v1 | **197** |
| Plan dirs không có file v1 | 16 |
| **File nguồn mirror** (PLAN/FINDINGS/VERIFICATION/AGENT_EVIDENCE) | **605** |
| File bị cắt excerpt (>12 000 ký tự) | 38 (~670 KB nội dung không trong mirror) |
| Entry mirror trên disk | 197 (= manifest) |
| `docs/plans/**/*.md` tổng | 932 |
| **Không thuộc v1** (ước lượng) | **~327** file `.md` trong `docs/plans` (CHECKLIST, context, …) |
| `docs/**/*.md` ngoài plans | ~209 (release, architecture, …) — **out of scope** |

Artifacts:

- [`.agents/memory/SOURCE_SCOPE_MANIFEST.json`](../../../../.agents/memory/SOURCE_SCOPE_MANIFEST.json)
- [`.agents/memory/MIGRATION_RECONCILIATION.json`](../../../../.agents/memory/MIGRATION_RECONCILIATION.json)
- [`.agents/memory/MIGRATION_MANIFEST.json`](../../../../.agents/memory/MIGRATION_MANIFEST.json)

Lệnh:

```bash
python3 .agents/scripts/migrate-plans-to-memory.py
python3 .agents/scripts/reconcile-memory-migration.py
```

## Chưa migrate (wave khác)

- `CHECKLIST.md`, `context.md`, `LEDGER`, RQM `runs/**`, `evidence/**`, `docs/release/**`, `AGENTS.md`, `.skills/`, … — xem `explicitly_out_of_scope` trong SOURCE_SCOPE_MANIFEST.

## Curated (tách khỏi mirror)

3 entry tay trong `entries/environment|patterns|workflows/` — không tính vào 197 mirror.
