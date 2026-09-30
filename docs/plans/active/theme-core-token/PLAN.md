# Core theme token foundation

## Goal

Turn the native UI token foundation into a clear, maintainable core token
contract with three layers — `Primitive → Semantic → Component` — while keeping
the existing `DbProTheme` field API as a compatibility facade and preserving
every currently rendered value.

## Architecture

| Layer | File | Owns | Consumers |
|---|---|---|---|
| Primitive | `crates/ui/src/tokens/primitive.rs` | light/dark raw palettes (private), spacing/radius/type/stroke/elevation/motion scales | re-exported scales via `crate::tokens`; palettes reachable only by the semantic layer |
| Semantic | `crates/ui/src/tokens/semantic.rs` | purpose-based roles with explicit light+dark treatments, status wash recipe | `DbProTheme` (flattened facade), components via `theme.semantic` |
| Component | `crates/ui/src/tokens/component/{button,input,dialog,table,feedback}.rs` | component-owned sizes, state precedence, role-consumption contracts | the matching components; sizes re-pointed at their contract |
| Adapter | `crates/ui/src/theme.rs` | `DbProTheme` facade + `apply()` mapping to `egui::Visuals`/`egui::Style`; `EGUI_*` constants are private here | all existing callers |

## Scope (this slice)

- Extract raw light/dark palettes out of the `DbProTheme` constructors into the
  primitive layer; rebuild both constructors from semantic roles.
- Define every prescribed semantic role (background, foreground, border,
  accent, status with subtle/fill/border/foreground, syntax) for both themes.
- Define component token contracts for Button, Input, Dialog/Overlay,
  Table/Grid, Feedback/Status: owned values, supported states, explicit
  precedence `disabled → loading → active → focus → hover → default`.
- Privatize renderer adapter constants (`EGUI_*`, `RADIUS_EGUI_*`) inside
  `theme.rs`.
- Fix two measured state-precedence defects: disabled+loading button rendered
  the spinner; disabled input could wear focus/error borders.
- Route alert status frames through `theme.semantic.status.*`.

## Non-goals

- DTCG/JSON generation tooling (one active Rust UI; typed Rust tokens are the
  runtime source).
- Palette or density changes — every rendered value is preserved.
- Migrating every component-local config file; genuinely local geometry stays
  in its component, classified by its config header or the contract doc.
- Editor helper derivations (`editor_*_fill`, line numbers) and badge inline
  opacity multipliers — recorded in FINDINGS as deferred.

## Acceptance criteria

- `DbProTheme::light()` and `DbProTheme::dark()` are built from explicit
  primitive palettes via `SemanticTokens`.
- `DbProTheme::apply()` contains no unowned radius, spacing, stroke, or shadow
  literals; `EGUI_*`/`RADIUS_EGUI_*` are private to the adapter.
- Every semantic role has both light and dark treatment (unit test).
- Button sizing has one source of truth (contract constants; config re-points).
- Existing semantic field access remains valid; aliases documented with one
  authoritative role.
- State precedence is explicit in code and pinned by tests.
- Formatting, targeted compilation, clean-code scan, and `git diff --check`
  pass; runtime capture recorded honestly.
