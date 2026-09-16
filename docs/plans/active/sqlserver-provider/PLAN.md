# SQL Server provider (#260)

## State

`RUNTIME_VERIFY`

## Goal

Add a real SQL Server provider adapter behind the existing native provider seam so
connection, query, canonical value decoding, schema introspection, table data
operations, capability gating, and SQL Server identifier/parameter syntax are
implemented by the provider rather than by UI branches.

## Evidence baseline

- Baseline: `cc399501338b7f38342a1bb12b9990e048537312`
- GitHub issue: #260
- Dependency: #234 provider contract is present in the codebase, but its live
  acceptance status is not assumed from historical release notes.
- Existing gap: `DriverType` and `CompositeConnector` have no SQL Server arm;
  workspace `sqlx` has no TDS feature.
- Implementation SHA: `774a4e7f6ce8d88939988711e8fadf0588f1c3c1`.

## Scope

- Add a SQL Server driver identity and provider factory.
- Add a TDS-backed connector using the existing `DbConnector` port.
- Implement typed query parameters and canonical row decoding without lossy
  conversion for decimal, bigint, uniqueidentifier, datetimeoffset, binary, or
  JSON-like values.
- Implement SQL Server dialect quoting and parameter placeholders.
- Implement schema/table/column/PK/FK/index/check/trigger/view/routine
  introspection through SQL Server catalogs.
- Reuse the existing runtime/UI provider dispatch and capability model.
- Add focused unit/conformance coverage and a fixture-driven integration lane.

## Non-goals

- PL/pgSQL debugging (#252).
- SQL Server-specific administration, backup/restore, or security workspaces
  beyond explicit capability refusals.
- Claiming issue completion without a live SQL Server fixture and native UI
  evidence where the issue requires them.

## Acceptance criteria

1. SQL Server connection lifecycle and representative query execution work.
2. Query cancellation is either implemented safely or capability-gated with a
   user-facing reason.
3. Canonical values preserve exact decimal/bigint/uniqueidentifier/
   datetimeoffset/binary semantics.
4. Catalog introspection returns stable schemas, tables, columns, keys, indexes,
   checks, triggers, views, and routines where supported.
5. Table reads and mutations use the shared safety/change-set path and SQL Server
   dialect.
6. Unsupported PostgreSQL-only features are capability-gated with explicit
   reasons.
7. Provider/unit/conformance tests pass, and a live SQL Server fixture proves the
   supported matrix.

## Completion gate

The code and automated gates are complete at
`774a4e7f6ce8d88939988711e8fadf0588f1c3c1`. The plan remains under
`docs/plans/active/` because the environment has no live SQL Server fixture and
no native UI screenshots/runtime recording at the required sizes. The issue must
not be marked completed until those evidence lanes run.
