# Layout Aesthetics Audit — Verification

**Status: not started.** No capture, measurement or score exists yet; this file records what will be
run and stays empty of results until they are real.

## Planned commands

```bash
cargo build -p db-pro-native --features capture          # capture harness
env DB_PRO_DATA_DIR=<temp> DB_PRO_WINDOW_SIZE=<WxH> \
    DB_PRO_CAPTURE_TO=<name>.png <surface env vars> ./target/debug/db-pro-native
cargo +nightly fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --locked -p db-pro-native
```

The audit changes no product code, so the Rust gates are only re-run if a finding is implemented —
and then they belong to the owning plan, not to this one.

## Evidence rules for this audit

- Every scored screen cites an image path plus the measured numbers behind the score.
- Requested vs rendered viewport is recorded per cell; 1440/1920 render at 838 logical height on this
  host and must not be described as full-height verification.
- `exit=-6` cells are the macOS TouchBar teardown crash after the PNG is written; they are recorded
  as crashed, not as clean passes.
- A missing cell is reported as missing; it is never inferred from a neighbouring surface.
- Scores are expert judgement with anchors, not user testing.

## Results

None yet.
