# Verification

## Baseline

- Baseline SHA: `a81757aadcb97062c74259b83b304ede051ca5ed`
- Implementation SHA: `11ff3534ab059748e449a0ab8791f4b60e95ee72`
- Branch: `fix/auren-license-policy`

## Commands

- `cargo metadata --format-version 1 --locked` — PASS, exit 0; resolved package
  license expressions were inspected, including local packages and bundled
  font-related metadata.
- `cargo fmt --all -- --check` — PASS, exit 0.
- `cargo check --workspace` — PASS, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS, exit 0.
- `cargo test --workspace` — PASS, exit 0; 1133 passed / 0 failed / 38 ignored across the workspace.
- `cargo build --release --locked -p db-pro-native` — PASS, exit 0.
- `cargo metadata --format-version 1 --locked` — PASS, exit 0; all six local packages report `MIT`.
- `git diff --check` — PASS, exit 0.

## Runtime evidence

Not applicable. This change does not alter database behavior or the native UI.

## Remaining limitation

No final platform-specific third-party notice bundle has been generated. This
must be completed before publishing binary artifacts publicly. The existing
candidate archives predate this policy and were not rebuilt in this change.
