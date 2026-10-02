# Verification

Source SHA: `52cdf982e748a0f57a1dae3b4493316268e22318`. Baseline before tab accent removal: `6b39d7c47d5744ad52b40dab731ce0336a08d171`.

## Commands actually executed

Tab removal: `git diff --check`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build --release --locked -p db-pro-native`: all exit 0. Workspace totals 1601 passed / 0 failed / 41 ignored. No test added for the four-line paint removal. The full test log was transient `/tmp/db-pro-tab-border-tests.log`.

Audit: `cargo build --release --locked -p db-pro-native --features capture` passed; native capture batch 24/24 wrote screenshots. Geometry probe: `cargo run --quiet -p db-pro-ui --example spacing_audit_probe` exited 0, output `evidence/measurements.json`. Initial probe constructor arity error corrected before successful run. Probe source saved as `evidence/measurements-probe.rs`; temporary workspace example removed. Source code unchanged by audit. Final `cargo fmt --all -- --check` and base release build passed.

To reproduce: copy probe into `crates/ui/examples/spacing_audit_probe.rs`, run the command, then remove the temporary example. It uses public shared components, native egui Context and actual output shape bounds.

## Artifacts

- `spacing-inventory.csv`: 976 matched source lines, 431 direct numeric lines, 162 files.
- `sizing-inventory.csv`: 307 sizing/allocation lines.
- `spacing-contracts.csv`: 251 declared constants.
- `inventory-summary.json`: counts and hotspot summary.
- `measurements.json`, `measurements-probe.rs`: 15 component/context measurements.
- `capture-manifest.json`: every capture's exit status, existence and physical size.
- `overview.jpg`, `overview-other.jpg`: montages reviewed; originals retained.

Native capture: isolated data directory per process, 30 settle frames. 20 initial captures at logical 1280×800 have physical 2560×1600. Buttons and Form Error at 1440×900/1920×1080 have physical 2880×1676/3840×1676 due host work area clamp. Both 1440 processes wrote PNG then were terminated after 12-second timeout (-15); others exited 0. This does not establish full-height 900/1080 acceptance.

Screenshots include normal/error/loading/empty fixture evidence across surfaces, not every state of every component. Gallery popup open states, all scroll positions, every locale and live providers were not exercised. No CI, independent review or frame-performance benchmark run. The report does not claim those gates passed.
