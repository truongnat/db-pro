# Agent evidence — rs-ui Runtime Integration

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · implementation lane |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `c0c1f5525b20a810913d1eee13c7ee2dd15b6664` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Integrate rs-ui incrementally behind thin `db-pro-ui` adapters while preserving DB Pro product state and services. This continuation routes query-output split resizing and workspace tab selection through rs-ui while retaining egui hosting and DB Pro actions. |
| Out of scope | Renderer/window replacement, schema tree, result grid, search inputs, SQL editor, provider logic. |

## 2. Progress checkpoint

- Source implementation commits: `6d5f3e7d64aa039f5da454eae0b5b3fdd83632c5`, `948654524a6cd8a350a3eb972df33f2a5a1cd9ed`, and `42f14e520fa1e8bb280c1ec0399d3f523d4792da`; evidence-only commits: `f747e16f97323796dbdfeb3baa10a4947f942528` and `27de16c361469b159bd83f87fa3ff8891af4dd19`.
- Exact rs-ui working-tree tree observed during this continuation: `b66c61ac255ac4bcd1eb2c3d10e11abc2011886d` (commit baseline `db3cf2bfed3d29f0e1d19963462488ed9157f1ea`).
- Completed acceptance rows: audit plan, minimal path dependencies, shell layout adapter, sidebar geometry/resize/scroll, query-output dock resize in both orientations, tab and close-button activation through `Pressable`, retained `Tab`/`Button` semantics and cleanup, and unit coverage for adapter direction/state.
- Remaining acceptance rows: native keyboard/accessibility verification, schema tree, virtual result grid, search/filter inputs, 1920×1080 capture, full native product smoke, dependency pin/reproducibility, renderer decision.
- Findings / risks: P2 mutable local dependency; P2 native end-to-end/runtime evidence incomplete. No P0/P1 introduced by this slice.
- Tests run at Stage 2 source commit: `cargo test -p db-pro-ui --quiet` → 950 passed / 0 failed / 0 ignored; `cargo test --workspace --quiet` → 1,607 passed / 0 failed / 41 ignored; workspace check/clippy, release native build, fmt check, and diff check passed. See `VERIFICATION.md` for exact commands.
- Dependency / blocker changes: only `ui-core` and `ui-runtime` are added as local sibling path dependencies; renderer/window crates remain excluded.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `42f14e520fa1e8bb280c1ec0399d3f523d4792da` (Stage 2 implementation commit) |
| Commit list | `94865452 feat(ui): route dock resizing and tabs through rs-ui`; `42f14e52 feat(ui): make tab close controls rs-ui pressables` |
| File / surface inventory | `crates/ui/src/native_runtime_shell.rs` adds retained tab/close-button semantics, normalized activation and output splitter `Resizable`; `query_output_dock_surface_view.rs` routes bottom/right resize gestures; `workspace_tab_primitives.rs` forwards tab and close activation/focus; `workspace_tabs_surface_view.rs` provides stable tab keys and cleanup lifecycle. |
| Acceptance mapping | Splitter resize and tab/close activation → `native_runtime_shell.rs` plus query/tab adapters; selected/focused Tab and child Button semantics → `native_runtime_shell.rs` tests; retained DB Pro actions → `workspace_tabs_surface_view.rs`; automated counts and runtime limits → `VERIFICATION.md`. |
| Commands and counts | `cargo test -p db-pro-ui --quiet`: 950/0/0; `cargo test --workspace --quiet`: 1,607/0/41; workspace check/clippy and release build passed. |
| CI run IDs / status | not run |
| Known limitations | egui remains the host and painter. Native focus/accessibility smoke, schema/result/input stages and full product smoke remain pending. |
| Migrations / config implications | Cargo lock records local path packages. Clean CI/release dependency availability is not established. No persisted data or DB protocol changed. |
| Out-of-scope changes | No product/domain, query, connection, persistence, database provider, or renderer implementation changes. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — no independent reviewer has reviewed this working tree |
| Verdict | n/a — self-review only |
| P0 / P1 / P2 counts | Introduced: P0 0 / P1 0 / P2 2 known incomplete integration risks. Inherited: not independently audited. |
| Findings | No independent approval recorded. Current implementation has known P2 gaps: mutable local dependency and incomplete native interaction verification. |
| CI disposition | not run |
| Next task(s) unblocked | Begin Stage 3 schema tree; retain native Stage 2 focus/accessibility smoke as an open runtime gate. |

## 5. Research / audit handoff

- Source date: 2026-10-02.
- Source URLs / references: DB Pro implementation snapshot tree `fa0b67697906a47570aca6af73dd339032182a7b`; rs-ui tree `4e07e37c0b88d3f56ffa8c3c4d1d57e3e8f744a0`; `crates/native-app/src/main.rs`; `crates/ui/src/app.rs`; `crates/ui/src/runtime.rs`; `crates/ui/src/native_runtime_shell.rs`; rs-ui resize APIs in `crates/ui-runtime/src/interaction.rs` and scroll APIs in `crates/ui-runtime/src/scroll.rs`.
- Factual findings: DB Pro remains hosted by eframe/egui and retains its state/task bridge. The sidebar width changes through the existing workspace setter. rs-ui `register_resizable` supplies Slider semantics and preserves pointer deltas from the press origin. Sidebar wheel deltas now pass through `ScrollState`, and output offsets are synchronized from egui after layout.
- Inference: this thin behavior seam allows continued migration without coupling DB Pro domain crates to rs-ui.
- Decision / recommendation: retain egui as host while progressively moving behavior and layout per the required migration order; do not add wgpu/window crates until a separate compatibility spike.
- Unresolved questions: immutable rs-ui source for CI/releases; whether and when the eframe renderer/window host should change.
- Downstream tasks activated: Stage 2 scroll/pressable/tab behavior; Stage 3 schema tree; Stage 4 result grid; Stage 5 text inputs.

## 6. Tổng kết

Đã chuyển resize sidebar/query-output dock và activation tab/nút đóng sang rs-ui, giữ nguyên state/action DB Pro. Tại SHA `42f14e520fa1e8bb280c1ec0399d3f523d4792da`, 1.607 test workspace pass (41 ignored), workspace check/clippy và native release build pass. Stage 2 còn runtime smoke/accessibility; có thể tiếp tục Stage 3 schema tree.
