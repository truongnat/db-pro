# #478 Findings

- `validate_connection_draft` (`crates/app/src/connection/logic.rs`) checked
  name/database, port, and SSH fields but never host/username. A Postgres
  draft with blank host therefore passed UI validation and only failed later
  in `ConnectionConfig::validate` (`crates/core/src/domain/connection.rs`),
  whose `ValidationError.field` attribution never reaches the form inputs.
- Domain requires host + username for `DriverType::Postgres | Mysql | SqlServer`
  (`validate_network_fields`); SQLite is exempt — mirrored in the fix.
- Port/database inputs already used `.required(true)`; host/username did not,
  so the form visually suggested they were optional.

## Root cause
Draft validation was written before host/username became mandatory upstream;
the network-driver branch only validated the port.
