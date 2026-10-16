# Verification — #441 / #442

Baseline: `76b120724a7542de8b587a002811ddacad28156a`. Candidate: uncommitted changes on shared `main`; parent supplies final SHA.

## Parent execution — authoritative integration update

- Baseline exact-source public runtime MySQL DDL probe failed with server error 1064 on double-quoted CREATE TABLE identifiers (`artifact://84`), reproducing #442 on MySQL 8.4.11 with ANSI_QUOTES absent.
- Candidate standalone public DbProRuntime cases `pk` and `ddl`: PASS separately on real SQLite, PostgreSQL 18 and MySQL 8.4.11. The DDL case introspects/reconstructs, drops/recreates, then proves composite PK/FK/CHECK/unique-index semantics by actual SQL mutations and re-introspection. SQLite fixture corrections use unqualified FK/index table targets; early fixture syntax errors do not count as product regressions.
- `DATABASE_URL=mysql://...@127.0.0.1:25433/dbpro_smoke MYSQL_REFERENCE_DATABASE=dbpro_reference cargo test -p db-pro-infrastructure --test mysql_integration mysql_cross_database_foreign_key_ddl_round_trips -- --ignored --exact --nocapture`: 1 passed / 0 failed / 0 ignored, exit 0 (`artifact://109`). Same-named objects/constraints in two databases verify the reconstructed FK points to remote parent, not a misleading local parent.
- Independent ReviewDataSql initially found two consumer-contract defects; fixed SQLServer catalog action spelling and MySQL referenced database/join scoping, plus supporting-index declaration order. Focused re-review ACCEPT, no unresolved introduced findings.
- A newly exposed inherited PostgreSQL INTEGER/Int64 binding failure is tracked separately as #533; its initial public PK probe failed. Three actual PG regression cases and the original INTEGER public probe pass on that candidate, not via changing the fixture to BIGINT. Final #533 correction/review remains separately tracked.
- SQLServer live runtime has NOT been exercised on this host. No reachable server/authorized fixture is available; do not infer SQLServer behavior from PostgreSQL/MySQL/SQLite. Keep #441 OPEN for its named SQLServer runtime gate even when code/tests are published.
- #442 MySQL-specific acceptance is exercised on the real default-mode provider and may close after publish. Functional-index expression introspection remains inherited/out of scope.
- Final integrated gates/quality failures are recorded by parent, not inferred from mock-based core tests. Historical worker NOT RUN statements below describe worker execution only.


## Executed evidence

Source inspection and issue/document retrieval only. Build, tests, lint, formatting, smoke and provider commands: **NOT RUN**, explicitly delegated to parent. No pass/fail/ignored counts are available. CI: not run.

## Integration-owner focused commands

```sh
cargo test -p db-pro-core select_by_composite_pk_uses_each_provider_pagination
cargo test -p db-pro-core select_by_pk_rejects_missing_or_mismatched_keys
cargo test -p db-pro-core get_table_ddl_quotes_special_identifiers_for_each_provider
cargo test -p db-pro-core ddl_preserves_postgres_native_index_and_deferred_composite_fk
cargo test -p db-pro-core sqlserver_ddl_preserves_canonical_foreign_key_actions
cargo test -p db-pro-core mysql_ddl_declares_supporting_indexes_before_foreign_keys_once
cargo test -p db-pro-infrastructure catalog_foreign_key_actions_use_sql_spellings
cargo test -p db-pro-core application::schema_service::tests
cargo test -p db-pro-core application::sql_builder::tests
```

The provider matrix uses real generation/service code with literal SQL expectations, but core service dependencies are mocks. It proves generated text/ordered parameters, **not** provider runtime behavior. Existing tests separately retain SQLite inline unique/FK/CHECK handling, PostgreSQL FK grouping and trigger full-definition/function ordering.

| Provider | Builder/service coverage added | Live evidence |
|---|---|---|
| PostgreSQL | composite PK SQL/params; special/reserved names; composite FK/actions, native index definition/deferred FK | NOT RUN |
| SQLite | composite PK SQL/params, empty schema; special/reserved DDL names, unqualified FK/index target | NOT RUN |
| MySQL | composite PK SQL/params; escaped backticks everywhere; inline composite FK and unqualified index name | NOT RUN |
| SQL Server | composite PK SQL/params with PK ORDER BY and OFFSET/FETCH; escaped brackets everywhere; inline composite FK and unqualified index name | NOT RUN |

