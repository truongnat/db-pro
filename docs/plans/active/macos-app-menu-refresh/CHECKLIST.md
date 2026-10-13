# Checklist

- [x] Confirm existing toolbar/palette refresh and connection-scoped cache invalidation APIs.
- [x] Define menu contents and the active-connection behavior, including New Connection, New Query, and Help.
- [x] Add a typed cache-invalidation option to schema introspection.
- [x] Build and inspect the native menu implementation with standard and capture-feature configurations.
- [x] Add regression coverage for Refresh Cache dispatch and no-active-connection behavior.
- [x] Run the native command-translator regression test.
- [x] Run formatting, targeted UI tests, and `cargo build --release --locked -p db-pro-native`.
- [ ] Observe menu actions in the live macOS app and record evidence.
- [ ] Verify Refresh and Refresh Cache against live SQLite and PostgreSQL connections independently.
- [ ] Obtain independent review and record final review outcome.
