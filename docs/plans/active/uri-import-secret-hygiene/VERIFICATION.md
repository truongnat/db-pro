# URI Import Secret Hygiene — Verification

## Automated checks

- `cargo test -p db-pro-app mapper_` — PASS, 3 passed / 0 failed (snippet parsing plus successful-import clear and invalid-input preservation).
- `cargo +nightly fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo build --release --locked -p db-pro-native` — PASS.
- `cargo build --release --locked -p db-pro-native --features capture` — PASS; emitted two existing dead-code warnings in `crates/native-app/src/capture.rs` for capture audit fields/constants.

## Runtime visual evidence

Six native New Connection dialog captures were produced with `DB_PRO_CAPTURE_NEW_CONNECTION=1`, `DB_PRO_WINDOW_SIZE`, and `DB_PRO_CAPTURE_TO`; light captures also set `DB_PRO_CAPTURE_CONNECTION_LIGHT=1`. Each process exited 0. Dialog screenshots were reviewed at 1280×800 in dark and light themes and 1920×1080 dark; the modal remains centered, readable, and free of clipping/overlap. The other sizes/themes were checked for successful image output and exact requested dimensions.

- `evidence/connection-dark-1280x800.png` — 1280×800
- `evidence/connection-light-1280x800.png` — 1280×800
- `evidence/connection-dark-1440x900.png` — 1440×900
- `evidence/connection-light-1440x900.png` — 1440×900
- `evidence/connection-dark-1920x1080.png` — 1920×1080
- `evidence/connection-light-1920x1080.png` — 1920×1080

The runtime capture is the unchanged, normal connection-dialog state; import success/error semantics are covered by app tests. No URI/password is injected into the screenshots. No database mutation was performed by the capture runs.

## Not run

- Full `cargo test --workspace` — not run; focused app tests plus the native release build cover the changed code and required UI build gate.
- Live provider round-trip — not applicable; only the UI draft's post-import plaintext URI lifetime changes.
