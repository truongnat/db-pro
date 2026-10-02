# Verification

Source commit: `cd58b354a43f55458759c298bc053c5a4dd36c0b`; baseline: `307c9cc0301f9d3de4f58a22c6d916789b483341`. Date: 2026-10-02.

## Commands actually run

| Command | Result |
| --- | --- |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | pass |
| `cargo check --workspace` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test --workspace` | 1601 passed, 0 failed, 41 ignored; UI: 944 passed |
| `cargo build --release --locked -p db-pro-native --features capture` | pass |
| `cargo build --release --locked -p db-pro-native` | pass |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | exit 0, 14 pass / 2 warning categories / 0 failures |

Raw test and scan output: `evidence/workspace-tests.log`, `evidence/clean-code.log`. Initial test-authoring failures (float equality/frame border expectation) and test closure clippy diagnostic were corrected before the passing final gates. Scan warning categories concern argument/function size; reviewed, no swallowed errors or new dependency. CI and independent review: not run. Performance timing/benchmark: not run; no improvement claim.

## Native evidence

Release capture feature, isolated temporary data directories, Component Gallery `form-error`, light theme:

| Requested logical size | Physical framebuffer | Exit |
| --- | --- | --- |
| 1280×800 | 2560×1600 | 0 |
| 1440×900 | 2880×1676 | PNG written; isolated process terminated after timeout (-15) |
| 1920×1080 | 3840×1676 | 0 |

Screenshots/logs: `evidence/form-error-*.png`, `evidence/form-error-*.log`. Retina scale is 2×; screen work area clamps larger viewport heights. These are evidence at requested widths, not a full-height acceptance pass.

Native GUI verification ran a temporary bundle copied from the same capture binary. `evidence/alert-selected-native.jpg` shows white selected description text on blue with dismiss at the right. Clicking Dismiss alert removed the summary while field validation errors remained. `evidence/input-selected-native.jpg` shows selected `app_prod` in the Database input, blue/white and focused border. Screenshots are original GUI pixels, including host titlebar/cursor overlay.

Button scaling is verified by egui output geometry tests; a native held-button recording is pending. Loading/empty button states and all theme/state/size combinations were not freshly captured. The feature remains RUNTIME_VERIFY until the formal visual gate is covered. The independent rs-ui repository was not changed by this fix.
