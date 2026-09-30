# Components

Shared native egui components. Component folders expose their public API from `mod.rs` and separate rendering, interaction decisions, and local defaults.

## Shared common layer

`common/` contains reusable support that is not a standalone widget:

- `common/layout.rs` owns dialog, sheet, and popup geometry calculations.
- `common/format.rs` owns display formatting and Unicode-safe truncation.
- `common/mod.rs` is the named entry point for the shared support API.

`common_utils` remains a compatibility facade for existing callers. New code should import from
`components::common` or the explicit root re-exports rather than adding to a catch-all module.
