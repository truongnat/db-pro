# Findings — #147

## F1 (P1, confirmed): `RolledBack` is reported for a batch that committed (PostgreSQL)

- Evidence: probe on this branch (PG 16.15 fixture), shapes A and B — `Err(phase=Statement,
  outcome=RolledBack)` with 1 row surviving.
- Why it matters: the product shows "rolled back" while the write is committed. A user who trusts
  the envelope retries an operation that already took effect, or deletes rows believing they are
  still there.
- Fingerprint (GH CLI, this branch): `gh issue view 147` — "and earlier writes actually survive"
  matches the measured shapes exactly.

## F2 (P1, confirmed): a batch can commit outside the wrapper transaction (both providers)

- PostgreSQL: any shape carrying `COMMIT` survives its writes (table above).
- SQLite: shape B (`INSERT; COMMIT; SELECT bad`) survives, and the envelope reports `Unknown`
  because the wrapper has nothing left to roll back.
- The harmful shape differs per provider (SQLite refuses a leading `BEGIN` itself), which is why
  the fix refuses the whole class at dispatch instead of special-casing a verb.

## F3 (accepted limitation): single statements are not covered

- Per #224, a single statement is a direct script execution surface (`execute`, not
  `execute_multi`). `BEGIN`, `COMMIT`, `ROLLBACK` typed alone keep working there. The multi-
statement path is the one whose contract is atomicity.

## F4 (root cause, from source)

- `QueryService::execute_multi` wraps all statements in one `execute_transaction` call and maps the
  connector envelope 1:1 into the UiEvent; the envelope's `RolledBack` therefore reaches the UI
  even when the wrapped transaction no longer exists at rollback time. The connector cannot detect
  this in general (its rollback is a no-op against a server that already committed), so the
  guarantee has to be created at dispatch — refuse the class, and the wrapper transaction always
  exists for the whole batch.
