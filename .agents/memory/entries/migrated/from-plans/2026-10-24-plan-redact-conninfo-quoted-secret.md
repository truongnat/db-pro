---
schema_version: 1
id: "MEM-PLAN-redact-conninfo-quoted-secret-695a12192424"
title: "Plan archive: Redact Conninfo Quoted Secret"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, redact-conninfo-quoted-secret]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "695a1219242409a2571ebc5797083994836e37b1823a5f2ca5cc3a69b6d7c14d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/redact-conninfo-quoted-secret/PLAN.md"
  - "docs/plans/active/redact-conninfo-quoted-secret/FINDINGS.md"
  - "docs/plans/active/redact-conninfo-quoted-secret/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/redact-conninfo-quoted-secret/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Redact Conninfo Quoted Secrets — Verification

State: REVIEW — implementation and core verification pass; workspace clippy has
pre-existing blockers and the full workspace test run was interrupted.

## Commands actually executed

| Command | Result |
|---|---|
| `cargo test -p db-pro-core --lib domain::replication::tests` | PASS — 8 passed, 0 failed |
| `cargo test -p db-pro-core --lib` | PASS — 456 passed, 0 failed |
| `cargo check --workspace` | PASS |
| `cargo +nightly fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | FAIL — three existing `collapsible_match` warnings in untouched `crates/core/src/domain/saved_task.rs` (lines 153, 158, 165) are promoted to errors |
| `cargo test --workspace` | INTERRUPTED before completion; no result claimed |

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | PASS — core unit tests above | N/A | pure core string function |
| SQLite | N/A | N/A | feature is PostgreSQL-only |

## UI lifecycle

```text
UI → command → backend → database → introspection → refreshed UI
```

Status: N/A — no runtime/DB path change; redaction happens before the value is
exposed.

## Remaining evidence gaps

- Full workspace tests were interrupted and remain unverified.
- Workspace clippy is blocked by the three unrelated existing warnings above.
- Live `pg_subscription` fixture not exercised; unit coverage is authoritative
  for this pure function boundary.

## Completion decision

Implementation is ready for review. Do not mark the plan COMPLETED until the
full applicable workspace gates are satisfied or the outstanding gates are
explicitly dispositioned.

