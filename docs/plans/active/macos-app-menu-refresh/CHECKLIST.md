# Checklist

- [x] Confirm existing toolbar/palette refresh and connection-scoped cache invalidation APIs.
- [x] Define menu contents and the active-connection behavior, including New Connection, New Query, and Help.
- [x] Add a typed cache-invalidation option to schema introspection.
- [x] Build and inspect the native menu implementation with `cargo check -p db-pro-native` and `cargo check -p db-pro-native --features capture`.
- [x] Add regression coverage for Refresh Cache dispatch and no-active-connection behavior.
- [x] Run formatting, targeted UI tests, and `cargo build --release --locked -p db-pro-native`.
- [ ] Run the native translator test; compilation was blocked by `No space left on device`.
- [ ] Observe menu actions in the live macOS app and record evidence.
- [ ] Review final diff and exact SHA.
