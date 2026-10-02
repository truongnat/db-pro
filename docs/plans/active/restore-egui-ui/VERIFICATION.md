# Verification

State: RUNTIME_VERIFY
Source SHA: `02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f`
Restored implementation SHA: `c0c1f5525b20a810913d1eee13c7ee2dd15b6664`
`git diff c0c1f5525b20a810913d1eee13c7ee2dd15b6664 02ae0bb5f1c031e1e5873ea94aa81b0f5dd4a75f -- crates Cargo.lock Cargo.toml` is empty: all shipping Rust/manifests match the pre-integration egui implementation. No `rs_ui` or `rs-ui` remains in Rust/manifests/lockfile.

## Automated gates

- `git diff --check`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `cargo check --workspace`: exit 0.
- `cargo test -p db-pro-ui`: 940 passed / 0 failed / 0 ignored.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace`: exit 0; 1597 passed / 0 failed / 41 ignored across 22 suite summaries. Ignored provider tests are not runtime proof.
- `cargo build --release --locked -p db-pro-native --features capture`: exit 0.
- `cargo build --release --locked -p db-pro-native`: exit 0 (executed by native perf scan); binary 32.0 MB.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: exit 0; 14 passes, 2 inherited heuristic warnings (3 functions with >3 arguments, 11 long functions). Restored source matches the established baseline; no new helper/module was introduced.
- `bash .skills/perf-audit/scripts/perf-scan.sh native`: exit 1; native build and 32.0 MB size checks passed, benchmark dispatch failed because `--quick` was forwarded to the libtest harness (`Unrecognized option: quick`). Inherited tooling defect; script unchanged.
- `cargo bench -p db-pro-ui --bench result_grid_benchmarks -- --quick`: exit 0; direct Criterion target completed. See evidence/grid-bench.log for measured intervals. No comparative improvement claim.

## Native runtime evidence

Capture-feature release binary using isolated temporary `DB_PRO_DATA_DIR`, `DB_PRO_CAPTURE_TO`, `DB_PRO_WINDOW_SIZE`, settle 30 frames. Normal uses Query/light helper; table uses deterministic typed-cell fixture; empty has no connections; loading shows pending connection request; error opens deterministic validation-error dialog. These are native egui framebuffers, not CPU mock previews. PostgreSQL/SQLite live database flows were not exercised; their code was unchanged.

Retina scale: 2.0. Requested logical widths were honored; requested heights 900/1080 were clamped by macOS to 838. Formal larger-height acceptance remains open, so this plan stays active. Native capture exits were 0 for 1280 and 1920 scenes; 1440 scenes wrote their PNG but did not auto-close, so the isolated capture processes were terminated after capture (first normal scene timed out at 45 s). No user app process was terminated. No live keyboard/drag/provider claim is made from these fixtures.

| Screenshot | Framebuffer pixels | Actual logical viewport |
|---|---|---|
| empty-1280x800.png | 2560×1600 | 1280×800 |
| empty-1440x900.png | 2880×1676 | 1440×838 |
| empty-1920x1080.png | 3840×1676 | 1920×838 |
| error-1280x800.png | 2560×1600 | 1280×800 |
| error-1440x900.png | 2880×1676 | 1440×838 |
| error-1920x1080.png | 3840×1676 | 1920×838 |
| loading-1280x800.png | 2560×1600 | 1280×800 |
| loading-1440x900.png | 2880×1676 | 1440×838 |
| loading-1920x1080.png | 3840×1676 | 1920×838 |
| normal-1280x800.png | 2560×1600 | 1280×800 |
| normal-1440x900.png | 2880×1676 | 1440×838 |
| normal-1920x1080.png | 3840×1676 | 1920×838 |
| table-1280x800.png | 2560×1600 | 1280×800 |
| table-1440x900.png | 2880×1676 | 1440×838 |
| table-1920x1080.png | 3840×1676 | 1920×838 |
