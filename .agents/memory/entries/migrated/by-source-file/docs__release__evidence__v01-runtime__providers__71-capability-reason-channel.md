---
schema_version: 1
id: "MEM-SRC-bf753fada5f804e2"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/71-capability-reason-channel.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/71-capability-reason-channel.md"
source_sha256: "bf753fada5f804e2027f18562920679b088be8979c260f00cd8076e8db2c070a"
mirror_body_sha256: "bf753fada5f804e2027f18562920679b088be8979c260f00cd8076e8db2c070a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Capability-level reason channel (#234 criterion 3)

## Claim

A `false` capability flag carries a named reason via `CapabilityFeature` +
`DatabaseCapabilities::limitation`, surfaced in the UI through
`CapabilityLookup::feature_limitation`.

## Landed

- `CapabilityFeature` enum covering gated query/schema/data/feature flags
- `supports` / `limitation` on `DatabaseCapabilities` (driver-specific copy)
- UI: Explain + cancel Stop path use `feature_limitation`
- MySQL `data_diff` flipped to `true` now that a dialect arm exists
- Architecture doc `provider-contract.md` updated

## Tests

```bash
cargo test -p db-pro-core limitation_explains --lib
cargo test -p db-pro-core mysql_capabilities --lib
```

## Remainder on #234

Criterion 2 (no provider-name branching where capability dispatch suffices; factory-driven
lookup instead of driver-label string map) is still partial.
