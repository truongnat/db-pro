# Findings

- The repository environment used for this change does not expose `rustc` or `cargo`, so compilation and runtime verification remain pending.
- The existing `crates/tauri-app/src/lib.rs` owns service wiring and Tauri command registration. The next implementation slice should extract this wiring before adding egui database calls.
- The current base intentionally uses mock connection/query state so its visual direction can be reviewed independently from backend migration.
- `crates/runtime` now owns a Tauri/egui-independent service graph for connection, query and schema services. The existing Tauri bootstrap remains unchanged until all service types and command state adapters are migrated.
- The native binary now boots the runtime, forwards typed UI commands through a tokio worker, loads persisted connection summaries into the explorer, and supports selecting/connecting a saved connection.
- Query results now cross the runtime/UI boundary as typed columns and cells, with NULL/boolean/number/text/JSON/bytes rendering.
- The native grid now supports filter, click-to-sort, column divider resize, visible-row virtualization through egui `show_rows`, scoped cell/row selection and clipboard copy. Column widths persist through eframe storage and are clamped to safe bounds when restored. Pure tests cover filtered index mapping, sorting and typed cell text; large-result benchmarks remain.
- Connection list/connect/query worker paths now call `ConnectionApi`/`QueryApi` instead of reaching through service internals. `TableDataApi`, `ExportApi`, `BackupApi` and `UserApi` now expose typed native-facing operations. Progress events, cancellation for backup/restore and remaining Tauri command adapters still need migration.
- `egui` 0.29 / `eframe` 0.29 were selected to stay compatible with the workspace's current Rust 1.77 policy; dependency compatibility must be confirmed by CI.
