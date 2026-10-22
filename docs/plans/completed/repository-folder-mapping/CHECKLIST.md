# Repository Folder Mapping Checklist

- [x] Inspect all six crates and loose documentation; record exact mapping before moving files.
- [x] Relocate 40 misplaced unit-test/cohesive source files without changing logical module/public paths.
- [x] Consolidate root project planning and task records under docs/plans/project, preserving all acceptance states.
- [x] Co-locate QA/code audit reports, design references/mockups, architecture investigations and historical frontend audits.
- [x] Co-locate loose query-workspace screenshots with their existing owning plan.
- [x] Update relative links, incoming references, Rust path attributes/includes and all 19 Stitch inventory entries.
- [x] Verify 93 destinations/no old copies/no collisions; verify 27 PNG/HTML assets unchanged by SHA-256.
- [x] Run cargo +nightly fmt --all -- --check: PASS.
- [x] Run cargo check --workspace: PASS.
- [x] Run cargo clippy --workspace --all-targets -- -D warnings: PASS.
- [x] Run cargo test --workspace: 2,262 passed / 0 failed / 65 ignored.
- [x] Run bash scripts/check-ui-architecture.sh: PASS; clean-code diff ratchet: 0 failures.
- [x] Run cargo build --release --locked -p db-pro-native: PASS.
- [x] Smoke actual native release: temporary SQLite fixture connected/schema loaded; inspect Light/Dark screenshots.
- [x] Check eframe persistence feature/storage safety before removing temporary runtime data.
- [x] Independent source and documentation reviews: ACCEPT, no introduced P0/P1/P2.
- [x] Record evidence, limitations, exact mapping and Vietnamese handoff; update STATUS.md and root navigation.
