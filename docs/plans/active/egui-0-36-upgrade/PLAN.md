# PLAN — egui/eframe 0.29.1 → 0.36.2

State: IMPLEMENTING → verification complete; pending regression triage decision.

## Scope

- Bump `egui`/`eframe` to 0.36.2 in `Cargo.toml`, `crates/ui/Cargo.toml`,
  `crates/native-app/Cargo.toml`; refresh `Cargo.lock`.
- Migrate all broken call sites to the 0.36 API surface.
- Preserve UI, layout, behavior, glow backend, wayland/x11/accesskit features.
- No refactoring beyond the migration; no UI Inspector feature work beyond
  keeping the existing dev-tools inspector compiling and tested.

## Non-goals

- Rendering backend change.
- Visual redesign or layout changes.
- MSRV bump beyond what eframe 0.36 requires (workspace toolchain already new
  enough; clippy reports rust-1.96 lint names).

## Migration highlights

- `RawInput::modifiers` removed → `Event::ModifierChanged`.
- `Context::run` → `ctx.run(RawInput)`/`begin_pass`/`end_pass`.
- `PlatformOutput::copied_text` → `commands`/`OutputCommand::CopyText`.
- `egui::TopBottomPanel` → unified `egui::Panel` (`PanelSide`, `exact_size`).
- `Sense` fields (`click`, `focusable`) restructured.
- `Context::run_ui` takes `&mut Ui` closure arg.
- `WindowResizeDirection`/`accesskit`/`emath`/`epaint` version bumps in lock.
- Widget geometry: `prev_pass.widgets` only; pending picks must retry until
  the target registers on the next pass.
