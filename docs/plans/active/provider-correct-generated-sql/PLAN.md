# Provider-correct generated SQL — #441 / #442

State: RUNTIME_VERIFY. MySQL #442 acceptance exercised on real provider; SQLite/PostgreSQL regression probes pass. SQLServer live gate for #441 remains pending; keep that issue open.
Baseline: `76b120724a7542de8b587a002811ddacad28156a`; branch: `main`; no PR.

## Requirements and context

- #441: reload a row using the complete caller-supplied primary key with dialect-correct identifiers/placeholders and unchanged parameter order; never emit SQL Server-incompatible `LIMIT`.
- #442: reconstruct CREATE TABLE columns, PK/CHECK/FK identifiers, referenced schema/table/columns and indexes using the existing `DriverDialect` quoting policy. Preserve SQLite unqualified names/inline unique constraints, PostgreSQL authoritative index definitions/ALTER TABLE FKs, and trigger behavior.
- Decision: existing PK validation checks nonempty/equal-length key/value lists, not catalog membership, so dropping the existing row cap would rely on an unproved uniqueness assumption. Reuse `SqlDialect::pagination_clause("1", "0")` and its ORDER BY requirement instead; SQL Server orders by the supplied PK columns and uses OFFSET/FETCH, other providers retain LIMIT. Constants add no bound parameters and no port change.
- Reconstructed index names must be unqualified (the table supplies the schema); non-PostgreSQL FKs must remain inline, using provider quoting.
- Reviewed FK producer/consumer contracts: normalize SQL Server catalog action descriptors before they reach domain consumers; retain MySQL REFERENCED_TABLE_SCHEMA with database-scoped catalog joins. Declare MySQL indexes inline before FKs to reuse supporting indexes instead of duplicate-index creation.

## Scope and non-goals

Own mutation/schema DDL generation, directly necessary core dialect constructor/callers/adjacent tests, MySQL/SQL Server FK introspection and adjacent provider tests, and this directory (expanded by parent after independent review). No ports, manifests, dependencies, shared changelog/status, data_diff, trigger redesign, arbitrary SQL-expression rewriting, or changes to identity/generated-column syntax. Existing MySQL functional-index metadata limitations are not presented as supported expression reconstruction.

## Work sequence and definition of done

1. Inspect dialect helpers, DDL consumers, tests and PK reload contract; record baseline evidence.
2. Add behavior-level regression cases for composite reload SQL/params and public SchemaService output across all four drivers; preserve native PostgreSQL indexes and SQLite/PG FK options.
3. Reuse `DriverDialect` for every reconstructed identifier, remove obsolete quote helper/import, keep provider schema rules, and route reload pagination through the existing dialect hooks.
4. Integration owner runs focused tests, workspace gates and separate real-provider public-service smoke before commit/push/issue closure. Source/unit coverage is not live SQL Server/MySQL evidence.

DoD: all requested SQL identifiers and ordered PK parameters covered; no port changes or duplicate quoting convention; independent provider smoke recorded by integration owner; no unverified completion claim.
