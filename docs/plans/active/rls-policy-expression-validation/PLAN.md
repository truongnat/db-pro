# RLS Policy Expression Validation — Plan

State: REVIEW
Branch: `fix/validate-rls-policy-expressions`
Issue: #507

## Goal

Ensure user-provided PostgreSQL `USING` and `WITH CHECK` values produce exactly one PostgreSQL expression and cannot escape the policy DDL wrapper or append a statement.

## Evidence / baseline

- `crates/core/src/application/object_mutation_builders.rs` pasted expressions verbatim in both `CREATE POLICY` and `ALTER POLICY` builders.
- `SchemaService::execute_ddl` splits SQL and batch execution splits it again before connector execution; a semicolon in the raw expression can escape the policy statement.
- Existing expressions are sourced from the RLS security UI and preview flows.
- Baseline: `beaa1882a6253d902748c27fba53b99d9969b635`.

## Scope

- Parse each non-empty predicate with `sqlparser`'s PostgreSQL dialect; require one expression and EOF.
- Apply validation to both create/generate and alter paths, for both expression fields.
- Keep emitted expression text unchanged. Place builder-owned closing parentheses after a newline so trailing line comments cannot hide them.
- Add regression tests for injected statements, valid PostgreSQL forms, backslash quote handling, and both builder paths.

## Non-goals

- No other provider's policy syntax or RLS implementation.
- No UI redesign or policy semantics changes.
- No broader DDL executor changes.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | parser/build tests; live DB mutation not required for this input-validation change |
| SQLite | no | existing capability gate reports that RLS is unsupported |

## Architecture

RLS UI input → `ObjectMutationService::plan` → `build_rls_statements` validates expression via core domain safety parser → unchanged expression is emitted in CREATE/ALTER DDL → existing DDL batch path.

## Acceptance criteria

- [x] Injection and trailing tokens are rejected before DDL emission.
- [x] Both `USING` and `WITH CHECK` are validated for CREATE and ALTER.
- [x] Representative valid PostgreSQL expressions remain accepted and preserved.
- [x] Builder-owned `)` is outside a trailing line comment.
- [ ] Independent review and PR CI complete.
