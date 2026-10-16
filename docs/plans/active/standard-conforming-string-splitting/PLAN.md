# Driver-aware SQL String Splitting — Plan

State: REVIEW — implementation and automated checks complete; provider runtime evidence pending
Baseline SHA: `beaa1882a6253d902748c27fba53b99d9969b635`
Branch: `fix/standard-conforming-string-splitting`
PR: [#519](https://github.com/truongnat/db-pro/pull/519) — Draft
Issue: truongnat/db-pro#477 — `[P1][Bug] Statement splitter treat backslash là escape trong mọi string`

## Goal

Make semicolon splitting and the single-statement safety gate follow the connected driver's string-literal rules, so PostgreSQL's default standard strings cannot hide a second statement while MySQL backslash escapes remain supported.

## Evidence / baseline

- `crates/core/src/domain/safety_lexer.rs` treated every backslash in every single-quoted string as escaping the following byte.
- Under PostgreSQL's default `standard_conforming_strings=on`, `SELECT 'x\\'; DROP TABLE important; --` closes the first string at the quote after the backslash; the baseline splitter treated it as one statement, bypassing the multi-statement gate.
- The baseline MySQL behavior needs to keep backslash escaping enabled.
- Baseline SQL-safety regression coverage was in `crates/core/src/domain/safety/tests.rs` and did not distinguish providers.

## Scope

- Add dialect-aware quoted-string lexing and driver-aware statement splitting.
- Wire query, export, DDL, SQLite transaction guard, and query row-cap splitting to the known driver.
- Add PostgreSQL default/E-string and MySQL regression tests, including a direct multi-statement-gate test.

## Non-goals

- Model every MySQL `sql_mode` (notably `NO_BACKSLASH_ESCAPES`) or PostgreSQL session override `standard_conforming_strings=off`.
- Replace the lightweight SQL lexer with a full parser.
- Change query execution semantics beyond statement-boundary recognition.

## Provider matrix

| Provider | Supported | Automated evidence | Live/runtime |
|---|---|---|---|
| PostgreSQL | yes, standard strings + `E'…'` | regression unit tests | NOT VERIFIED |
| MySQL | yes, default backslash escapes | regression unit tests | NOT VERIFIED |
| SQLite | yes, standard quote behavior | workspace compile; caller explicitly selects SQLite mode | NOT VERIFIED |
| SQL Server | yes, standard quote behavior for this splitter | driver mapping | NOT VERIFIED |

## Architecture

`DriverType` selects `SqlStringDialect`; `safety::split_statements_for_driver` is used by `QueryService`, `ExportService`, `SchemaService`, SQLite batch transaction validation, and query-row-limit preprocessing. The default splitter remains PostgreSQL-standard for shared safety classification and PostgreSQL connector checks.

## Acceptance criteria

- [x] PostgreSQL standard strings treat backslash literally, and the reported payload splits into multiple statements.
- [x] PostgreSQL `E'…'` strings preserve backslash escapes.
- [x] MySQL default mode preserves backslash-escaped quote behavior.
- [x] Connection-backed execution, export and DDL boundaries use the configured driver mode.
- [x] Existing query-row-limit and SQLite batch paths choose their driver mode.
- [ ] Independent PR review and provider runtime checks are recorded; no unsupported runtime claim.
