# Masking NULL correctness and unmatched rules — Verification

State: REVIEW
Baseline SHA: `d5aea9568430c5391bb6150a9748332fd2e0d62e`
Branch: `fix/mask-null-text-distinction`

## Commands actually executed

```text
cargo test -p db-pro-core masking
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 448 filtered out

cargo test -p db-pro-app masking
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 554 filtered out

cargo test --workspace
all suites ok: 1957 passed; 0 failed; 42 ignored

cargo check --workspace
exit 0 (independent rerun after #503 change)

cargo +nightly fmt --all -- --check
FMT_CHECK_OK (clean after `cargo +nightly fmt --all`)

cargo build --release --locked -p db-pro-native
Finished `release` profile [optimized] target(s) in 43.62s (after both #502/#503 changes)

bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci
pass: 14   warn: 2   fail: 0   (both warnings pre-existing: mask_cell >3 params;
                                   build_preview >50 lines, already allow-listed)
```

### Clippy — SKIPPED (pre-existing baseline failure)

```text
cargo clippy --workspace --all-targets -- -D warnings
error: could not compile `db-pro-core` (lib test) due to 3 previous errors
crates/core/src/domain/saved_task.rs:153:17, :158:17, :165:17  (collapsible_match)
```

`crates/core/src/domain/saved_task.rs` is byte-identical to `main`
(`git diff main -- crates/core/src/domain/saved_task.rs` is empty), so these are
inherited baseline failures, not regressions from this change. The masking files
under test produce no clippy diagnostics. Not fixed here per "do not fix unrelated
baseline failures".

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | N/A | N/A | pure text transform; no SQL emitted |
| SQLite | N/A | N/A | pure text transform; no SQL emitted |

## UI lifecycle

```text
UI
→ command
→ backend
→ database
→ introspection
→ refreshed UI
```

Status: N/A — masking is a pure domain transform reached only from preview/demo
harness code, not a database call site. #503's unmatched-rule warning is displayed
in the masking preview message; exact case-sensitive matching is preserved to
avoid applying a rule to a different quoted PostgreSQL identifier.

## Remaining evidence gaps

- `TransferRow` cannot express SQL NULL apart from empty text (F2); transfer-path
  NULL rendering is unchanged and documented, not provider-verified.
- Masking is not wired to a live export path, so no end-to-end provider evidence
  is applicable.

## Completion decision

Do not mark COMPLETED until all applicable lifecycle gates are satisfied. This
change is a focused P1 masking-correctness fix; P0=0, P1=0 after the fix. Clippy
is skipped due to an inherited baseline failure outside the touched files.
