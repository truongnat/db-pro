# new-connection-secret-and-input — Verification

State: COMPLETED

Runtime evidence captured at all three required viewports (see "Runtime UI evidence" below).
The remaining interaction-only states (dialog loading spinner, validation-error toast) require
click-driving the live app and are documented as a gap, not claimed passed.

## Commands actually executed

Exact commands run on branch `fix/new-connection-secret-and-input` at SHA `34961e20`, with the
capture infrastructure added in the follow-up commit (capture.rs cherry-picked from `4faeda37`,
plus a driver-side viewport pin so no unrelated welcome-branch UI had to be pulled in).

```text
# 1. Full workspace test suite (offline) — non-capture build
cargo test --workspace --offline
test result: ok. 514 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
(no test result: FAILED blocks; one transient SQLite-integration flake re-ran clean: 32/0)

# 2. Clippy, all targets, deny warnings — non-capture build
cargo clippy --workspace --all-targets --offline -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) — exit 0

# 3. Release native build (gate)
cargo build --release --locked -p db-pro-native
Finished `release` profile [optimized] target(s) — exit 0

# 4. Clean-code ratchet scan (changed files vs main)
bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
✓ pass: 15   ⚠ warn: 1   ✗ fail: 0
  (warn = pre-existing fn-size debt in workspace_actions.rs:294, pulled into the diff by the
   nearby `pub` change; ratcheted, not introduced by this change)

# 5. Targeted regression guards (with fix present)
cargo test -p db-pro-infrastructure --lib retrieve_secret_with_disabled_keyring
  test result: ok. 2 passed; 0 failed
cargo test -p db-pro-ui --lib input_field_receives_click_when_clicked
  test result: ok. 1 passed; 0 failed
cargo test -p db-pro-ui --lib password_eye_toggles_when_clicked
  test result: ok. 1 passed; 0 failed

# 6. Revert-verification (each guard reverted, then the test re-run)
#    keyring: removed the empty-service arm -> both infra tests FAILED with
#      "keyring entry creation failed: Attribute service name is empty is invalid: disallowed"
#    text: re-added frame.interact block -> input_field_receives_click_when_clicked FAILED
#    password: re-added frame.interact block -> password_eye_toggles_when_clicked FAILED
#    All three guards fail without the fix and pass with it. Files restored after each check.

# 7. Runtime UI evidence capture (the `capture` feature, gate sizes)
cargo build --release --locked --offline -p db-pro-native --features capture
Finished `release` profile [optimized] target(s) — exit 0
mkdir -p /tmp/dbpro_shots /tmp/dbpro_cap_data
for size in 1280x800 1440x900 1920x1080; do
  DB_PRO_DATA_DIR=/tmp/dbpro_cap_data DB_PRO_WINDOW_SIZE=$size \
    DB_PRO_CAPTURE_TO=/tmp/dbpro_shots/nc_$size.png DB_PRO_CAPTURE_NEW_CONNECTION=1 \
    ./target/release/db-pro-native
done
# Each run logged: "DB_PRO_DISABLE_KEYRING: OS keyring disabled — secrets use the
# encrypted-file + session stores" (the exact P1 scenario), and the New Connection
# dialog opened with NO Configuration Error.
# PNGs (also committed under screenshots/):
#   screenshots/new-connection-1280x800.png   1280x800   470 KB
#   screenshots/new-connection-1440x900.png   1440x900   659 KB
#   screenshots/new-connection-1920x1080.png  1920x1080 1259 KB
```

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | VERIFIED (headless unit) | NOT EXERCISED (no live DB) | Secret-store path is provider-independent; covered by `retrieve_secret_*` tests |
| SQLite | VERIFIED (headless unit) | NOT EXERCISED (no live DB) | Same as above |

## UI lifecycle

```text
UI (New Connection dialog)
→ command (open_new_connection)
→ backend (ConnectionService)
→ database (n/a for the keyring/secret path; the dialog opens without contacting a DB)
→ introspection (n/a)
→ refreshed UI (dialog renders with the password field + eye toggle)
```

Status: VERIFIED at runtime — the New Connection dialog (the affected surface) renders at
1280×800 / 1440×900 / 1920×1080 with the password input + eye toggle visible, on the
disabled-keyring path that previously surfaced "Configuration Error".

## Runtime UI evidence

Three screenshots of the **New Connection** dialog (password input + eye toggle) are committed
under `screenshots/`:

| Viewport | File | Size | Shows |
|---|---|---|---|
| 1280×800 | new-connection-1280x800.png | 470 KB | compact layout; password field + eye visible |
| 1440×900 | new-connection-1440x900.png | 659 KB | full layout; password field + eye clearly shown |
| 1920×1080 | new-connection-1920x1080.png | 1259 KB | roomy layout; password field + eye clearly shown |

All three were captured with `DB_PRO_DISABLE_KEYRING` active (the P1 scenario) and the dialog
opened **without** the prior "Configuration Error". This is runtime proof that:
- the P1 keyring misclassification no longer blocks the new-connection flow, and
- the P2 input widgets (password field + eye toggle) render correctly at every gate size.

The eye-toggle *interaction* and double-click *selection* are dynamic and covered by the
headless egui regression tests (`password_eye_toggles_when_clicked`,
`input_field_receives_click_when_clicked`), which assert the real click-resolution path.

## Remaining evidence gaps

- **Dialog interaction states (loading spinner / validation-error toast) not captured.** The
  capture driver opens the dialog but does not drive clicks, so the transient "Testing
  connection…" spinner and an invalid-input error toast were not screenshot. The keyring-error
  state is *absent by the fix* and was verified gone at runtime (clean dialog open under disabled
  keyring). Live PG/SQLite connect was not exercised against a real database.
- These gaps are documented, not claimed passed.

## Completion decision

Marked COMPLETED: the affected surface is captured at all three required viewports, every code
guard is revert-verified, and all automated gates are green. The only outstanding items are the
interaction-only dialog states above, which need a manual click-driven run.
