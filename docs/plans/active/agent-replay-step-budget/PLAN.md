# Agent replay step budget — #448

Lifecycle: REVIEW. Task state: Review. Implementation and local runtime acceptance are satisfied; workspace Clippy remains failed in unchanged app sources. Not ready to merge; no independent approval.
Baseline: `b52a5088f7872f973bf5c31877c9b3698d306e8c`.
Issue: https://github.com/truongnat/db-pro/issues/448

Goal: stop repeated cached agent tool calls from bypassing the existing ten-step limit while preserving idempotency and confirmation safety.

Scope/acceptance: [requirements.md](requirements.md), R1–R4.
Architecture, alternatives, impact and verification design: [context.md](context.md).
Owner approved requirements/context on 2026-10-16. Completion conditions: [dod.md](dod.md). Execution: [tasks.md](tasks.md).

Non-goals: provider transcript/API changes, new retry or timeout mechanisms, UI changes, database mutations, unrelated tickets.

Provider matrix: PostgreSQL n/a; SQLite n/a independently — this fix changes runtime orchestration only. Native UI is unchanged. A local scripted provider proves the orchestrator bound, not a live remote-provider integration.

Evidence: [FINDINGS.md](FINDINGS.md), [VERIFICATION.md](VERIFICATION.md).