## Independent public-service/live smoke

This recipe deliberately uses actual runtime services and providers, not mocks. It is **unexecuted** handoff material. Only run against a disposable writable database; the probe creates/drops names beginning `dbpro441`. Use one provider per fresh runtime state directory. SQL Server and MySQL require reachable servers, credentials, CREATE/DROP/INSERT rights and catalog visibility; configure TLS to match the fixture. SQLite needs only an empty scratch file; PostgreSQL needs a disposable database/schema. An ignored provider test that silently skips for missing environment is not evidence.

Copy the Rust block below to a temporary `crates/runtime/examples/provider_generated_sql_probe.rs`; remove that temporary file after parent verification. No permanent probe file or manifest change is needed. Example invocation (supply actual fixture values; do not print credentials):

```sh
PROBE_STATE="$(mktemp -d)" \
PROBE_CONFIG='{"name":"generated-sql-probe","host":"127.0.0.1","port":3306,"database":"disposable_db","username":"probe","driver":"mysql","ssl_mode":"disable","ssh_tunnel":null}' \
PROBE_PASSWORD="$MYSQL_PROBE_PASSWORD" PROBE_CASE=ddl \
cargo run -p db-pro-runtime --example provider_generated_sql_probe
```

Run `PROBE_CASE=pk` and `PROBE_CASE=ddl` independently for `postgres`, `sqlite`, `mysql`, `sqlserver` (driver JSON names). For SQLite use a fresh absolute database file, empty username/host and port 0. Default schemas are `public`, `main`, configured MySQL database and `dbo`; `PROBE_SCHEMA` can select an existing disposable schema. SQL Server commonly needs port 1433 and appropriate `ssl_mode`. The MySQL fixture must have `ANSI_QUOTES` off; SQL Server may use `QUOTED_IDENTIFIER OFF` to demonstrate brackets do not depend on that setting. Session modes must apply to every pooled connection, not just one arbitrary query.

