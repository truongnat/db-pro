---
schema_version: 1
id: "MEM-SRC-04c6c75c3d9ad7e4"
title: "Source mirror: docs/plans/active/agent-replay-step-budget/dod.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/agent-replay-step-budget/dod.md"
source_sha256: "04c6c75c3d9ad7e4a67c9942a76f0057837901a79f913512938dc893bba72159"
mirror_body_sha256: "04c6c75c3d9ad7e4a67c9942a76f0057837901a79f913512938dc893bba72159"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Definition of Done — #448

Requirements/context approved by owner via scope-selection prompt on 2026-10-16.

- [x] D1 / R1: 11 identical completed-call attempts terminate with the existing max-steps error after at most 11 provider responses; successful replay never reruns the tool.
- [x] D2 / R2: Cached success, failure, and rejection cannot evade the budget; existing collision, rejection and confirmed-mutation idempotency tests pass.
- [x] D3 / R3: Ten attempts still permit a final provider completion; a new run has a fresh allowance; confirmation continuation retains the original allowance.
- [x] D4 / R4: A permanent meaningful regression fails before the fix and passes afterward; a separate public-orchestrator smoke confirms provider-call and execution counts.
- [x] D5 / R4: Run workspace fmt/check/clippy/test gates; report exact failures or unavailable prerequisites without weakening checks. Workspace Clippy FAIL in unchanged app sources; other gates PASS.
- [x] D6: Update docs/evidence/status to actual lifecycle; no commits/push/PR/issue closure without owner instruction. Database providers individually n/a; UI unchanged. Lifecycle REVIEW, not merge-ready.
