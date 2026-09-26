# Component authoring guide

This directory contains the native `egui` components used by DB Pro. Use the migrated [`Select`](select/README.md) as a concrete example when adding a component or migrating an existing one.

## Structure

```text
components/<name>/
  mod.rs       # declares modules and exports the stable public API
  ui.rs        # egui rendering/layout and interaction plumbing
  handler.rs   # typed behavior, decisions, and UI-independent calculations
  config.rs    # component-owned constants/defaults
  README.md    # purpose, API, behavior, and a usage example
```

Keep each layer meaningful. A small component does not need fabricated handler/config boilerplate; follow the active component-layer plan and explain any deliberate deviation. For a migration governed by an active plan, treat that plan as authoritative: record any exception there and get it reviewed rather than assuming this guide waives a plan requirement.

## Practical authoring workflow

1. **Survey before editing.** Find the component's current exports, implementation, callers and tests. Check neighboring components for established builder/API conventions, shared interaction helpers, `tokens.rs`, and `DbProTheme`. Record behavior that must remain unchanged, including keyboard/mouse actions, focus, dismissal and accessibility IDs where applicable.
2. **Define the public seam first.** Decide the smallest stable API callers need. Declare implementation modules and re-export public types/functions from the component `mod.rs`; preserve intended `components/mod.rs` exports. Prefer moving code without changing names or behavior. Update callers only when a module-path change requires it.
3. **Separate responsibilities.** Keep egui widget construction, layout, geometry acquisition and painting in `ui.rs` (private `ui/` submodules are fine). Put meaningful decisions, state transitions, validation and calculations that do not require `Ui` into typed `handler.rs` functions. UI code gathers egui inputs, calls handlers, then applies the returned outcome. Do not move painting into handlers or invent a handler solely to satisfy the file layout.
4. **Make dependencies explicit.** Pass measured values into pure calculations rather than having handlers query egui state. For example, Select's option painter passes the label's left edge, vertical center and galley height to `option_galley_pos`. Name non-obvious factors such as `VERTICAL_CENTER_FACTOR`; avoid unexplained numeric literals.
5. **Keep configuration local.** Put only component-owned defaults and constants in `config.rs`. Reuse shared spacing, typography, colors and other design values from `tokens.rs` and `DbProTheme`; do not duplicate theme tokens or shared values locally. If a required layer has no meaningful responsibility, raise and document the deviation rather than adding placeholder boilerplate.
6. **Prove the seam.** Add focused tests for handler decisions/calculations, including boundary cases and state transitions. Keep egui interaction/rendering tests with the UI layer where existing patterns support them. Add a concise component README with purpose, public API, important behavior/constraints, and a usage example.
7. **Migrate in reviewable steps.** Move one coherent component/family at a time, preserve observable behavior, then inspect exports and callers for drift. Avoid broad mechanical moves that obscure semantic changes.
8. **Verify and record.** Run `cargo fmt --all -- --check` and the smallest relevant `db-pro-ui` test/check after a batch; run broader workspace gates required by the feature acceptance criteria. Inspect `git diff` and `git diff --check`. Record exact commands/results in the active feature's `VERIFICATION.md`; distinguish source inspection from automated and runtime evidence, and never record an unrun gate as passed.

## Select example

```rust
Select::new("connection.pool", &mut selected_index, &options)
    .theme(theme)
    .label("Cluster pool")
    .has_more(has_more)
    .load_more(&mut request_more)
    .show(ui);
```

The Select layers illustrate the boundary: `ui/option.rs` gathers egui geometry and paints the galley; `handler.rs` calculates positions and selection behavior; `config.rs` owns Select-specific sizing. See [`select/README.md`](select/README.md) for the complete component API.

## Per-component review checklist

Before considering a component batch ready, confirm:

- [ ] `mod.rs` is the component entry point and expected public re-exports still resolve.
- [ ] UI owns egui rendering/layout; handler owns meaningful typed decisions/calculations without `Ui` coupling.
- [ ] Config contains only component-specific defaults; shared visual values come from canonical tokens/theme.
- [ ] Tests cover extracted handler behavior and important boundary cases.
- [ ] Component README describes API/behavior and includes a usage example.
- [ ] Existing callers and observable behavior are preserved; the diff contains no unrelated redesign.
- [ ] Relevant format/test/check commands and unavailable gates are accurately recorded in the active plan.

## Plan

This guide supports [`component-layer-structure`](../../../docs/plans/active/component-layer-structure/PLAN.md), whose scope is the complete public component library. Select is the first migrated batch, not the stopping point.

## Tổng kết bằng tiếng Việt

Dùng Select làm mẫu khi tạo hoặc chuyển component: giữ API ở `mod.rs`, để egui trình bày trong `ui.rs`, tách quyết định/phép tính thuần vào `handler.rs`, tái sử dụng token/theme và đặt tên cho hằng số có ý nghĩa. Thêm README/test, chạy kiểm tra phù hợp và cập nhật plan bằng bằng chứng thực tế.
