# Verification — Legacy Component Sunset

## Wave 0 (dead-fn deletion)

Commands actually run:

- `rg -n -w '<fn>' -g '*.rs' crates` per candidate → 0 call/import-shape hits for
  all 11 (also zero for `UiLevel` outside legacy.rs).
- `cargo check -p db-pro-ui` → PASS (4.16s, no warnings).
- `cargo check --workspace` → PASS (8.64s): core, infrastructure, ui, app,
  runtime, native-app all compile — proof no external callers exist.

Not yet run: fmt --check workspace, clippy, tests, captures (scheduled Wave 6).
