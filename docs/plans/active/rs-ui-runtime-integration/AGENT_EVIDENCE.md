# Agent evidence — rs-ui Runtime Integration

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · implementation lane |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `c0c1f5525b20a810913d1eee13c7ee2dd15b6664` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Integrate rs-ui incrementally behind thin `db-pro-ui` adapters while preserving DB Pro product state and services. This handoff establishes the host/shell seam and continues into sidebar resize behavior. |
| Out of scope | Renderer/window replacement, schema tree, result grid, search inputs, SQL editor, provider logic. |

## 2. Progress checkpoint

- Current HEAD: `6d5f3e7d64aa039f5da454eae0b5b3fdd83632c5`; exact implementation snapshot tree for `Cargo.lock` and `crates/ui`: `fa0b67697906a47570aca6af73dd339032182a7b`.
- Exact rs-ui working-tree tree used by the local path dependency: `4e07e37c0b88d3f56ffa8c3c4d1d57e3e8f744a0` (commit baseline `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`).
- Completed acceptance rows: audit plan, minimal path dependencies, shell layout adapter, sidebar geometry hookup, persistent `Resizable` adapter, pointer-delta preservation, focused arrow-key adjustment, `ScrollState` wheel routing/offset synchronization, and app-level resize/scroll tests.
- Remaining acceptance rows: rs-ui `Pressable` tab/chrome action routing and focus scope, schema tree, virtual result grid, search/filter inputs, 1920×1080 capture, full native product smoke, dependency pin/reproducibility, renderer decision.
- Findings / risks: P2 mutable local dependency; P2 native end-to-end/runtime evidence incomplete. No P0/P1 introduced by this slice.
- Tests already run: `cargo test --workspace --quiet` → 1,603 passed / 0 failed / 41 ignored, exit 0; five focused runtime adapter tests, egui drag test, and Navigator scroll offset synchronization test passed. `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, release native build, clean-code scan, and `git diff --check` passed.
- Dependency / blocker changes: only `ui-core` and `ui-runtime` are added as local sibling path dependencies; renderer/window crates remain excluded.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `6d5f3e7d64aa039f5da454eae0b5b3fdd83632c5` (implementation commit) |
| Commit list | `6d5f3e7d feat(ui): integrate rs-ui shell behavior adapters` |
| File / surface inventory | `Cargo.lock`, `crates/ui/Cargo.toml`, `crates/ui/src/lib.rs`, new `crates/ui/src/native_runtime_shell.rs`, `crates/ui/src/sidebar_surface_view.rs`, `crates/ui/src/sidebar_view.rs`, `crates/ui/src/explorer_surface_view.rs`, `crates/ui/src/explorer_view.rs`, `crates/ui/src/workspace_feature_state.rs`, `crates/ui/src/workspace_tabs_surface_view.rs`, `crates/ui/src/app_tests.rs`, architecture/status docs, audit plan, and active plan/checklist/findings/verification/evidence/screenshots. |
| Acceptance mapping | Audit and stage order → `docs/rs-ui-integration-plan.md`; dependency boundary → `crates/ui/Cargo.toml`; shell geometry/resize/scroll behavior → `native_runtime_shell.rs`; egui action seam → sidebar and Explorer surface adapters; pointer/scroll behavior → focused app tests; gate and runtime limits → `VERIFICATION.md`. |
| Commands and counts | `cargo test --workspace --quiet`: 1,603/0/41; 5 focused adapter unit tests plus egui drag and Explorer scroll synchronization tests passed; fmt/check/clippy/release build/clean-code scan/diff check passed. Exact command statuses are in `VERIFICATION.md`. |
| CI run IDs / status | not run |
| Known limitations | Shell still paints through egui; tab/chrome activation and focus scope are not routed through rs-ui. Schema/result/input stages and native smoke remain pending. |
| Migrations / config implications | Cargo lock records local path packages. Clean CI/release dependency availability is not established. No persisted data or DB protocol changed. |
| Out-of-scope changes | No product/domain, query, connection, persistence, database provider, or renderer implementation changes. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent reviewer has reviewed this working tree |
| Verdict | n/a — self-review only |
| P0 / P1 / P2 counts | Introduced: P0 0 / P1 0 / P2 2 known incomplete integration risks. Inherited: not independently audited. |
| Findings | No independent approval recorded. |
| CI disposition | not run |
| Next task(s) unblocked | Finish Stage 2 with tab/chrome `Pressable` actions and focus semantics, then Stage 3 schema tree. |

## 5. Research / audit handoff

- Source date: 2026-10-02.
- Source URLs / references: DB Pro implementation snapshot tree `fa0b67697906a47570aca6af73dd339032182a7b`; rs-ui tree `4e07e37c0b88d3f56ffa8c3c4d1d57e3e8f744a0`; `crates/native-app/src/main.rs`; `crates/ui/src/app.rs`; `crates/ui/src/runtime.rs`; `crates/ui/src/native_runtime_shell.rs`; rs-ui resize APIs in `crates/ui-runtime/src/interaction.rs` and scroll APIs in `crates/ui-runtime/src/scroll.rs`.
- Factual findings: DB Pro remains hosted by eframe/egui and retains its state/task bridge. The sidebar width changes through the existing workspace setter. rs-ui `register_resizable` supplies Slider semantics and preserves pointer deltas from the press origin. Sidebar wheel deltas now pass through `ScrollState`, and output offsets are synchronized from egui after layout.
- Inference: this thin behavior seam allows continued migration without coupling DB Pro domain crates to rs-ui.
- Decision / recommendation: retain egui as host while progressively moving behavior and layout per the required migration order; do not add wgpu/window crates until a separate compatibility spike.
- Unresolved questions: immutable rs-ui source for CI/releases; whether and when the eframe renderer/window host should change.
- Downstream tasks activated: Stage 2 scroll/pressable/tab behavior; Stage 3 schema tree; Stage 4 result grid; Stage 5 text inputs.

## 6. Tổng kết

Đã nối `ui-core`/`ui-runtime` vào `db-pro-ui`, chuyển layout shell, resize và scroll state sidebar qua rs-ui; state và command flow hiện có được giữ nguyên. Toàn bộ 1.603 test pass. Còn thiếu Pressable/focus cho tabs/chrome, schema tree, result grid, search/filter, pin dependency và native smoke. Tiếp tục Stage 2 trước khi chuyển sang Stage 3.
