# Verification

Baseline SHA: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main. Source is uncommitted; evidence/source.patch records cumulative checkout changes including prior Grid/footer work. Exact patch/binary/PNG hashes: evidence/manifest.json. No commit, merge, PR, push or CI run.

Passed final commands (CARGO_INCREMENTAL=0 for cargo build/check/test/clippy):
- git diff --check; cargo fmt --all -- --check.
- cargo check --workspace.
- cargo clippy --workspace --all-targets -- -D warnings.
- cargo test --release --locked -p db-pro-ui components::select: 10 passed, including selection substring matches.
- cargo test --release --locked -p db-pro-ui query_row_limit: 10 passed.
- cargo build --release --locked -p db-pro-native, plus --features capture.
- clean-code --diff --ratchet --ci: 14 passed / 2 warning groups / 0 failures. Existing function-size/file-size debt reviewed; Welcome's redundant custom action/composer helpers removed.

Twelve fresh native eframe captures: Query dropdown open and static Welcome, light/dark, at each requested 1280×800, 1440×900 and 1920×1080 size. PNGs are 2x resolution. Host caps larger logical heights to 838; exact 900/1080 heights remain pending. Visually inspected both light surfaces at 1280, dark Select at 1440 and dark Welcome at 1920. Welcome has no data-dependent/loading/error panels after removal.

Native captures establish layout and shared popup paint. Real mouse/keyboard row-limit selection and accessibility review remain pending; SQL/provider behavior is unchanged, with existing row-cap tests passing. No new dependency, provider adapter, transaction or migration. No benchmark or performance improvement claim.

Feature remains RUNTIME_VERIFY pending exact taller viewports, interactive UI and independent review. Final workspace suite: 1653 passed / 2 failed / 41 ignored, exit 101. UI: 995 passed / 2 failed. Failures are new_postgresql_connection_defaults_to_tls_require and explicit_disable_selection_is_preserved_on_submit, the same two baseline-reproduced connection-form failures. Removed three obsolete Welcome tests together with removed private behavior; no replacement mirror tests written. Known two connection-form failures reproduced on previous baseline export, see ../data-grid-record-view/evidence/baseline-tests.log.

## Select variant follow-up

Shared public SelectSize Default/Sm and SelectVariant Outline/Ghost implemented; Query uses compact Ghost. Default form behavior is preserved. Two new headless geometry regressions pass: compact variants match adjacent Sm Button height and center; default Select retains input form height. Select-filter suite: 12 passed / 0 failed.

Final fmt/check/workspace clippy and locked native release/capture builds passed. Full workspace release suite: 1655 passed / 2 failed / 41 ignored; same two baseline-reproduced connection-form failures, not newly introduced. Native current Query open/closed captures cover light/dark and three widths (12); prior Welcome captures retained with their original binary hashes. Larger exact heights and physical keyboard/mouse/accessibility acceptance remain pending. Source remains uncommitted; final cumulative patch and artifact hashes in manifest. No performance claim or new dependency.
