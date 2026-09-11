# crates/tauri-app — LEGACY (transitional host)

**Status: legacy / not the shipped application.**

This crate is the original Tauri 2 host that served the React/Vite frontend through
a system WebView. The React frontend was **archived on 2026-09-11** under
`_archive/frontend/`, and the product UI is now native Rust (`eframe` + `egui`).

## What is still active here

- The crate remains a workspace member so `cargo check/clippy/test --workspace`
  keeps compiling it and nothing silently rots.
- `tauri.conf.json` still drives the legacy bundler path, with its `frontendDist`
  and `before*Command` values pointing at the archived frontend.

## What is NOT active

- The Tauri WebView is **not** part of the production runtime.
- CI no longer builds the frontend or runs the Tauri bundler
  (see `.github/workflows/ci.yml` and `.github/workflows/release.yml`).
- `pnpm tauri dev` / `pnpm tauri build` are not part of any documented workflow.

## Direction

`docs/10-egui-native-migration-plan.md` selects `eframe`/`egui` as the target
presentation layer and recommends removing Tauri from the production runtime. This
crate is scheduled for deletion at cutover (Phase 9 of that plan) once the native
app has full feature parity.

Do not add features to this crate. New UI work belongs in `crates/ui` and
`crates/native-app`.

See also `_archive/README.md` for how to restore and run the archived frontend for
parity comparison.
