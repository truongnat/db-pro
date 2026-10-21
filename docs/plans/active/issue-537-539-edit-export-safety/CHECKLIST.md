# Checklist — #537 / #538 / #539

## Implementation

- [x] #537 shared `MAX_CELL_TEXT_BYTES` + `UiCell::is_truncated()` in `crates/ui`
- [x] #537 runtime mapper uses the shared constant (no duplicate cap)
- [x] #537 inspector warning banner; Export bytes hidden for truncated values
- [x] #537 inline editor entry refused in result grid and table flow
- [x] #537 write-boundary guard in `stage_data_cell_edit`
- [x] #538 driver threaded from `active_driver()` into both SQL writers
- [x] #538 dialect identifier quoting and literal escaping
- [x] #538 SQL Server boolean literals `1/0`
- [x] #538 COPY remains PostgreSQL-only
- [x] #539 overwrite token is `Option<PathBuf>` bound to `effective_export_path()`
- [x] #539 Format/path/Browse changes clear the token
- [x] #539 prompt displays the effective path
- [x] #537 localized message in en/vi/zh/ja/ko

## Tests

- [x] `runtime_query_types_tests.rs` — 4 truncation-detection tests
- [x] `translate_map_tests.rs` — 4 mapper truncation tests
- [x] `table_row_edit_tests.rs` — editor-entry and write-boundary rejection
- [x] `result_grid_export_tests.rs` — MySQL, SQL Server, PG/SQLite, COPY
- [x] `query_view_tests.rs` — overwrite does not follow a format change
- [x] `overlay_state_tests.rs` — initial token is `None`

## Gates

- [x] `cargo +nightly fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` — 2,169 passed / 0 failed / 56 ignored
- [x] `cargo build --release --locked -p db-pro-native`
- [x] `bash scripts/check-ui-architecture.sh`
- [x] `python3 tools/cc-scan.py . --fail-on error` — no new findings; 19
      inherited error-level findings remain (empty baseline), 118 warnings
- [ ] Runtime visual capture of the export dialog / inspector error state —
      not captured in this wave (needs a live connection + result set)

## Publishing

- [x] Plan directory with PLAN/CHECKLIST/FINDINGS/VERIFICATION/EVIDENCE
- [x] `docs/plans/STATUS.md` row
- [x] Commit on `main` — `26c68d26a42b671d107faf0a1805d588bb15f2c0`
- [x] L3 handoff gate answered — review run, 0 findings
- [x] Push `main` and close #537/#538/#539
