# Checklist

## Foundation

- [x] Define provider-neutral Agent domain contracts.
- [x] Add document-versioned SQL patch validation.
- [x] Add bounded context and result summary builders.
- [x] Add execution permission decisions backed by core SQL safety.
- [x] Add focused unit tests for safety and stale-state invariants.
- [x] Add run state machine with run-id validation and pending confirmations.
- [x] Add centralized mode/tool permission matrix and structured tool errors.

## Runtime/UI follow-up

- [x] Route typed agent tool requests through the runtime worker.
- [x] Add typed provider tool-call events and a bounded multi-step orchestrator.
- [x] Keep run/session/document routing explicit, including stale-safe query refresh.
- [x] Pause and resume the provider loop for patch and database confirmations.
- [x] Keep agent query output as bounded workflow state instead of overwriting the visible query result.
- [x] Add tool activity and patch preview to the existing compact panel.
- [x] Add executor-side read-only/mutation confirmation enforcement.
- [x] Verify PostgreSQL and SQLite runtime paths independently.
- [x] Run native UI and live provider verification.

