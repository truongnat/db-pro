# Verification — Explorer First-Tab Visual Redesign

## Source baseline

- Design inventory source SHA: `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`.
- Base commit: `7c889eec`; feature source changes remain uncommitted on `feature/explorer-first-tab-redesign`.

## Runtime evidence

The native `db-pro-native` capture harness was exercised with the seeded PostgreSQL connection:

| Requested viewport | Captured framebuffer | State | Artifact |
|---|---|---|---|
| 1280×800 | 2560×1600 (Retina 2×) | Connecting | `screenshots/explorer-before-1280x800.png` |
| 1280×800 | 2560×1600 (Retina 2×) | Connecting, expanded | `screenshots/explorer-after-1280x800.png` |
| 1440×900 | 2880×1676 (window height capped by display) | Connecting, expanded | `screenshots/explorer-after-1440x900.png` |
| 1920×1080 | 3840×1676 (window height capped by display) | Connecting, expanded | `screenshots/explorer-after-1920x1080.png` |
| 1280×800 | 2560×1600 (Retina 2×) | Empty, isolated release data directory | `screenshots/explorer-empty-1280x800.png` |

The pending row is now visibly expanded and renders `Connecting…` without a click; this was
confirmed in all three connecting-state captures. An empty Explorer capture was also collected
from an isolated release data directory. The PostgreSQL fixture timed out in the connection
worker, so these captures do not verify a connected schema tree or database behavior. The host
display caps the native window at 838 logical pixels high; the 1440×900 and 1920×1080 targets were
requested but not achieved. The error state has not been captured.

## Gates

| Gate | Result | Evidence |
|---|---|---|
| Native runtime baseline | PASS — connecting and empty states observed | Before/after + empty screenshots above |
| Explorer captures at requested resolutions | PARTIAL | Host caps the native window at 838 logical px; 1440×900 and 1920×1080 heights not achieved |
| `cargo fmt --all -- --check` | PASS | Executed; exit 0 |
| `cargo test -p db-pro-ui` | PASS — 678 passed | Focused UI crate suite |
| `cargo check --workspace` | PASS | Executed; exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Executed; exit 0, no warnings |
| `cargo test --workspace` | PASS — 1327 passed / 0 failed / 41 ignored | Executed; ignored provider/fixture tests are not counted as passing |
| `cargo build --release --locked -p db-pro-native` | PASS | Executed; release profile completed |
| Clean-code scan (`rust --diff --ratchet --ci`) | PASS — 16 checks | 2 changed production Rust files; 0 warnings |

## Provider matrix

| Provider | Supported behavior changed | Automated evidence | Live runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | No | N/A | N/A — visual-only | Existing capability/state unchanged |
| SQLite | No | N/A | N/A — visual-only | Existing capability/state unchanged |

