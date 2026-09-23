# Verification — query-workspace-zed-shell

## Quality gates

| Gate | Result | Notes |
|---|---|---|
| `cargo fmt` (touched files) | PASS | |
| `cargo check -p db-pro-ui` | PASS | |
| `cargo clippy -p db-pro-ui --all-targets -- -D warnings` | PASS | |
| `cargo test -p db-pro-ui --lib` | PASS | 523 passed |

## Runtime evidence (required)

Capture at 1280×800 and 1440×900 (dark required; light sanity). One 1920×1080 sanity.

| Shot | State | Captured |
|---|---|---|
| A | New untitled query | CAPTURED · light `A-untitled-1280x800.png`, `A-untitled-1440x900.png`; dark `A-untitled-dark-1280x800.png` |
| B | File-backed SQL | pending |
| C | Completion open | pending |
| D | Find open | pending |
| E | Running query | pending |
| F | Result dock | pending |
| G | Multiple results | pending |
| H | Error / Messages | pending |
| I | Parameters | pending |
| J | Manual transaction open | pending |

## Provider

UI composition only — no new provider paths.
