# Agents Memory Framework — Verification

Baseline SHA: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`. Recorded: 2026-10-24.

## 1. Lifecycle gates

Read `docs/plans/STATUS.md`, `docs/plans/FEATURE_LIFECYCLE.md` before claiming completion.

## 2. Scope manifest

File: `.agents/memory/SOURCE_SCOPE_MANIFEST.json` — wave `v1-plan-evidence-excerpt`, in/out of scope.

## 3. v2 content reconcile (bắt buộc — bảo toàn byte)

`build-knowledge-inventory.py` chỉ ghi `mirror_present_unverified` (filesystem). **Không** dùng đó làm bằng chứng migrate.

```bash
python3 .agents/scripts/build-knowledge-inventory.py
python3 .agents/scripts/migrate-pending-knowledge.py
python3 .agents/scripts/reconcile-knowledge-sources-v2.py
# scope_complete / content_scope_complete must be true; unresolved_count === 0
python3 .agents/scripts/classify-knowledge-groups.py
python3 .agents/scripts/migrate-pending-knowledge.py
python3 .agents/scripts/reconcile-knowledge-sources-v2.py
```

**Last run:** read **only** `.agents/memory/KNOWLEDGE_RECONCILIATION_V2.json` — fields `reconciled_at_utc`, `scope_complete`, `unresolved_count`, `by_status` are authoritative (do not duplicate timestamps or counts in this doc). After any edit to this file, `KNOWLEDGE_MIGRATION_STATUS.md`, or `GROUP_RECONCILIATION.md`, rerun §9 until `reconciled_at_utc` is newer than that edit.

**Pass:** `scope_complete` === true, `unresolved_count` === 0, no `mirrored_truncated_verified` entries (truncation fails scope).

Large JSON (full byte, không cap): `docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/expected-instances.json` — **1 980 162** bytes, `truncated: false`, `mirrored_full_verified`, `source_sha256` `4611ad74368d6e08906326bba575f79107e0fd7d216b01cbd9d6356f8b03d1c9`.

Artifact: `.agents/memory/KNOWLEDGE_RECONCILIATION_V2.json`. Parser fix: mirror body sau `---` giữ newline đầu nguồn (LICENSE.txt).

**Proof inventory ≠ content:** sau `build-knowledge-inventory.py` alone, summary vẫn `mirror_present_unverified`; `reconcile-knowledge-sources-v2.py` vẫn `scope_complete: true` nếu mirror bytes khớp.

## 4. v1 plan excerpt reconcile

```bash
python3 .agents/scripts/migrate-plans-to-memory.py
python3 .agents/scripts/reconcile-memory-migration.py
# pass_v1.overall must be true
```

**Last run (2026-10-24):**

```json
{
  "included_source_files": 605,
  "truncated_source_files": 38,
  "mirror_files_on_disk": 197,
  "manifest_entry_count": 197,
  "docs_plans_md_total": 932,
  "plans_md_not_in_v1_scope_estimate": 327,
  "pass_v1": {
    "counts_match": true,
    "no_missing_mirror": true,
    "no_body_drift": true,
    "overall": true
  }
}
```

Full snapshot: `evidence/framework-discovery-2026-10-24.json` + `.agents/memory/MIGRATION_RECONCILIATION.json`.

## 5. Completion claim (allowed wording)

✅ “Repo text sources: **v2 hash reconcile PASS** (`mirrored_full_verified` + approved exclusions; `mirrored_truncated_verified` = 0, truncation fails scope).”

✅ “Wave **v1-plan-evidence-excerpt** đã migrate **605** file nguồn (4 tên/plan + audit slices) vào **197** entry excerpt, đối soát PASS.”

❌ “Đã migration hết kiến thức / toàn bộ docs / toàn repo.”

❌ “`mirror_present` / `pending === 0` chứng minh nội dung đã mirror.”

## 6. Search smoke

```bash
.agents/scripts/search-memory.sh "rustc pin"
.agents/scripts/search-memory.sh "release-qualification-matrix"
```

## 7. Product gates

`cargo test` / clippy — **not run** (docs/infrastructure).

## 8. Skills + template references

```bash
python3 .agents/scripts/test-knowledge-mirror-utils.py
.agents/scripts/validate-skills.sh
```

**Expected:** mirror-utils tests exit `0`; validator reports **69** `SKILL.md` files with valid frontmatter and **14** resolved paths under `.agents/template/` (includes `software/security-review.md` from blueprint 3).

Lifecycle skills added from blueprint 3: `software-lifecycle`, `requirements-discovery`, `solution-design`, `implementation`, `bug-investigation`, `refactoring`, `testing-validation`, `code-review`, `security-review`, `release-deployment`, `incident-response`, `memory-curation`, `handoff-retrospective`.

## 9. Post-edit reconcile (mandatory)

**Git:** v2 mirror tree (`entries/migrated/by-source-file/`) and pipeline JSON are **gitignored** — regenerate locally after clone (`scripts/bootstrap-agents-memory.sh` or §3 commands). **CI:** same full bootstrap before reconcile — never `reconcile-knowledge-sources-v2.py` alone on a fresh checkout (`docs/ARTIFACT_POLICY.md` §8).

After any edit to plan docs listed in v2 inventory, run full pipeline and refresh `reconciled_at_utc` in §3:

```bash
python3 .agents/scripts/build-knowledge-inventory.py
python3 .agents/scripts/migrate-pending-knowledge.py
python3 .agents/scripts/reconcile-knowledge-sources-v2.py
python3 .agents/scripts/classify-knowledge-groups.py
python3 .agents/scripts/migrate-pending-knowledge.py   # classify rewrites GROUP_RECONCILIATION.md
python3 .agents/scripts/reconcile-knowledge-sources-v2.py
```
