---
schema_version: 1
id: "MEM-PLAN-provider-correct-generated-sql-0393fcc6fab6"
title: "Plan archive: Provider Correct Generated Sql"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, provider-correct-generated-sql]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "0393fcc6fab6cf5a24bbb6541a0d9fb6950198845b41b3cf602e35f1a1a4ed0f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/provider-correct-generated-sql/PLAN.md"
  - "docs/plans/active/provider-correct-generated-sql/FINDINGS.md"
  - "docs/plans/active/provider-correct-generated-sql/VERIFICATION.md"
  - "docs/plans/active/provider-correct-generated-sql/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/provider-correct-generated-sql/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — #441 / #442

All repository source observations in this file refer to baseline `76b120724a7542de8b587a002811ddacad28156a` (not runtime observations).

| Severity | Baseline evidence | Failure and bounded decision |
|---|---|---|
| P1 / #441 | `crates/core/src/application/sql_builder/mutation.rs:94-110` appends `LIMIT 1`; `table_data_service.rs:103-121` submits the result unchanged. | SQL Server cannot execute this reload SQL. Existing key validation checks shape, not catalog uniqueness, so preserve the row cap via `SqlDialect::pagination_clause("1", "0")`. Add PK-column ORDER BY only when the dialect requires it; preserve key/value parameter order, with no added parameters or port changes. |
| P2 / #442 | `schema_ddl.rs:251-254` hardcodes double quotes throughout reconstruction; `schema_service.rs:197-214` already passes the actual connection driver. | MySQL default mode rejects reconstructed identifiers. Reuse `driver_dialect.rs:40-45` rather than retain a second quoting convention. A small crate-visible typed constructor avoids converting DriverType to a string. |
| P2 / related DDL syntax | `schema_ddl.rs:188-205` schema-qualifies reconstructed index names for every non-SQLite driver. | CREATE INDEX requires an unqualified index name for PostgreSQL/MySQL/SQL Server. Keep target table qualification; preserve authoritative PostgreSQL `index.definition` verbatim. |
| P2 / acceptance prerequisite | `schema_ddl.rs:108-116,208-219` only emits SQLite and PostgreSQL FKs; MySQL/SQL Server FKs are omitted. | Use the existing inline FK path for every non-PostgreSQL driver, with driver-correct target/constraint/column quoting. Keep composite order and existing action formatting. This is necessary for the requested FK-preserving reconstruction. |

## Consumer and invariant inventory

- Baseline `76b120724a7542de8b587a002811ddacad28156a`: `SchemaService::get_table_ddl` is the reconstruction consumer; views retain stored definitions, triggers use `format_trigger_ddl`. Helpers are private to `schema_service::schema_ddl`; the only direct old quote-helper consumer outside that module is its test import.
- Baseline `76b120724a7542de8b587a002811ddacad28156a`: `sql_builder/read.rs:16-38` already delegates pagination and SQL Server's ORDER BY requirement to `SqlDialect`. The other mutation builders have no LIMIT/OFFSET. No broad pagination changes are necessary.
- Baseline `76b120724a7542de8b587a002811ddacad28156a`: the reload builder/service accepts PK column/value lists without introspection and relies on the caller to supply the full real key. Reusing dialect pagination preserves the existing at-most-one-row behavior even when catalog uniqueness is not independently proven; no new catalog validation is added.
- Existing SQLite schema-prefix omission, inline unique constraints, PG/SQLite FK action/deferral formatting, raw CHECK/default/generated expressions and trigger definition/function ordering are intentionally preserved. Cross-dialect translation of SQL expression bodies and identity/generated-column syntax is outside #442.

## External SQL syntax references

Read on 2026-10-16; documentation evidence, not server execution:

