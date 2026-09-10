# Verification

## Automated

Not run: `rustc` and `cargo` are unavailable in the current environment.

Commands to run when the toolchain is available:

```bash
cargo fmt --all -- --check
cargo check -p db-pro-runtime -p db-pro-ui -p db-pro-native
cargo test -p db-pro-runtime -p db-pro-ui
cargo clippy --workspace --all-targets -- -D warnings
```

## Run native preview

```bash
DB_PRO_DATA_DIR=/tmp/db-pro-native-data cargo run -p db-pro-native
```

The app reads saved connections from the existing metadata schema. Select a connection in Explorer to connect, then open Query and run a statement. PostgreSQL/SQLite credentials remain in the configured keyring; no password is passed to the UI bridge.

## Manual

Pending native runtime review:

- Launch native binary.
- Check dark theme, panel hierarchy, typography and focus/hover states.
- Check sidebar resize/collapse and agent panel open/close.
- Check `Cmd/Ctrl+P`, `Cmd/Ctrl+B`, and Escape.
- Review at 1280×800, 1440×900 and 1920×1080.
