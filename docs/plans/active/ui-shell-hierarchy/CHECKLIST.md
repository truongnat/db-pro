# Checklist — UI03 Shell Hierarchy

- [x] Audit top bar breadcrumbs, badges, center search trigger, and action buttons.
- [x] Audit Activity Bar item spacing, active indicators, and bottom dock (Settings).
- [x] Audit Sidebar header, workspace dropdown, and "+ New query" action button.
- [x] Audit Sidebar resize divider and ensure consistent border strokes and theme tokens.
- [x] Run cargo fmt, cargo check, clippy, test workspace, and release build.
- [x] Document findings and verification.

## Wave 2 — Region Contrast & Shell Elevation Hardening (Current)

- [x] Investigate and align shell panel borders in `crates/app/src/shell/`:
  - [x] CentralPanel: switch from `ShellSurface::Panel` to `ShellSurface::App` so workspace canvas steps down from navigation chrome.
  - [x] Sidebar / CentralPanel boundary: flush split inset with borderless outer frames per quick-open contract.
- [x] CentralPanel & Welcome surface elevation:
  - [x] Elevate Welcome action cards with crisp resting border (`border_default`) instead of faint hairline `border_subtle`.
- [x] Automated tests:
  - [x] `cargo test -p db-pro-ui` (728 passed)
  - [x] `cargo test -p db-pro-app` (721 passed, including `shell_panel_boundaries_stay_borderless_in_both_themes`)
- [x] Runtime Verification:
  - [x] Capture 1280x800, 1440x900, 1920x1080 native screenshots for Light & Dark mode (`/tmp/ui_shell_evidence/`).
  - [x] Inspected visual contrast across all major regions: distinct workspace canvas vs sidebar, crisp elevated cards.
