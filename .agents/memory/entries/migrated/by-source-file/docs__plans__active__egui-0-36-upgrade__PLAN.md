---
schema_version: 1
id: "MEM-SRC-17281ce5fd2e791b"
title: "Source mirror: docs/plans/active/egui-0-36-upgrade/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/egui-0-36-upgrade/PLAN.md"
source_sha256: "17281ce5fd2e791b684c64fdac6f286f9a80228dc0c9329b1d4b374f3ef8709e"
mirror_body_sha256: "17281ce5fd2e791b684c64fdac6f286f9a80228dc0c9329b1d4b374f3ef8709e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
