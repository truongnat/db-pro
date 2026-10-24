---
schema_version: 1
id: "MEM-SRC-1556d92430457fc6"
title: "Source mirror: docs/architecture/plugin-sdk.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/architecture/plugin-sdk.md"
source_sha256: "1556d92430457fc684d263e15c1b15b4acda3785ca54e03fde5160ffa0cebc71"
mirror_body_sha256: "1556d92430457fc684d263e15c1b15b4acda3785ca54e03fde5160ffa0cebc71"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plugin / Extension SDK (#256)

## Intent

DB Pro grows provider and tool surfaces through an **explicit, capability-gated
extension registry**. This is not an unrestricted plugin marketplace.

Third-party native code loading (`.dylib` / `.so` / `.dll`) remains **deferred**
until a sandbox and trust/signing policy exist.

## Host contract

| Item | Value |
|------|--------|
| Host API version | `1.0` (`HOST_API_VERSION`) |
| Manifest | `ExtensionManifest` (serde JSON-compatible) |
| Loader | `ExtensionRegistry` in `db-pro-core` |
| Provider reuse | Commands may declare `required_capability` from the #234 provider contract |

## Permissions

Extensions must declare only what they use:

- `register_command`
- `register_agent_tool`
- `contribute_sidebar`
- `read_schema`

Missing permissions fail the load of that extension only.

## Failure isolation

`ExtensionRegistry::load` / `load_all`:

- incompatible `min_host_api` → `Failed` report, host continues
- duplicate id → `Failed`, host continues
- other extensions still load

## Example

`example_diagnostics_extension()` registers:

- command `example.open_diagnostics_hint`
- sidebar item under Monitor

Destructive Agent tools contributed later must still pass through canonical
confirmation policies (same as built-in Agent tools).

## Out of scope (deferred)

- Arbitrary native module loading
- Unsigned network/filesystem grants
- Hot-reload of untrusted packages
