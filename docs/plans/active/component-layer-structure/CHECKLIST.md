# Component UI Layer Structure — Checklist

## Architecture
- [x] Confirm `mod.rs` is the component's public entry/exporter.
- [x] Define UI as egui presentation/layout and handler as behavior/decisions/calculations.
- [x] Define local-config vs shared-token boundary.
- [x] Require a usage `README.md` in every component folder.

## Migration inventory
- [x] Select: `mod.rs`, `ui.rs`, `handler.rs`, `config.rs`, `README.md`.
- [ ] Accordion, AgentComposer, AgentPrimitives, Alert, AspectRatio, Badge, Button, Calendar.
- [ ] Card, Chrome, Code, Collapsible, Command, Database, DevTools, Dialog.
- [ ] Diff, Explain, Feedback, Form, HoverCard, Input, Logs, Navigation.
- [ ] Overlay, RadioGroup, ResponsiveLayout, ScrollArea, Selection, Separator, SqlEditor.
- [ ] Table, Tabs, Toggle, Transaction, Tree, Workspace.
- [ ] Audit `components/mod.rs` re-exports and classify shared infrastructure (`animation`, `interact`, `common_utils`, `legacy`).

## Reusable implementation guide
- [x] Document the repeatable component authoring workflow, responsibilities, review checklist and Select example in `crates/ui/src/components/README.md`.
- [x] Cross-link the durable guide from `PLAN.md`; keep the plan as the migration-specific contract.
- [ ] Apply the guide to the remaining public components during their migration batches.

## Select verification
- [x] Preserve Select API/re-exports.
- [x] Separate UI rendering from handler decisions and calculations.
- [x] Add handler tests.
- [x] Add/verify Select README usage example.

## Quality gates before completion
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo build --release --locked -p db-pro-native`
- [ ] Review each batch diff and confirm no behavior/API regression.

## Tổng kết bằng tiếng Việt
Đây là kế hoạch migration toàn bộ component công khai. Select là batch đầu; checklist từng component sẽ được đánh dấu theo thay đổi thực tế, kèm README hướng dẫn dùng.