```rust
use db_pro_core::application::sql_builder::qualify;
use db_pro_core::domain::connection::{ConnectionConfig, DriverType};
use db_pro_core::domain::query::CellValue;
use db_pro_core::ports::DbConnector;
use db_pro_runtime::DbProRuntime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ConnectionConfig = serde_json::from_str(&std::env::var("PROBE_CONFIG")?)?;
    let driver = config.driver;
    let schema = std::env::var("PROBE_SCHEMA").unwrap_or_else(|_| match config.driver {
        DriverType::Postgres => "public".into(),
        DriverType::SQLite => "main".into(),
        DriverType::Mysql => config.database.clone(),
        DriverType::SqlServer => "dbo".into(),
    });
    let mode = std::env::var("PROBE_CASE")?;
    assert!(mode == "pk" || mode == "ddl");
    let runtime = DbProRuntime::new(std::env::var("PROBE_STATE")?).await?;
    let connection = runtime.connections().create(config, &std::env::var("PROBE_PASSWORD").unwrap_or_default()).await?;
    let id = connection.id;
    let handle = runtime.connections().connect(&id).await?;
    let connector = runtime.connector();
    let dialect = connector.dialect(&handle)?;
    let name = "dbpro441`\"]child";
    let parent_name = "dbpro441`\"]parent";
    let target = qualify(dialect.as_ref(), &schema, name);
    let parent = qualify(dialect.as_ref(), &schema, parent_name);
    let foreign_parent = if driver == DriverType::SQLite {
        dialect.quote_identifier(parent_name)
    } else {
        parent.clone()
    };
    let tenant = dialect.quote_identifier("tenant");
    let part = dialect.quote_identifier("part");
    let amount = dialect.quote_identifier("amount");
    let check = dialect.quote_identifier("dbpro441`\"]check");
    let foreign_key = dialect.quote_identifier("dbpro441`\"]fk");
    let index = dialect.quote_identifier("dbpro441`\"]idx");
    connector.execute(&handle, &format!(
        "CREATE TABLE {parent} ({tenant} INTEGER NOT NULL, {part} INTEGER NOT NULL, PRIMARY KEY ({tenant}, {part}))"
    ), &[]).await?;
    connector.execute(&handle, &format!(
        "CREATE TABLE {target} ({tenant} INTEGER NOT NULL, {part} INTEGER NOT NULL, {amount} INTEGER NOT NULL, \
         PRIMARY KEY ({tenant}, {part}), CONSTRAINT {check} CHECK ({amount} > 0), \
         CONSTRAINT {foreign_key} FOREIGN KEY ({tenant}, {part}) REFERENCES {foreign_parent} ({tenant}, {part}) ON DELETE CASCADE)"
    ), &[]).await?;
    let index_target = if driver == DriverType::SQLite {
        dialect.quote_identifier(name)
    } else {
        target.clone()
    };
    connector.execute(&handle, &format!("CREATE UNIQUE INDEX {index} ON {index_target} ({amount})"), &[]).await?;
    connector.execute(&handle, &format!("INSERT INTO {parent} VALUES (10,20), (10,21), (10,22)"), &[]).await?;
    if mode == "ddl" {
        runtime.schema().introspect(&id, true).await?;
        let ddl = runtime.schema().get_table_ddl(&id, &schema, name).await?;
        println!("{ddl}");
        connector.execute(&handle, &format!("DROP TABLE {target}"), &[]).await?;
        runtime.schema().execute_ddl(&id, &ddl).await?;
    }
    connector.execute(&handle, &format!("INSERT INTO {target} VALUES (10,20,1), (10,21,2)"), &[]).await?;
    if mode == "pk" {
        let keys = vec!["tenant".into(), "part".into()];
        let values = vec![CellValue::Int64(10), CellValue::Int64(21)];
        let row = runtime.table_data().fetch_row_by_pk(&id, &schema, name, &keys, &values).await?.unwrap();
        assert_eq!(row.rows.len(), 1);
        assert!(matches!(row.rows[0].0[2], CellValue::Int64(2)));
        let missing = vec![CellValue::Int64(10), CellValue::Int64(22)];
        assert!(runtime.table_data().fetch_row_by_pk(&id, &schema, name, &keys, &missing).await?.is_none());
    } else {
        for values in ["(10,22,-1)", "(99,1,4)", "(10,22,1)"] {
            assert!(connector.execute(&handle, &format!("INSERT INTO {target} VALUES {values}"), &[]).await.is_err());
        }
        runtime.schema().introspect(&id, true).await?;
        let info = runtime.schema().get_table_info(&id, &schema, name).await?;
        assert_eq!(info.primary_key.unwrap().columns, vec!["tenant", "part"]);
        assert_eq!(info.foreign_keys.len(), 1);
        assert!(info.indexes.iter().any(|item| item.name == "dbpro441`\"]idx" && item.unique));
    }
    connector.execute(&handle, &format!("DROP TABLE {target}"), &[]).await?;
    connector.execute(&handle, &format!("DROP TABLE {parent}"), &[]).await?;
    runtime.connections().disconnect(&id).await?;
    println!("provider generated SQL {mode}: PASS");
    Ok(())
}
```

On failure, retain the actual error and generated SQL, then remove leftover `dbpro441` tables only in the disposable fixture (child first, parent second). The probe does not silently clean up on failure because doing so could mask evidence. It is not a UI-flow verification.

## Baseline reproduction approach

Against baseline `76b120724a7542de8b587a002811ddacad28156a`, use the same public-service probe with separate cases: SQL Server `pk` reaches the hardcoded LIMIT failure (#441); MySQL default-mode `ddl` reaches rejected double-quoted CREATE TABLE identifiers (#442). Do not reset shared main: parent can use a disposable checkout/copy of the baseline or inject the baseline generated statements through the live connector. The new focused test cases are expected to fail at baseline; that RED result has **not been executed** by this worker. Additional baseline DDL failures (schema-qualified index names / missing MySQL and SQL Server FKs) are related acceptance defects, not regressions from the proposed patch.

## Exact optional fixture commands and provider invocations

All commands below are **unexecuted worker recipes**, not evidence. Parent reported an existing disposable PostgreSQL 18 fixture at `127.0.0.1:25432` (`dbpro_smoke`, user `dbpro`); commands use that fixture without altering its server. Suggested extra host ports 25433/25434 are loopback-only and avoid the existing PostgreSQL binding. Container passwords below are disposable fixture values only. SQL Server's developer image requires accepting Microsoft's EULA and sufficient local Docker resources; record pull/start/resource failures instead of calling SQL Server verified.

### MySQL 8.4 disposable container

```sh
docker run --rm -d --name dbpro-generated-mysql \
  -p 127.0.0.1:25433:3306 \
  -e MYSQL_ROOT_PASSWORD=dbpro_root_fixture_only \
  -e MYSQL_DATABASE=dbpro_smoke \
  -e MYSQL_USER=dbpro -e MYSQL_PASSWORD=dbpro_fixture_only \
  --health-cmd='mysqladmin ping -h 127.0.0.1 -uroot -p"$MYSQL_ROOT_PASSWORD" --silent' \
  --health-start-period=20s --health-interval=3s --health-timeout=2s --health-retries=60 \
  mysql:8.4 --sql-mode=STRICT_TRANS_TABLES,NO_ENGINE_SUBSTITUTION
