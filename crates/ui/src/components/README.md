# Components

Shared native egui components. Component folders expose their public API from `mod.rs` and separate rendering, interaction decisions, and local defaults.

## Component contract

Use this shape when adding or changing a public component. `mod.rs` is the Rust entry point. Start with `ui.rs` and `handler.rs`; use `ui/` or `handlers/` when the component has distinct UI parts or decision groups. Keep the same ownership whichever file shape is used.

```text
<component>/
  mod.rs       public exports
  ui.rs        egui layout and painting, or ui/ with private submodules
  handler.rs   events, decisions and calculations, or handlers/ with private submodules
  config.rs    component-owned defaults only
  DESIGN.md    UI structure and rationale; behavior, event flow and performance reasoning
  README.md    short human-facing introduction and example
  API.md       full usage contract for developers and coding agents
```

Import shared theme values and tokens directly where they are used. `config.rs` does not import from `DbProTheme`, `tokens/`, or another core module, and does not mirror core constants under local names. The [Button component](button/README.md) is the first implementation of this contract; its [design](button/DESIGN.md) and [API](button/API.md) show the intended document boundaries.

Comment non-obvious code flow in English at the boundary between layers: caller inputs and egui events → handler decisions and state changes → values returned to UI → rendering or caller action. Explain why the sequence or precedence matters. Keep the comments beside the relevant code and update them when the flow changes; avoid comments that only repeat a statement's name.

## Quality requirements

- **Consistency:** components use the same semantic theme roles, core sizing/spacing tokens, and state precedence in light and dark modes. Review every supported variant and state together.
- **Accessibility:** each interactive control has a meaningful name, role, enabled/busy state, keyboard path, and visible focus. Check target size, text and focus contrast, reduced-motion behavior, and the native accessibility tree where supported.
- **Approachable UI/UX:** actions communicate their purpose and outcome; loading, disabled, error, empty, and narrow layouts remain understandable without relying on color or motion alone.
- **Dynamic customization:** theme and core tokens are the source of shared visual changes; component variants and local configuration expose only real component choices. Check that switching theme updates the rendered control without duplicated values.
- **Code quality:** follow the repository's Rust formatting and quality gates, keep UI painting apart from handler decisions, and document non-obvious flow beside the code. Record each gate as passed, failed, or not run based on actual execution.

## Shared common layer

`common/` contains reusable support that is not a standalone widget:

- `common/layout.rs` owns dialog, sheet, and popup geometry calculations.
- `common/format.rs` owns display formatting and Unicode-safe truncation.
- `common/mod.rs` is the named entry point for the shared support API.

`common_utils` remains a compatibility facade for existing callers. New code should import from
`components::common` or the explicit root re-exports rather than adding to a catch-all module.
