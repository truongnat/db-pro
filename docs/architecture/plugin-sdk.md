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