- [PostgreSQL CREATE INDEX](https://www.postgresql.org/docs/current/sql-createindex.html): “No schema name can be included here”; index is created in its parent table's schema.
- [MySQL 8.4 CREATE INDEX](https://dev.mysql.com/doc/refman/8.4/en/create-index.html): `CREATE ... INDEX index_name ... ON tbl_name`.
- [SQL Server CREATE INDEX](https://learn.microsoft.com/en-us/sql/t-sql/statements/create-index-transact-sql?view=sql-server-ver17): `INDEX index_name ON <object>`; only the object accepts schema qualification.

## Remaining evidence boundary

Implementation and regression tests are handed off unexecuted. No claim of passing tests, live provider success or issue closure is made. Live SQL Server/MySQL require reachable writable disposable databases, credentials, the provider's expected TLS configuration and catalog visibility. Parent owns prerequisite discovery and all execution gates.

## Independent review corrections

ReviewDataSql blocked the first uncommitted candidate (schema DDL blob `6a56b45d9a14ed06d4eec068a32fbc21ed2efe2e`) on two newly consumed producer-contract defects. Parent classified the cross-database wrong-target integrity risk as P1. Both implementations and regression cases below are unexecuted worker corrections, not a claim that review/runtime now passes.

- **P2 SQL Server actions:** baseline `76b120724a7542de8b587a002811ddacad28156a`, `sqlserver/introspect.rs:247-280`, returns `NO_ACTION`, `SET_NULL`, `SET_DEFAULT` directly. Normalize descriptors to canonical spaced SQL actions in the producer, so all domain consumers see `NO ACTION` / `SET NULL` / `SET DEFAULT`. Added adjacent direct catalog-action unit coverage and live SchemaService recreation for all three action pairs. Also removed two inherited literal `// cc-scan` lines embedded inside that FK SQL statement: they were SQL payload, not Rust comments, and would prevent this producer path executing.
- **P1 MySQL referenced database:** baseline `76b120724a7542de8b587a002811ddacad28156a`, `mysql/introspect.rs:214-271`, omits REFERENCED_TABLE_SCHEMA and joins catalog rows on constraint name alone. Select/retain the actual referenced schema and scope joins by constraint schema, table and name. The ignored provider test creates identical constraint/table names across two databases and divergent local/remote parent keys; after SchemaService reconstruction, remote-only keys must succeed and local-only keys must fail.
- **Supporting-index interaction:** baseline `76b120724a7542de8b587a002811ddacad28156a`, `mysql/introspect.rs:158-207`, includes FK supporting indexes in Index metadata; the newly enabled inline FK branch creates its own index if not already declared. Emit MySQL indexes inline before FKs and suppress duplicate post-create index statements. Primary filtering and unique flags are retained; PostgreSQL native expression index SQL remains unchanged. Existing MySQL functional-index EXPRESSION metadata is not selected at baseline and is an inherited limitation, not claimed support.

Parent expanded ownership to these two introspection sources and adjacent tests. No port/dependency change or FK-dropping workaround was introduced. Review recheck and all gates remain integration-owner tasks.


## Source: `VERIFICATION.md`

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

Against baseline `76b120724a7542de8b587a002811ddacad28156a`, use the same public-service probe with separate cases: SQL Server `pk` reaches the hardcoded LIMIT failure (#441); MySQL default-mode `ddl` reaches rejected double-quoted CREATE TABLE identifiers (#442). Do not reset shared main: parent can use a disposable checkout/copy of the baseline or inject the baseline generated statements through the live connector. The new focused test cases are expected to fail at baseline; that RED result has **not been executed** by this worker. Additional baseline DDL failures (schema-qualified index names / missing MySQL and SQL Server FKs) are related acceptance defects, not regressions from the proposed patc

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — provider-correct generated SQL (#441 / #442)

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | ProviderSqlDialect — worker slice |
| Issues | #441, #442 |
| Task state | Review (implementation complete; gates delegated) |
| Baseline SHA | `76b120724a7542de8b587a002811ddacad28156a` — baseline source observations in FINDINGS.md |
| Branch / PR | shared `main`; no PR; parent alone commits/pushes/closes |
| Scope interpretation | PK reload SQL and provider-correct reconstructed table DDL, adjacent core tests and this plan directory |
| Out of scope | dialect ports, data_diff, manifests/dependencies, shared changelog/status, trigger redesign, expression/identity syntax translation |

## 2. Progress checkpoint

- Current candidate: uncommitted worker patch layered on baseline `76b120724a7542de8b587a002811ddacad28156a`; no final commit SHA assigned by worker.
- Completed acceptance: implementation, composite PK SQL/ordered-value regression cases, public SchemaService special/reserved identifier output matrix, PG native-index/deferred-FK guard, independent real-provider probe recipe.
- Remaining acceptance: parent executes tests and public-service/provider smoke; independent review; final integration evidence and issue closure.
- Findings/risks: inherited P1 #441 and P2 #442 plus related schema-qualified index/missing MySQL-SQL Server FK paths documented with baseline anchors in FINDINGS.md. Proposed patch addresses them; no executed proof yet.
- Tests run: none. Build/test/lint/format/cargo/smoke execution prohibited for this worker.
- Prerequisites: parent is preparing live fixtures; no database availability/credentials claim by worker. LSP was reported unavailable in the assignment, so literal source navigation was used. Semantic find also failed due unavailable judge account; literal fallback used.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline `76b120724a7542de8b587a002811ddacad28156a`; candidate is uncommitted (parent must supply exact final SHA) |
| Commit list | none by worker |
| File / surface inventory | `crates/core/src/application/sql_builder/mutation.rs` — preserve row cap via existing dialect pagination/order hooks; `driver_dialect.rs` — crate-visible typed constructor reused by existing name constructor; `schema_ddl.rs` — shared dialect quoting, inline non-PG FKs, valid index-name qualification, preserved native PG/trigger behavior; `schema_service.rs` — remove obsolete test-only quote import; `tests/sql_builder_tests.rs` — four-driver composite PK SQL/ordered parameters and malformed-key tests; `schema_service_tests.rs` — special/reserved identifier public-service matrix, native index/deferred composite FK guard, corrected reconstructed PG index expectation; this directory's PLAN/CHECKLIST/FINDINGS/VERIFICATION/AGENT_EVIDENCE |
| Expanded provider inventory | `crates/infrastructure/src/mysql/introspect.rs` — actual referenced schema and database/table-scoped catalog joins; `crates/infrastructure/src/sqlserver/introspect.rs` — canonical FK actions and removal of literal non-SQL comment text from FK query; `crates/infrastructure/src/sqlserver/tests/introspect_tests.rs` — catalog-action regression; `crates/infrastructure/tests/mysql_integration.rs` — cross-database SchemaService reconstruction and actual target-key enforcement; `crates/infrastructure/tests/sqlserver_integration.rs` — action metadata plus SchemaService reconstruction; `schema_ddl.rs`/`schema_service_tests.rs` additionally preserve MySQL supporting indexes inline before FK creation |
| Acceptance mapping | #441 → composite PK builder/value tests plus live `PROBE_CASE=pk`; #442 → public schema output matrix plus live `PROBE_CASE=ddl`, original SQLite/trigger suites retained |
| Commands and counts | NOT RUN; exact focused selectors and independent public-service probe are in VERIFICATION.md |
| CI run IDs / status | not run |
| Known limitations | no live-provider evidence; raw expressions/identity syntax remain provider-authored and untranslated; parent must execute and remove temporary probe source |
| Migrations / config implications | none; generated output changes, no persisted-format/schema migration |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | baseline `76b120724a7542de8b587a002811ddacad28156a`; first uncommitted DDL candidate blob `6a56b45d9a14ed06d4eec068a32fbc21ed2efe2e` reviewed by ReviewDataSql |
| Verdict | BLOCK on first candidate; corrections implemented, independent recheck pending |
| P0 / P1 / P2 counts | reviewer reported 0/0/2 introduced; parent upgraded cross-database wrong-target risk to P1 |
| Findings | SQL Server raw catalog action tokens and MySQL wrong referenced database; producer fixes and direct/runtime regression cases added, not executed by worker |
| CI disposition | all execution gates delegated, none reported passed |
| Next tasks unblocked | parent integration gates and separate provider-runtime verification for #441/#442 |

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Source references: baseline `76b120724a7542de8b587a002811ddacad28156a`, `crates/core/src/application/{driver_dialect.rs,schema_ddl.rs,schema_service.rs,table_data_service.rs,sql_builder.rs,sql_builder/read.rs,sql_builder/mutation.rs}`, adjacent test files, `docs/architecture/provider-contract.md`, canonical lifecycle/evidence templates. Complete external links and the exact baseline source anchors are in FINDINGS.md.
- Factual findings: baseline hardcoded reload LIMIT and DDL double quotes; DDL reconstruction has a DriverType; shared DriverDialect already implements provider escaping; reconstructed index names are qualified; MySQL/SQL Server FKs are omitted.
- Inference: core output tests cannot establish live-provider acceptance. Existing validation does not establish catalog uniqueness; retaining the row cap avoids introducing that assumption.
- Decision: reuse existing quoting and dialect pagination/order hooks without extending ports; retain provider-specific DDL representation and native SQL bodies. After independent review, normalize SQL Server action metadata, preserve actual MySQL referenced databases with scoped joins, and declare MySQL supporting indexes before FKs. Parent owns final review/runtime decision.
- Unresolved questions: no implementation question; live provider availability/TLS/configuration and execution results are pending parent.
- Downstream tasks: parent verifies #441 and #442 before closure.

## 6. Tổng kết

Đã sửa SQL tải lại hàng theo khóa chính để không phát `LIMIT` không hợp lệ trên SQL Server; DDL dùng chung `DriverDialect` để quote/escape đúng cho bốn provider, bỏ schema khỏi tên index và giữ FK của MySQL/SQL Server. Đã thêm test cho PK ghép, thứ tự tham số, định danh đặc biệt, CHECK/FK/index và giữ hành vi trigger. Chưa chạy bất kỳ gate hay provider thật nào theo phân công; cha cần chạy các selector và probe public-service trong VERIFICATION.md, ghi SHA/kết quả thật rồi mới đóng issue. Không sửa port, manifest, data_diff hoặc file dùng chung.

Review follow-up: Đã sửa hợp đồng metadata FK: SQL Server chuyển action có dấu gạch dưới sang cú pháp SQL chuẩn; MySQL giữ đúng database đích và giới hạn JOIN catalog theo database/bảng/constraint. Index MySQL được khai báo trước FK để tránh trùng index hỗ trợ. Đã thêm unit test và hai test provider thật có ignore; chưa chạy gate nào, cha cần chạy lại review và smoke cross-database.

