# Findings: `keyring` Dependency Build in Linux Container Environments

## Problem Statement
In Linux container environments where system `libdbus-1-dev` and `dbus-1.pc` are not installed, `libdbus-sys v0.2.7` (a dependency of `keyring`) fails to build during `cargo check --workspace` or `cargo test --workspace` with `pkg_config failed: Package dbus-1 was not found`.

## Severity
P1 — Build failure on Linux container environments.

## Root Cause
`keyring` was configured with `apple-native`, `linux-native-sync-persistent`, and `windows-native`, but omitted `vendored`. `linux-native-sync-persistent` uses `libdbus-sys`, which defaults to searching system `pkg-config` unless `vendored` is enabled.

## Resolution
Include `"vendored"` in the `keyring` features list in root `Cargo.toml`.
