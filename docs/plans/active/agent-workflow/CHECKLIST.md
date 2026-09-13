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
- [ ] Tie session state to QueryDocument and document version.
- [ ] Add tool activity and patch preview to the existing compact panel.
- [x] Add executor-side read-only/mutation confirmation enforcement.
- [ ] Verify PostgreSQL and SQLite runtime paths independently.
- [ ] Run native UI and live provider verification.