docker inspect --format '{{.State.Health.Status}}' dbpro-generated-mysql
```

Wait for `healthy` before using it. Record server version and confirm `ANSI_QUOTES` is absent:

```sh
docker exec dbpro-generated-mysql mysql -udbpro -pdbpro_fixture_only dbpro_smoke \
  -e 'SELECT VERSION(), @@GLOBAL.sql_mode, @@SESSION.sql_mode'
DATABASE_URL='mysql://dbpro:dbpro_fixture_only@127.0.0.1:25433/dbpro_smoke' \
cargo test -p db-pro-infrastructure --test mysql_integration mysql_connects_and_executes_query -- --ignored --exact --nocapture
DATABASE_URL='mysql://dbpro:dbpro_fixture_only@127.0.0.1:25433/dbpro_smoke' \
cargo test -p db-pro-infrastructure --test mysql_integration mysql_positional_parameters_bind_and_round_trip -- --ignored --exact --nocapture
```

These existing ignored tests check fixture connectivity/binding only. They do **not** replace the #441/#442 public-service probes below. The full ignored MySQL/PG suites may require seeded repository fixtures; do not assume an empty scratch database satisfies those unrelated tests.

### SQL Server 2022 disposable container

```sh
docker run --rm -d --name dbpro-generated-sqlserver \
  -p 127.0.0.1:25434:1433 \
  -e ACCEPT_EULA=Y -e MSSQL_PID=Developer \
  -e 'MSSQL_SA_PASSWORD=Dbpro_fixture_Only!441' \
  --health-cmd='/opt/mssql-tools18/bin/sqlcmd -C -S localhost -U sa -P "$MSSQL_SA_PASSWORD" -Q "SELECT 1" -b -o /dev/null' \
  --health-start-period=30s --health-interval=3s --health-timeout=3s --health-retries=60 \
  mcr.microsoft.com/mssql/server:2022-latest
docker inspect --format '{{.State.Health.Status}}' dbpro-generated-sqlserver
```

After `healthy`, create the disposable database and record the actual version:

```sh
docker exec dbpro-generated-sqlserver /opt/mssql-tools18/bin/sqlcmd \
  -C -S localhost -U sa -P 'Dbpro_fixture_Only!441' -b \
  -Q 'CREATE DATABASE dbpro_smoke; SELECT @@VERSION'
SQLSERVER_URL='sqlserver://sa:Dbpro_fixture_Only%21441@127.0.0.1:25434/dbpro_smoke' \
cargo test -p db-pro-infrastructure --test sqlserver_integration sqlserver_connects_and_queries -- --ignored --exact --nocapture
SQLSERVER_URL='sqlserver://sa:Dbpro_fixture_Only%21441@127.0.0.1:25434/dbpro_smoke' \
cargo test -p db-pro-infrastructure --test sqlserver_integration sqlserver_preserves_parameterized_decimal_and_bigint -- --ignored --exact --nocapture
```

Some older SQL Server images put sqlcmd at `/opt/mssql-tools/bin/sqlcmd`; if the health log reports a missing executable, use that installed path in both healthcheck and exec commands. A provider test's “skipping ... unavailable” message is not a PASS. Baseline `76b120724a7542de8b587a002811ddacad28156a`, `sqlserver/connector.rs:152-156`, makes `ssl_mode=require` request encryption while trusting the disposable server certificate; the probe JSON below selects that existing behavior.

### Exact public-service probe runs

First materialize the Rust example above; no manifest/dependency edits are needed (`db-pro-runtime` already depends on core, infrastructure, tokio and serde_json at baseline `76b120724a7542de8b587a002811ddacad28156a`). Each loop executes two independent cases sequentially with fresh runtime metadata. Do not run the cases concurrently against the same scratch schema because they intentionally use the same object names.

```sh
# Existing PostgreSQL fixture supplied by parent.
for case in pk ddl; do
  PROBE_STATE="$(mktemp -d)" PROBE_CASE="$case" \
  PROBE_CONFIG='{"name":"generated-sql-pg","host":"127.0.0.1","port":25432,"database":"dbpro_smoke","username":"dbpro","driver":"postgres","ssl_mode":"disable","ssh_tunnel":null}' \
  PROBE_PASSWORD=dbpro_fixture_only \
  cargo run -p db-pro-runtime --example provider_generated_sql_probe || break
