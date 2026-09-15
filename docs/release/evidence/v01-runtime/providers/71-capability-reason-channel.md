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
