# Settings Shell Visual Polish

## State

`RUNTIME_VERIFY`

## Goal

Make the native Settings surface read as a dedicated settings workspace like
the supplied light/dark references: a quiet background, a distinct navigation
rail, grouped settings navigation, and an obvious active pill.

## Scope

- Native Settings shell composition in `crates/ui`.
- Light/dark theme mapping through existing `DbProTheme` semantic tokens.
- Settings navigation grouping and active/hover painting.
- Deterministic native capture coverage for the affected surface.

## Non-goals

- No new settings, persistence fields, database commands, or provider logic.
- No redesign of the normal database workspace shell outside Settings mode.
- No React/Tauri frontend changes.

## Acceptance

1. Settings opens as a full central surface while the normal workspace chrome is
   suppressed for that mode.
2. Existing section selection and settings actions still dispatch unchanged.
3. Light and dark surfaces use `DbProTheme` tokens without new raw colors.
4. Native captures exist for 1280×800 in both themes; standard Rust gates and
   the locked native release build are executed.

## Provider impact

Presentation-only. PostgreSQL and SQLite behavior is unchanged and therefore
not applicable.