done

# SQLite: distinct absolute fresh database for each case.
for case in pk ddl; do
  scratch="$(mktemp -d)"
  PROBE_STATE="$scratch/state" PROBE_CASE="$case" PROBE_PASSWORD='' \
  PROBE_CONFIG="{\"name\":\"generated-sql-sqlite\",\"host\":\"\",\"port\":0,\"database\":\"$scratch/provider.sqlite\",\"username\":\"\",\"driver\":\"sqlite\",\"ssl_mode\":\"disable\",\"ssh_tunnel\":null}" \
  cargo run -p db-pro-runtime --example provider_generated_sql_probe || break
done

# MySQL container above; ANSI_QUOTES explicitly absent at server startup.
for case in pk ddl; do
  PROBE_STATE="$(mktemp -d)" PROBE_CASE="$case" \
  PROBE_CONFIG='{"name":"generated-sql-mysql","host":"127.0.0.1","port":25433,"database":"dbpro_smoke","username":"dbpro","driver":"mysql","ssl_mode":"disable","ssh_tunnel":null}' \
  PROBE_PASSWORD=dbpro_fixture_only \
  cargo run -p db-pro-runtime --example provider_generated_sql_probe || break
done

# SQL Server container above.
for case in pk ddl; do
  PROBE_STATE="$(mktemp -d)" PROBE_CASE="$case" \
  PROBE_CONFIG='{"name":"generated-sql-sqlserver","host":"127.0.0.1","port":25434,"database":"dbpro_smoke","username":"sa","driver":"sqlserver","ssl_mode":"require","ssh_tunnel":null}' \
  PROBE_PASSWORD='Dbpro_fixture_Only!441' \
  cargo run -p db-pro-runtime --example provider_generated_sql_probe || break
done
```

Require **both** explicit `provider generated SQL pk: PASS` and `provider generated SQL ddl: PASS` for each provider; a shell loop's final status alone is insufficient. Save generated DDL and actual command/errors. After collecting evidence, remove the temporary example and only the two newly created disposable containers (`docker stop dbpro-generated-mysql dbpro-generated-sqlserver`); do not stop parent's PostgreSQL fixture. Fresh scratch metadata/database directories can be removed after runtime handles close.

## Independent-review FK regression gates

The following new tests are **not run by this worker**. The MySQL test requires a distinct existing database granted to the same user; parent reported provisioning `dbpro_reference`. It creates uniquely named same-name parent/child/constraint fixtures in both databases, verifies composite metadata has no cross-database join multiplication, reconstructs the child through actual SchemaService, and proves only the remote parent's key satisfies the recreated FK. It also reproduces supporting-index duplication unless indexes are preserved inline before MySQL creates the FK. It drops its own tables on success.

```sh
DATABASE_URL='mysql://dbpro:dbpro_fixture_only@127.0.0.1:25433/dbpro_smoke' \
MYSQL_REFERENCE_DATABASE=dbpro_reference \
cargo test -p db-pro-infrastructure --test mysql_integration mysql_cross_database_foreign_key_ddl_round_trips -- --ignored --exact --nocapture

SQLSERVER_URL='sqlserver://sa:Dbpro_fixture_Only%21441@127.0.0.1:25434/dbpro_smoke' \
cargo test -p db-pro-infrastructure --test sqlserver_integration sqlserver_foreign_key_actions_round_trip_through_schema_service -- --ignored --exact --nocapture
```

SQL Server's new ignored regression requires a live writable SQL Server database and deliberately fails rather than silently skipping if explicitly selected without one. It creates unique fixtures for NO ACTION, SET NULL and SET DEFAULT, verifies domain metadata uses spaced SQL spellings, and executes reconstructed table DDL through SchemaService. SQL Server runtime remains pending parent fixture availability. Parent separately reported baseline MySQL generated-DDL error 1064 (`artifact://84`), MySQL 8.4.11 with ANSI_QUOTES absent, and a PostgreSQL INTEGER/Int64 binding failure being investigated in the parent lane. These reports are not worker-executed results and do not establish candidate success.
