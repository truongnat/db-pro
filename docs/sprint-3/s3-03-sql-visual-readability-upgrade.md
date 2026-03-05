# Sprint 3 - S3-03 SQL Visual Readability Upgrade

## Goal

Improve SQL readability in default light mode with stronger token contrast and clearer completion popup hierarchy.

## Delivered

- Upgraded SQL syntax highlight palette:
  - stronger keyword/operator contrast
  - clearer function/type/literal differentiation
  - explicit invalid token styling
  - file: `src/features/sql-editor/theme.ts`
- Improved completion popup readability:
  - denser but clearer row spacing
  - stronger matched-text emphasis
  - improved detail text contrast
  - selected-state readability tuning
  - file: `src/features/sql-editor/theme.ts`

## Visual Changes Summary

- `keyword`: deeper blue + bold for scanability.
- `string`: amber tone separated from numeric literals.
- `number/bool/literal`: purple + semibold for quick distinction.
- `function`: magenta accent to separate function calls from identifiers.
- `type/class`: teal tone with semibold weight.
- `invalid`: red wavy underline for immediate error visibility.

## Validation

- `npm run -s build` => PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo test --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check` => PASS (`checked=27 errors=0 warnings=0`)

Validated on 2026-03-05.
