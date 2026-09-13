# Checklist

## Foundation

- [x] Define provider-neutral Agent domain contracts.
- [x] Add document-versioned SQL patch validation.
- [x] Add bounded context and result summary builders.
- [x] Add execution permission decisions backed by core SQL safety.
- [x] Add focused unit tests for safety and stale-state invariants.

## Runtime/UI follow-up

- [ ] Route typed agent actions through the runtime worker.
- [ ] Tie session state to QueryDocument and document version.
- [ ] Add tool activity and patch preview to the existing compact panel.
- [ ] Add controlled read-only execution and explicit mutation confirmation.
- [ ] Verify PostgreSQL and SQLite runtime paths independently.
- [ ] Run native UI and live provider verification.
