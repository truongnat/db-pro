# Explorer First-Tab Visual Redesign

**State:** RUNTIME_VERIFY  
**Branch:** `feature/explorer-first-tab-redesign`  
**Design baseline:** `docs/design/sidebar-explorer/sidebar-explorer-baseline.md` and `docs/design/sidebar-explorer/sidebar-explorer-feature-research.md`  
**Source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`

## Goal

Make the first sidebar tab (Explorer / Database Navigator) feel like a deliberate, premium native database IDE surface: clear hierarchy, compact but readable tree, purposeful spacing, and consistent interaction states. Preserve existing behavior and use `DbProTheme` semantic tokens.

## Scope

- Explorer toolbar/search/refresh composition.
- Connection, database, schema, folder, and object-row visual hierarchy.
- Empty, loading, error, connected, connecting, disconnected, selected, hovered, and focused states where already represented.
- Native runtime review at 1280×800, 1440×900, and 1920×1080.

## Non-goals

- No new Explorer features from F1–F11 in the research report.
- No provider/runtime/database behavior changes.
- No activity rail, Files, Query, or other tab redesign in this slice.
- No new color token unless the current semantic system cannot express a demonstrated need.

## Design decisions

- Treat the research report as a capability inventory and risk map, not a visual specification or implementation commitment.
- Follow the native migration brief's calm density, layered hierarchy, compact typography, progressive disclosure, keyboard discoverability, and data-first principles.
- Prefer existing native components and spacing conventions; never hard-code colors where theme tokens exist.
- Improve grouping and information hierarchy without adding decorative cards or redundant borders.

## Acceptance criteria

1. Connection and schema hierarchy is scannable within two seconds; connection status, active selection, and expandable structure are distinguishable without relying on color alone.
2. Search and refresh form one compact, aligned toolbar; narrow sidebar widths do not overlap, clip important controls, or create horizontal scrolling.
3. Rows align icon, label, badges, and disclosure affordance; long names remain safely clipped with discoverable full text.
4. Empty/loading/error and connected/disconnected states remain understandable and provide recovery/action where existing behavior allows.
5. Existing Explorer actions, reducers, guards, keyboard shortcuts, and provider capability behavior remain unchanged.
6. Screenshots are captured and visually inspected at all three required resolutions; runtime evidence is recorded honestly.
7. `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the release native build are run and results recorded.

## Provider scope

Pure presentation change; PostgreSQL and SQLite behavior is intentionally unchanged. Any provider-specific state displayed must continue to come from existing capability/state inputs.
