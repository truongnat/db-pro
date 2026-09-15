# Provider contract

> Status: **Implemented** (provider seam `0729d699`; MySQL provider `640eaf4c`; capability
> truthfulness and the value/decoder conformance case added 2026-09-15, issue #234)

## Source

- Factory port: `crates/core/src/ports/provider_factory.rs` (`ProviderFactory`)
- Connector port: `crates/core/src/ports/db_connector.rs` (`DbConnector`), `crates/core/src/ports/dialect.rs` (`SqlDialect`)
- Capability model: `crates/core/src/domain/capabilities.rs` (`DatabaseCapabilities`)
- Registry and dispatch: `crates/infrastructure/src/connector.rs` (`CompositeConnector`)
- Providers: `crates/infrastructure/src/{postgres,sqlite,mysql}/`
- Value contract: `docs/release/provider-value-contract.md`
- Conformance suite: `crates/infrastructure/tests/provider_contract_conformance.rs`

## What a provider is

A provider is one engine's implementation of two ports plus a capability advertisement:

```text
ProviderFactory  → driver(), capabilities(config), build() → Box<dyn DbConnector>, test_connection()
DbConnector      → connect / disconnect / test_connection
                   query / execute / cancel / execute_batch
                   execute_transaction / execute_parameterized_transaction
                   introspect / explain / dialect
```

Service- and UI-level behaviour is **not** part of the provider: a provider supplies the
engine-facing half only. Everything above `DbConnector` is shared, which is what
keeps a second engine from needing a second UI stack.

## Registration and dispatch

`CompositeConnector::new()` (`crates/infrastructure/src/connector.rs`) registers one factory per
driver in a `HashMap<DriverType, Box<dyn ProviderFactory>>`; `register_factory()` inserts or
replaces an entry for a driver and returns the previous one, so a test can install a stub and
restore the original.

The composite is the only dispatch point for connections:

| Call | Dispatch |
|---|---|
| `connect(config, password)` | factory for `config.driver` builds the connector; the handle records the driver |
| `test_connection` | the factory's own `test_connection` |
| `query` / `execute` / `cancel` / `execute_batch` / `execute_transaction` / `execute_parameterized_transaction` / `introspect` / `explain` | cloned connector for that handle |
| `dialect(handle)` | per-driver `SqlDialect` (PostgreSQL `$n`, SQLite/MySQL `?`) |
| `capabilities(config)` | the factory's capability set, falling back to `DatabaseCapabilities::for_driver` |

`DriverType` is a closed enum, so provider registration is a compile-time act: adding an engine
means a new enum variant, a factory, a connector implementation, a capability set and a UI
capability entry. There is no dynamic plugin loading, and none is required.

## The capability model

`DatabaseCapabilities` (`crates/core/src/domain/capabilities.rs`) is the contract consumers gate
on, grouped into `query`, `schema`, `data` and `features`. It is the answer to "may I offer this
action?" and is deliberately **UI-independent**: a view reads the flags, it does not branch on
the driver's name.

### What `true` means

A flag is `true` only when a shipping code path serves that capability for that driver. Where the
engine could do the work but the product's only path rejects the driver, the flag is `false`, and
the reason is recorded in the capability's doc comment together with the code path it was measured
against. `DatabaseCapabilities::mysql()` is written this way: `parameters`/`positional_parameters`
are false because the connector drops the parameter list, and `server_sessions`, `partitions`,
`backup` and `data_diff` are false because their only product paths reject MySQL. The rule exists
so a capability-driven consumer can never be told to offer an action the provider will refuse.

### What a `false` flag carries

Each `false` flag also has a capability-level reason via
`DatabaseCapabilities::limitation(CapabilityFeature)` (#234 criterion 3). The UI resolves
gated actions through `CapabilityLookup::feature_limitation`, so consumers get a named
reason instead of inventing one from a boolean. Operation-level `DbError::Unsupported`
remains the second line of defence when a caller bypasses the capability gate.

Consumers resolve flags through the lookup that owns the naming, not by re-deriving them per
view: `CapabilityLookup` (`crates/ui/src/app.rs`) resolves a connection's driver label to a
provider entry and returns `Supported`, `NoActiveConnection` or `UnsupportedDriver { driver }`,
with `allows(..)` for gating. A lookup that cannot answer never satisfies a capability predicate.

## The value-decoding contract

Every provider's query mapper must decode result cells into the domain `CellValue` classes exactly
as `docs/release/provider-value-contract.md` defines them — that document is the contract, and it
is not restated here. The rules a new provider is measured against, from the same document:

1. exact numeric classes stay exact (`Int64`/`Decimal`, never an `f64` hop);
2. a timestamp without time zone never gains an invented offset, and an instant keeps its marker;
3. one unsupported class degrades to a byte-exact cell without taking the row down;
4. `NULL` is preserved per family.

Decoding is **class-aware**: the mapper dispatches on the class the provider reports for the
column. Probing a cell by "try `bool`, then `i64`, then `f64`" is not a valid implementation — the
MySQL mapper did that and served every integer fitting in an `i8` as a boolean, including
`SELECT count(*)` (`docs/release/evidence/v01-runtime/providers/60-mysql-live-fixture-and-mapper.md`,
defect D1).

Proof for a provider is a fixture plus live tests, not a code reading:

| Provider | Decoder fixture | Live matrix |
|---|---|---|
| PostgreSQL | `fixtures/postgres/` (`decoder_matrix`, 26 classes) | `docs/release/evidence/v01-runtime/providers/40-pg-decoder-matrix-fixture.md` |
| MySQL 8 | `fixtures/mysql/` (`decoder_matrix`, 41 columns) | `providers/60-…` and `crates/infrastructure/tests/mysql_fixture_matrix.rs` |
| SQLite | in-memory, `crates/infrastructure/tests/integration.rs` | `providers/02`–`05` |

`provider_value_decoder_contract_holds_against_the_fixture`
(`crates/infrastructure/tests/provider_contract_conformance.rs`) runs the class-aware assertions
against whichever provider `DATABASE_URL` names and self-skips with a reason when it names none.

## What a new provider must supply

A provider is **supported** when all of the following exist and are verified on `main`:

| # | Required | Reference implementation |
|---|---|---|
| 1 | A `ProviderFactory` impl registered in `CompositeConnector::new()` for its `DriverType` | `PostgresFactory`/`SqliteFactory`/`MySqlFactory` (`crates/infrastructure/src/connector.rs`) |
| 2 | A `DbConnector` impl covering the full trait surface; a trait method the engine cannot serve returns `DbError::Unsupported` with a named reason rather than an empty success | MySQL `cancel` |
| 3 | A class-aware query mapper satisfying the value contract | `postgres/query_mapper.rs`, `mysql/query_mapper.rs` |
| 4 | An introspector producing the canonical schema model (`IntrospectResult`: schemas, tables, columns, PKs, FKs, indexes, triggers, views, routines) | `postgres/introspect.rs`, `mysql/introspect.rs` |
| 5 | A `SqlDialect` arm (placeholder + identifier quoting + pagination shape) for the paths that build SQL | `PostgresDialect`, `SqliteDialect`, `MySqlDialect` |
| 6 | A capability set whose every `true` flag names a shipping code path | `DatabaseCapabilities::mysql()` |
| 7 | A committed fixture and live tests that run against a real server, self-skipping with a reason when `DATABASE_URL` is absent | `fixtures/mysql/` + `mysql_fixture_matrix.rs` |
| 8 | A UI capability entry: the driver label the runtime stores maps to the provider entry in `CapabilityLookup::for_driver_label` | `crates/ui/src/app.rs` |

Items not on this list are product work, not provider work (user management, backup, partitions,
data diff): those services own their own driver gates, and a provider that cannot serve them must
advertise the corresponding capability as `false` (item 6) instead of failing at call time.

## Known gaps in this contract

Recorded rather than implied, so a consumer cannot mistake them for guarantees:

1. **Capability reason channel** — implemented as `CapabilityFeature` +
   `DatabaseCapabilities::limitation` / `CapabilityLookup::feature_limitation` (#234 criterion 3).
2. **The UI still resolves capabilities from a driver label.** `CapabilityLookup::for_driver_label`
   maps `PostgreSQL`/`SQLite`/`MySQL`, and a label with no entry is the explicit
   `UnsupportedDriver` state, but the lookup is a string mapping rather than a query to the
   registered factories, and several UI paths still branch on the provider name where a capability
   would do (`query_view.rs` formatting/parsing dialect, schema defaults, `connection_service.rs`,
   `user_service.rs`) — #234 criterion 2 remainder.
3. **PostgreSQL is still reached directly** in places (`crates/runtime/src/api.rs` object
   dependencies, partitions, tablespaces, rename) instead of through the composite.
4. ~~**MySQL has no dialect arm**~~ — `CompositeConnector::dialect` returns `MySqlDialect`;
   table-data change-set and data-diff paths are reachable (#235).
5. ~~**MySQL has no parameter binder**~~ — positional `?` binder ships; capability flags are `true`.
