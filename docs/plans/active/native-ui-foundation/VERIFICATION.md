# Verification

## Automated

Not run: `rustc` and `cargo` are unavailable in the current environment.

Commands to run when the toolchain is available:

```bash
cargo fmt --all -- --check
cargo check -p db-pro-ui -p db-pro-native
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Manual

Pending native runtime review:

- Launch native binary.
- Check dark theme, panel hierarchy, typography and focus/hover states.
- Check sidebar resize/collapse and agent panel open/close.
- Check `Cmd/Ctrl+P`, `Cmd/Ctrl+B`, and Escape.
- Review at 1280×800, 1440×900 and 1920×1080.
