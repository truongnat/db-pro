# Redact Conninfo Quoted Secrets — Findings

Record only evidence-backed findings.

## P0

None.

## P1

### F1 — Quoted secret value leaks its tail (#476)

- Evidence: `redact_conninfo` (`crates/core/src/domain/replication.rs:49`)
  ends the redaction range at the first ASCII whitespace. A libpq-quoted value
  may contain spaces, so the range stops inside the secret.
- Failure scenario: `host=db password='top secret' dbname=app` →
  `host=db password=[redacted] secret' dbname=app`. The literal `secret'`
  reaches `SubscriptionInfo.conninfo_redacted`, is rendered at
  `crates/app/src/tools/replication_surface_view.rs:132`, and the struct is
  `Serialize`, so the fragment can reach agent context.
- Severity: P1 — a security redaction boundary that partially fails.
- Scope: replace the whitespace end-scan with a quoted/escaped-aware value
  scanner in `redact_conninfo`.
- Provider: PostgreSQL only (logical replication inventory).
- Testability: pure core function → deterministic unit tests.

### F2 — Only the first occurrence of each key is redacted

- Evidence: `out.find(key)` is called once per key and never re-scanned.
- Failure scenario: `password=one host=db password=two` leaves `password=two`
  in clear text.
- Severity: P1 — same boundary, same class of leak.
- Scope: loop occurrences per key.
- Testability: unit test asserting two `[redacted]` markers and no secret text.

## P2

None.

## Rejected / downgraded findings

- The ticket also notes the unquoted `\S+` case as already covered. Confirmed:
  unquoted values already stop at whitespace correctly; retained as-is.

## Runtime gaps

- PostgreSQL: no live `pg_subscription` fixture used; the redaction input is a
  raw string, so unit evidence is authoritative for this pure function.
- SQLite: n/a — feature is PostgreSQL-only.
- UI: no UI change; the value is already rendered through the existing surface.
