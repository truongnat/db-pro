# Definition of Done — #448

Requirements/context approved by owner via scope-selection prompt on 2026-10-16.

- [x] D1 / R1: 11 identical completed-call attempts terminate with the existing max-steps error after at most 11 provider responses; successful replay never reruns the tool.
- [x] D2 / R2: Cached success, failure, and rejection cannot evade the budget; existing collision, rejection and confirmed-mutation idempotency tests pass.
- [x] D3 / R3: Ten attempts still permit a final provider completion; a new run has a fresh allowance; confirmation continuation retains the original allowance.
- [x] D4 / R4: A permanent meaningful regression fails before the fix and passes afterward; a separate public-orchestrator smoke confirms provider-call and execution counts.
- [x] D5 / R4: Run workspace fmt/check/clippy/test gates; report exact failures or unavailable prerequisites without weakening checks. Workspace Clippy FAIL in unchanged app sources; other gates PASS.
- [x] D6: Update docs/evidence/status to actual lifecycle; no commits/push/PR/issue closure without owner instruction. Database providers individually n/a; UI unchanged. Lifecycle REVIEW, not merge-ready.
