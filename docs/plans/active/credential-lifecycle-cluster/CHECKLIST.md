# Credential Lifecycle Cluster — CHECKLIST

Baseline: `beaa1882a6253d902748c27fba53b99d9969b635`.

## Triage

- [x] Read `docs/plans/STATUS.md`, `docs/plans/FEATURE_LIFECYCLE.md`, `AGENTS.md`.
- [x] Locate and read the #479 call site (`mapper.rs:203-217`, `advanced_panels.rs:108-131`).
- [x] Locate and read the #483 call site (`pg_dump.rs:177-221`).
- [x] Verify `UiConnectionDraft` serde/persistence claim for #479.
- [x] Verify `cloud_snippet` is not logged, exported, or sent to the backend.
- [x] Decide cluster coherence and scope.

## Implementation (#483)

- [x] Construct `PgPassFile` guard before `write_all` (new `write_and_guard`).
- [x] Keep `Drop` cleanup and 0600 creation behaviour unchanged.
- [x] Add regression test forcing a partial write then failure and asserting cleanup.
- [x] Keep happy-path test (`pgpass_file_is_owner_only_and_removed_on_drop`) green.

## Deferred (#479)

- [x] Record corrected risk model and separate-change rationale in FINDINGS.md.

## Gates

- [x] Focused pg_dump module tests: `cargo test -p db-pro-infrastructure backup::pg_dump::tests` (8 passed).
- [x] `cargo +nightly fmt --all -- --check`.
- [x] Attempted `cargo clippy -p db-pro-infrastructure --all-targets -- -D warnings`; blocked by 3 existing `collapsible_match` errors in untouched `saved_task.rs`.
- [x] `cargo check --workspace`.
- [x] `cargo test --workspace` not run; focused crate tests run and reported in `VERIFICATION.md`.
- [x] Evidence docs filled with exact commands and results.

## Handoff

- [x] `VERIFICATION.md` contains only commands actually executed.
- [x] `AGENT_EVIDENCE.md` filled against the baseline SHA.
- [x] Parent agent owns commit/push/PR.
