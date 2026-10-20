# Native Feature UI Cleanup

State: IMPLEMENTING. Work directly on main under the owner override.
Baseline HEAD: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; existing uncommitted Quick Open/theme/shell work is preserved.

## Goal and scope

Owner requests research and implementation to make native feature UI clean, attractive and compact. Audit active native feature families, then fix concrete presentation defects and shared sources of repeated chrome. Preserve every action and data-safety boundary.

- Shared panel/toolbar surfaces: quiet fill-based grouping, no nested decorative boxes; preserve input, focus, validation and data-grid boundaries.
- Welcome/Query/Agent/Settings: compact, readable hierarchy with no content hidden underneath side panels.
- Table/Grid/Schema/DDL/ER/Tools/Monitoring/Transfers/Security: audit shared frame/spacing and actual overflow; implement evidence-backed fixes, not speculative restyling.
- Reuse current DbProTheme, tokens, components, fonts and capture engine. No framework/dependency/palette changes.

## Design contract

Database developers/analysts, keyboard/mouse desktop use, native Rust egui 0.36.2. Prioritize data over chrome. Clear primary action, neutral secondary actions, visible focus/hover, stable layout, 4/8 spacing rhythm and independent Light/Dark review. Keep dense desktop controls; do not apply phone-sized targets or 16px mobile body requirements indiscriminately.

ui-ux-pro-max research: explicit UX search confirms visible keyboard focus. Initial design-system result was an unrelated documentation landing/OLED-only recommendation, rejected. Narrow style search returned Minimalism & Swiss Style, applicable for restrained enterprise/tool hierarchy and both themes. Existing project tokens remain authoritative; no suggested raw palette/font changes are adopted.

## Non-goals

No domain/provider/SQL/runtime changes, no action deletion, no archived frontend, no wholesale view rewrites, no theme palette replacement. Existing sample SQLite and Clippy failures remain outside this presentation task.

## Acceptance

- Audit records concrete findings, owners, severity and provider impact (PostgreSQL n/a; SQLite n/a).
- Shared surfaces are consistent across their native feature callers.
- Observed clipping/overflow fixed at its owner; actions remain accessible.
- Focus, hover, selected, disabled and validation states retain semantic feedback.
- Run affected package tests, final fmt/check/architecture/release gates; report existing workspace failures without suppression.
- Capture and inspect affected Light/Dark surfaces at requested 1280×800, 1440×900 and 1920×1080; record actual clamped sizes and fixture limits.
- Lifecycle remains RUNTIME_VERIFY while exact-height/native-pointer/independent-review evidence is incomplete.
