---
schema_version: 1
id: "MEM-SRC-a4d2117f72442f2f"
title: "Source mirror: docs/plans/active/issue-478-connection-draft-validation/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-478-connection-draft-validation/FINDINGS.md"
source_sha256: "a4d2117f72442f2f07ea93cd0f303a2703445ab71f3b353090dfb2736ea344ba"
mirror_body_sha256: "a4d2117f72442f2f07ea93cd0f303a2703445ab71f3b353090dfb2736ea344ba"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
