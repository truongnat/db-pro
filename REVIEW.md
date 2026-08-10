# DB Pro — Adversarial Review Policy

## Objective

Treat every PR as potentially incorrect until evidence proves otherwise.

Do not optimize for number of comments.
Optimize for real defects.

## Severity

### P0
- catastrophic data loss
- critical security vulnerability
- unusable application

### P1
- incorrect database mutation
- SQL injection
- broken transaction atomicity
- precision/data corruption
- stale state producing incorrect behavior
- provider-specific correctness failure
- safety policy bypass

### P2
- maintainability
- UX inconsistency
- missing non-critical tests
- performance concern without correctness impact

P0/P1 block merge.

---

## Required DB review

For database changes ask:

1. What happens if statement 2 of 3 fails?
2. What happens with PostgreSQL?
3. What happens with SQLite?
4. What happens when affectedRows = 0?
5. What happens when metadata cache is stale?
6. What happens with quoted identifiers?
7. What happens with NULL?
8. What happens with BIGINT / NUMERIC precision?
9. What happens if another session changes the row?
10. What state does UI show after failure?

---

## Review previous reviewers

When other AI reviewers have commented:

Do not agree automatically.

For every P0/P1:
- prove it
- disprove it
- or downgrade it

Provide concrete file/function evidence.

Look specifically for shared blind spots between reviewers.

---

## Test review

Do not accept:
"tests pass"

as sufficient evidence.

Check whether tests actually prove the claimed invariant.

Example:

A transaction rollback test must verify that statement 1
did NOT persist after statement 2 failed.

---

## Runtime claims

Source reasoning != runtime verification.

Only mark runtime behavior verified if actual runtime evidence exists.

---

## Summary

Return:

P0:
P1:
P2:

Confirmed findings:
Rejected findings:
Disagreements:
Missing tests:
Runtime gaps:
