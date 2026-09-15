# MySQL parameterized change-set path (#235 criterion 5)

## Claim

MySQL table mutations use the canonical `sql_builder` + `execute_parameterized_transaction`
path (same as PostgreSQL/SQLite table-data apply), not ad-hoc SQL.

## Landed

- `MySqlConnector::execute_parameterized_transaction` — begin/bind/execute/rollback on zero
  affected rows / max-row invariant / commit
- Live `mysql_table_data_change_set_round_trips`: insert → select → update → delete via
  `CompositeConnector` + MySQL dialect + `sql_builder`

## Command

```bash
DATABASE_URL=mysql://root:…@127.0.0.1:33306/dbpro_fixture \
  cargo test -p db-pro-infrastructure --test mysql_integration \
  mysql_table_data_change_set_round_trips -- --ignored
```

## Result

`ok` against `mysql:8.4` container `db-pro-mysql-1`.
