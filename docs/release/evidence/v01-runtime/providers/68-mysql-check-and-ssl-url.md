# MySQL CHECK introspection + SSL URL mapping (#235 remainder)

## Claim

MySQL 8 CHECK constraints are read from `information_schema`, and the stored `SslMode` is
honoured on the sqlx connection URL instead of always negotiating `PREFERRED`.

## Changes

| Area | Detail |
|---|---|
| CHECK | `mysql/introspect.rs` joins `TABLE_CONSTRAINTS` + `CHECK_CONSTRAINTS` |
| TLS | `mysql_connection_url` appends `?ssl-mode=DISABLED\|REQUIRED\|VERIFY_CA\|VERIFY_IDENTITY` |
| Tests | `mysql_connection_url_maps_every_ssl_mode`; live `mysql_introspects_named_check_constraints` |

## Commands

```bash
cargo test -p db-pro-infrastructure mysql_connection_url_maps_every_ssl_mode --lib
# with MySQL DATABASE_URL:
cargo test -p db-pro-infrastructure --test mysql_integration \
  mysql_introspects_named_check_constraints -- --ignored
```

## Residual on #235

Table-data change-set E2E still pending; issue stays open.
