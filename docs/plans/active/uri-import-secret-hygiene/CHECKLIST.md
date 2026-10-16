# URI Import Secret Hygiene — CHECKLIST

- [x] Confirm #479 implementation and callers; correct unsupported persistence/backend claims.
- [x] Keep successful import data in dedicated fields and clear raw URI from draft.
- [x] Preserve invalid URI text on parse failure.
- [x] Add success/failure regression tests.
- [x] `cargo test -p db-pro-app mapper_` — 3 passed.
- [x] `cargo +nightly fmt --all -- --check` and `git diff --check`.
- [x] `cargo build --release --locked -p db-pro-native`.
- [x] Native screenshots 1280×800 / 1440×900 / 1920×1080, dark and light; inspected representative renders and verified all six dimensions.
- [x] Complete `VERIFICATION.md` and `AGENT_EVIDENCE.md` with actual results.
- [ ] Fetch/rebase on `origin/main`, inspect diff, push and open PR referencing #479.
