# Verification — #448

Baseline: `b52a5088f7872f973bf5c31877c9b3698d306e8c`.
Scope: owner-approved implementation and local runtime verification; working-tree candidate based on the baseline SHA, not a new commit.

## Executed investigation

- `git status --short --branch`: initial checkout was clean, main tracking origin/main.
- `git rev-parse HEAD`: baseline above.
- `gh issue list --state open --limit 250 --json number --jq 'length'`: 156.
- Open PR listing: 13, all Draft. #448 overlap searches returned no linked/matching PR; not proof against unlinked work.
- Source reads: budget, cache replay, drive loop, existing idempotency/confirmation tests.

## Runtime probe

Command: `cargo run --offline --manifest-path /tmp/db-pro-448-probe/Cargo.toml`.
Probe: real public orchestrator, local finite repeating provider, counting tool runner, no remote API calls, no database writes.

Observed baseline output:

```text
limit=10 provider_calls=13 tool_executions=1 max_steps_failure=false completed=true
BUG #448: repeated cached calls bypassed the tool-step limit
```

Exit 101: the probe assertion requiring max-steps failure failed, reproducing #448. The final completion is supplied deliberately after twelve repeated calls to keep reproduction finite. This proves the existing ten-step budget was exceeded; an actually infinite or paid-provider run was not attempted. Execution stayed idempotent (one tool execution).

Raw command output: `artifact://8` (session artifact). Probe source/manifest are temporary and are removed after recording this result.

## Fixed-path evidence

- RED: `cargo test --locked -p db-pro-runtime --lib agent_orchestrator::tests` → 15 passed / 3 failed / 0 ignored / 24 filtered out, exit 101 (`artifact://11`). New cached-success, cached-failure, and cached-rejection budget regressions failed before the production edit. The exact-limit completion test passed.
- GREEN: same focused command after moving the existing budget check → 18 passed / 0 failed / 0 ignored / 24 filtered out, exit 0 (`artifact://15`).
- Public runtime smoke: `cargo run --offline --manifest-path /tmp/db-pro-448-smoke/Cargo.toml` → exit 0 (`artifact://15`).

```text
attempts=12 provider_calls=11 tool_executions=1 tool_events=10 max_steps_failure=true completed=false
attempts=10 provider_calls=11 tool_executions=1 tool_events=10 max_steps_failure=false completed=true
PASS: over-budget replay stops; fresh run can complete at exact limit; one execution per run
```

The smoke uses the real public orchestrator and a local scripted provider, not a live remote AI service. Its second newly constructed run verifies a fresh allowance after the first exceeded its budget. Temporary reproduction/smoke projects were removed after recording their outputs.

## Quality gates

- Initial `cargo +nightly fmt --all -- --check` failed on formatting in the four new tests only (`artifact://14`). Ran `rustfmt +nightly --edition 2021 crates/runtime/src/tests/agent_orchestrator_tests.rs`.
- Final `cargo +nightly fmt --all -- --check` → PASS, exit 0 (chained gate output `artifact://19`).
- `cargo check --workspace` → PASS, exit 0 (`artifact://19`).
- `cargo clippy --workspace --all-targets -- -D warnings` → FAIL, exit 101 (`artifact://19`): two `unnecessary_to_owned` diagnostics in `db-pro-app` production code (compiler output omitted source spans), plus three `field_reassign_with_default` diagnostics in `crates/app/src/schema/tests/schema_workbench_tests.rs:79,114,133`. No suppressions or unrelated edits were made.
- Baseline classification: `git diff b52a5088f7872f973bf5c31877c9b3698d306e8c -- crates/app` returned empty, exit 0. The failing app sources are unchanged against the exact baseline; this is not a claim that a separate baseline Clippy run was executed.
- `cargo clippy -p db-pro-runtime --all-targets -- -D warnings` → PASS, exit 0.
- The chained `cargo test --workspace` was not reached after the workspace Clippy failure. Independent `cargo test --workspace` → PASS, exit 0: 1,964 passed / 0 failed / 42 ignored across 24 suites (`artifact://23`). Ignored cases are not counted as passing or live-provider evidence.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → exit 0: 14 pass / 2 warnings / 0 failures. Both warnings are inherited in the unchanged `drive` loop (53 lines) and existing 54 clone sites; this fix adds neither clones nor a new loop.
- `python3 tools/cc-scan.py crates/runtime/src/agent_orchestrator.rs crates/runtime/src/tests/agent_orchestrator_tests.rs` → exit 0: 100/100, 0 errors / 0 warnings / 0 info.

Candidate SHA-256 (uncommitted files on baseline `b52a5088f7872f973bf5c31877c9b3698d306e8c`):

```text
6c87f6b2948c74e051a9e304f42bfd44242d9562ad3cf062fb7262ff4c5ac45f  crates/runtime/src/agent_orchestrator.rs
c463829707ac0354117ad8f24ec35f4f8e9d712f911ca27c9cb35251124cb8b8  crates/runtime/src/tests/agent_orchestrator_tests.rs
```


## Tool limitations

- Descriptive repository search failed because every judge request was rejected; literal searches and direct issue-named source reads were used instead. Tool issue reported.
- rust-analyzer references request failed: server exited unexpectedly (code 0). No symbol refactor or public API change was performed.

## Not executed / not claimed

No native UI screenshots, real remote AI-provider requests, or database-provider runs. PostgreSQL n/a and SQLite n/a independently for this runtime-only change. No CI run, commit, PR, or independent review. Workspace Rust gate results are recorded above, not inferred from focused tests.
