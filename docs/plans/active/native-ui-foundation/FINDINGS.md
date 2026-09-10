# Findings

- The repository environment used for this change does not expose `rustc` or `cargo`, so compilation and runtime verification remain pending.
- The existing `crates/tauri-app/src/lib.rs` owns service wiring and Tauri command registration. The next implementation slice should extract this wiring before adding egui database calls.
- The current base intentionally uses mock connection/query state so its visual direction can be reviewed independently from backend migration.
- `crates/runtime` now owns a Tauri/egui-independent service graph for connection, query and schema services. The existing Tauri bootstrap remains unchanged until all service types and command state adapters are migrated.
- The native binary now boots the runtime, forwards typed UI commands through a tokio worker, loads persisted connection summaries into the explorer, and supports selecting/connecting a saved connection. Query execution is sent only with a real persisted connection ID; result rendering is the next slice.
- `egui` 0.29 / `eframe` 0.29 were selected to stay compatible with the workspace's current Rust 1.77 policy; dependency compatibility must be confirmed by CI.
