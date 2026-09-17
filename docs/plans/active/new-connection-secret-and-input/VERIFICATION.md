# new-connection-secret-and-input — Verification

State: RUNTIME_VERIFY

## Commands actually executed

Exact commands run on branch `fix/new-connection-secret-and-input` at SHA `34961e2058cae70390039806675edb8cd1b36b07` (baseline `a3a6a5290e3cf992354ee187169818a5c57122ac`).

```text
# 1. Full workspace test suite (offline)
cargo test --workspace --offline
test result: ok. 514 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
(no test result: FAILED blocks)

# 2. Clippy, all targets, deny warnings
cargo clippy --workspace --all-targets --offline -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) — exit 0

# 3. Release native build
cargo build --release --locked -p db-pro-native
Finished `release` profile [optimized] target(s) — exit 0

# 4. Clean-code ratchet scan (changed files vs main)
bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
✓ pass: 15   ⚠ warn: 1   ✗ fail: 0
  (warn = pre-existing fn-size debt in password.rs:67 / text.rs:74 show(); ratcheted, not introduced by this change)

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
#      (edit_clicked was false: frame swallowed the click)
#    password: re-added frame.interact block -> password_eye_toggles_when_clicked FAILED
#      (eye never toggled: frame swallowed the click)
#    All three guards fail without the fix and pass with it. Files restored after each check.
```

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | VERIFIED (headless unit) | NOT VERIFIED | Secret-store path is provider-independent; covered by `retrieve_secret_*` tests |
| SQLite | VERIFIED (headless unit) | NOT VERIFIED | Same as above |

## UI lifecycle

```text
UI
→ command
→ backend (ConnectionService)
→ database (n/a for the keyring/secret path; connect uses the fallback store)
→ introspection (n/a)
→ refreshed UI
```

Status: NOT VERIFIED at runtime — see gaps below.

## Remaining evidence gaps

- **UI runtime screenshots are SKIPPED.** `crates/native-app/src/capture.rs` does not exist (no in-app capture driver), and `screencapture` is blocked by TCC on this host, so screenshots/recordings at 1280×800 / 1440×900 / 1920×1080 (normal + loading/error/empty states) cannot be produced here. The behaviour is instead guarded by headless egui regression tests (`input_field_receives_click_when_clicked`, `password_eye_toggles_when_clicked`) that reproduce the click-resolution path deterministically.
- Live PostgreSQL/SQLite connectivity through the disabled-keyring path was not exercised against a real database; the secret-store fallback is validated headlessly.

## Completion decision

Do **not** mark COMPLETED until the UI runtime-evidence gate is satisfied (screenshots at the three required resolutions, or an explicit owner decision to accept headless guard coverage in lieu of runtime capture). All automated gates are green and every guard is revert-verified.
