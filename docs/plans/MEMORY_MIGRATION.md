# Plan evidence → memory (pointer)

**Hoàn tất chỉ cho wave `v1-plan-evidence-excerpt`** — không phải toàn repo. Số liệu: [`SCOPE_AND_COMPLETION.md`](active/agents-memory-framework/SCOPE_AND_COMPLETION.md).

**Canonical owner:** [`docs/plans/active/agents-memory-framework/`](active/agents-memory-framework/PLAN.md)

Use that directory for PLAN, CHECKLIST, FINDINGS, VERIFICATION, and AGENT_EVIDENCE.

Quick commands:

```bash
.agents/scripts/search-memory.sh "<query>"
scripts/bootstrap-agents-memory.sh          # v2 mirrors + reconcile (post-clone)
python3 .agents/scripts/migrate-plans-to-memory.py   # v1 excerpt only
```

Artifact policy: `docs/ARTIFACT_POLICY.md`.

Manifest: [`.agents/memory/MIGRATION_MANIFEST.json`](../../.agents/memory/MIGRATION_MANIFEST.json)
