---
schema_version: 1
id: "MEM-SRC-cfdf9ea55b7c7286"
title: "Source mirror: docs/plans/active/ui-component-language/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-component-language/VERIFICATION.md"
source_sha256: "cfdf9ea55b7c7286e47e2ede382d5aa447eeb86e6d0f8f64bae65f7f17e8dd4b"
mirror_body_sha256: "cfdf9ea55b7c7286e47e2ede382d5aa447eeb86e6d0f8f64bae65f7f17e8dd4b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification

## Baseline

- Baseline SHA: `8da3c108ce237811eef6931d6c081cef9bf85447`
- Branch: `fix/ui-component-language`

## Commands executed

- `cargo fmt --all -- --check` — PASS, exit 0.
- `cargo check --workspace` — PASS, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS, exit 0.
- `cargo test -p db-pro-ui primary_native_surfaces_do_not_reintroduce_raw_buttons` — 1 passed / 0 failed / 482 filtered, exit 0.
- `cargo test --workspace --no-fail-fast 2>&1 | rg '^test result:'` — 1147 passed / 0 failed / 38 ignored across test binaries, exit 0.
- `cargo build --release --locked -p db-pro-native` — PASS, exit 0.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — 13 pass / 2 warn / 1 fail; inherited long Schema Workbench methods are reported, not introduced by the button slice.

Implementation commits: `ae25aeff6c3e6a9f2edd5d8ac877d55c5db738a0`, `78eb8c34506f82c0e8e9221f6a028771e15c2fd7`.

## Runtime evidence

The native release binary launched and completed local schema introspection
against a 977-table fixture. Screenshot/accessibility capture was attempted but
is blocked: Orca reports no screenshot support and cannot identify the native
window. Therefore the required native screenshots for 1280×800, 1440×900 and
1920×1080, both themes and required states remain pending under #287. No UI
visual pass is claimed.

## Provider matrix

Not applicable; this slice changes only native control composition and does not
change database/provider behavior.
