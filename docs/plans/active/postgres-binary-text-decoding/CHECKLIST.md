# Checklist: Fix PostgreSQL Binary-Protocol Textual Value Decoding

- [x] Identify problem statement, evidence, and failure scenario.
- [x] Implement `raw.format()` matching in `decode_textual_value`.
- [x] Add unit tests for text and binary protocol decoding.
- [x] Run `cargo test -p db-pro-core` and `cargo check -p db-pro-core`.
- [x] Complete pre commit steps.
- [x] Create branch `fix/postgres-binary-text-decoding` and submit PR.
