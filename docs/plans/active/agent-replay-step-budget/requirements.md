# Requirements — #448 agent replay step budget

Status: approved by owner on 2026-10-16; R1–R4 implemented and locally verified.
Issue: https://github.com/truongnat/db-pro/issues/448

- [x] R1: Count completed-call cache replays toward the existing `MAX_AGENT_TOOL_STEPS = 10` run budget. On the 11th tool attempt, emit the existing max-steps failure and stop provider continuation.
- [x] R2: Preserve idempotency: repeated identical call IDs never re-execute successful, failed, or rejected tools; preserve input/tool collision detection and confirmation safety.
- [x] R3: Preserve the boundary: a run with ten tool attempts can still receive the provider's final text/completion. A new orchestrator starts with a fresh budget; confirmation continuation does not reset it.
- [x] R4: Prove the failure before the fix and the bounded behavior afterward with the real orchestrator and a local deterministic provider, without paid API calls or database mutations. Keep focused regression tests in the existing adjacent test file.

In scope: runtime step accounting, regression coverage, issue-specific evidence.
Out of scope: provider API redesign, transcript reserialization, new retry/timeout/budget knobs, UI changes, database execution changes, other GitHub tickets.

Acceptance: R1–R4 demonstrated by observable terminal events, provider request counts, tool execution counts, and the existing confirmation/idempotency regression suite. Live AI-provider and native UI verification are not claimed by this focused runtime fix.
