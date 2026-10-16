# RLS Policy Expression Validation — Findings

## P0

None observed.

## P1

**Fixed in this branch, pending independent review:** raw `using_expr` / `with_check_expr` values were pasted into CREATE and ALTER policy DDL. A payload containing `); DROP TABLE ...; --` can escape the parenthesized clause and is split into separate statements by the existing batch path.

Evidence at baseline `beaa1882a6253d902748c27fba53b99d9969b635`: `crates/core/src/application/object_mutation_builders.rs` lines 630–670; issue #507. The builder now validates with PostgreSQL expression parsing and full-input EOF, before inserting the original text.

## P2

The `sqlparser` PostgreSQL dialect is a parser implementation, not the PostgreSQL server grammar. Less-common PostgreSQL extensions or syntax outside its supported grammar may be rejected; this implementation intentionally fails closed. Representative casts, `current_setting`, `IN (SELECT ...)`, interval expressions, custom `?` operator, and escaped string forms are covered by tests.

## Rejected / downgraded findings

- “Reject all comments” is unnecessary when the complete input is parsed as one expression; however, builder-owned closing parentheses are emitted after a newline so a trailing line comment cannot consume them.
- A custom handwritten quote/parenthesis scanner was rejected after identifying a PostgreSQL standard-conforming-string backslash ambiguity that could hide an injected semicolon.

## Runtime gaps

- PostgreSQL: no live database mutation executed; parser and DDL-planning tests only.
- SQLite: RLS unsupported and explicitly capability-gated.
- UI: no end-to-end native UI run; UI preview test exercises the same planning function.
