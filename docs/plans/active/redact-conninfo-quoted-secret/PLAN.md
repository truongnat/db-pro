# Redact Conninfo Quoted Secrets — Plan

State: REVIEW — implementation complete; full workspace suite interrupted and clippy blocked by existing warnings
Baseline SHA: `d5aea9568430c5391bb6150a9748332fd2e0d62e`
Branch: `main` (owner workflow override: no worktree / no topic branch)
Issue: truongnat/db-pro#476 — `[P1][Security] redact_conninfo dừng ở whitespace`

## Goal

`redact_conninfo` must fully remove secret material from a PostgreSQL
`conninfo` string for every value shape libpq accepts, so that
`SubscriptionInfo.conninfo_redacted` — which is serialized to the UI and can
reach agent context — never contains a partial password, passfile, or TLS key.

## Evidence / baseline

- Current implementation: `crates/core/src/domain/replication.rs:49`
  redacts `password=`/`PASSWORD=`/`passfile=`/`sslkey=`/`sslpassword=` by
  scanning from after `=` to the first ASCII whitespace.
- Known failure scenario: `host=db password='top secret' dbname=app` yields
  `host=db password=[redacted] secret' dbname=app`. The tail `secret'` is
  exposed through `conninfo_redacted` (`crates/app/src/tools/replication_surface_view.rs:132`).
- Second defect: the loop only redacts the first occurrence of each key, so a
  duplicated key leaves a later secret in clear text.
- Existing tests: `crates/core/src/domain/tests/replication_tests.rs`
  (`redacts_password_in_conninfo`) only covers an unquoted, space-free value.

## Scope

- `crates/core/src/domain/replication.rs`: `redact_conninfo` value-end logic
  (single-quoted values with `\'`/`\\` escapes) and all-occurrence redaction.
- `crates/core/src/domain/tests/replication_tests.rs`: regression coverage.

## Non-goals

- No change to the `SubscriptionInfo` shape, the SQL that reads
  `pg_subscription`, or the infrastructure call site.
- No general libpq conninfo parser; only the redaction boundary is hardened.
- No UI, agent, or provider-connector change.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | `redact_conninfo` is a pure core function; unit tests sufficient |
| SQLite | n/a | logical replication is PostgreSQL-only (`replication.rs` doc) |

## Architecture

`redact_conninfo` (`crates/core`) ← `PostgresAdapter::list_subscriptions`
(`crates/infrastructure`) → `SubscriptionInfo.conninfo_redacted`
(`crates/app` surface + serialization). The core function is the single
security boundary; hardening it covers every consumer.

## Acceptance criteria

- [x] Quoted value containing a space (`password='top secret'`) is fully redacted.
- [x] Escaped quote/backslash inside a quoted value does not terminate redaction early.
- [x] Every occurrence of a sensitive key is redacted, not just the first.
- [x] Unquoted values keep the existing first-whitespace behavior.
- [x] Regression tests cover the four cases above.
