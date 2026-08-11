# DB Pro — Adversarial Review Policy

## Objective

Treat every PR as potentially incorrect until evidence proves otherwise.
Optimize for real defects, not comment count.

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

## Required database review

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

## Provider review

A provider claim requires provider evidence.

- PostgreSQL source support does not prove PostgreSQL runtime behavior.
- SQLite source support does not prove SQLite runtime behavior.
- Unsupported operations must be capability-gated with a user-visible reason.
- Never infer one provider from another provider's test.

## Review previous reviewers

Do not agree automatically.
For every P0/P1 from another reviewer: prove it, disprove it, or downgrade it with evidence.
Look for shared blind spots.

## Test review

"Tests pass" is not sufficient evidence.
Verify that the test proves the claimed invariant.
For rollback, verify earlier statements did not persist after a later statement failed.

## Runtime claims

Source reasoning != runtime verification.
Only mark runtime behavior verified when actual runtime evidence exists.

## Output

Return:

P0:
P1:
P2:

Confirmed findings:
Rejected findings:
Disagreements:
Missing tests:
Runtime gaps:
