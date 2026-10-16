# RLS Policy Expression Validation — Verification

State: REVIEW

## Commands actually executed

```text
cargo fmt --all
PASS (format applied)

cargo fmt --all -- --check && git diff --check
PASS (no formatting or whitespace errors)

cargo test -p db-pro-core policy_expression
PASS: 3 passed, 0 failed, 0 ignored; exit 0

cargo test -p db-pro-core rls_policy
PASS: 7 passed, 0 failed, 0 ignored; exit 0

cargo test -p db-pro-app policy_preview
PASS: 3 passed, 0 failed, 0 ignored; exit 0

cargo check --workspace
PASS; exit 0
```

The focused suites cover the #507 payload against CREATE and ALTER `USING`, CREATE `WITH CHECK`, valid PostgreSQL expression preservation, escaped strings/backslash boundaries, and keeping the generated close delimiter outside a trailing line comment. The initial handwritten scanner was discarded after review identified a backslash-quote ambiguity; the final validator uses `sqlparser`'s PostgreSQL dialect and requires EOF after one expression.

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | PASS (parser and DDL-plan unit tests) | NOT RUN | No DB was mutated; live provider not required for validation-only change |
| SQLite | PASS (existing capability gate) | N/A | RLS unsupported by design |

## UI lifecycle

```text
RLS form → preview planning → PostgreSQL expression validation → SQL preview
```

Status: automated preview test; no native UI/database round-trip performed.

## Remaining evidence gaps

- Independent security review and PR CI.
- Full workspace tests and clippy not run.
- No live PostgreSQL execution; no database mutation performed.

## Completion decision

Keep in REVIEW until independent review and PR checks are available.
