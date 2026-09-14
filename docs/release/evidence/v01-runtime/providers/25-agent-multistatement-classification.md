# Agent SQL classification — a multi-statement batch is classified by its most dangerous statement (#147, #129)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 9fb25e9` (worktree clean at the start of the change); fix commit recorded in the ledger
- Related register/limitation entries: none changed — no release document claimed the old behaviour
- Issues: **#147** ([P1][RC1][Query] multi-statement partial-write semantics), **#129**
  ([RC1][Data Safety] destructive SQL / multi-statement execution boundaries)

## 1. The defect

The Agent tool path classified a whole SQL **string** by its leading keyword and then handed the
same string to the multi-statement executor:

| Step | Location (pre-fix) | What it did |
|---|---|---|
| 1 | `crates/core/src/domain/agent_workflow.rs:349` | `let safety = AgentSqlSafety::classify(sql);` on the *whole* script |
| 2 | `crates/core/src/domain/agent.rs:366` | `AgentSqlSafety::classify` → `classify_statement_safety(sql)`, one statement classification for the entire string |
| 3 | `crates/core/src/domain/agent.rs:392` | `ReadOnly` + `allow_read_only_auto_run` → `AgentExecutionDecision::Allowed` |
| 4 | `crates/runtime/src/agent_executor.rs:375` | `ensure_execution_confirmation` returns `Ok(())` for an allowed script |
| 5 | `crates/runtime/src/agent_executor.rs:207` | `query_api().execute_multi(connection_id, sql, ...)` — the whole batch runs |

`SELECT 1; DROP TABLE t;` therefore classified as `ReadOnly` and could auto-execute. The
interactive path does not have this hole: single-statement execution goes through
`reject_multi_statement` (`crates/core/src/application/query_service.rs:145`, `:393` via
`crates/core/src/application/sql_policy.rs:3`), the generated-draft path forces confirmation for any
semicolon-bearing draft (`crates/runtime/src/agent.rs:678-683`), and the "Run All" script path is an
explicit user action. Only the agent path could auto-run a mixed batch.

Note on the triage pointer: the guard the triage attributed to `crates/ui/src/agent.rs:678-683` is
actually `crates/runtime/src/agent.rs:678-683` (`draft_requires_confirmation`); `crates/ui/src/agent.rs`
is 378 lines and holds no such guard. The substance of the finding is confirmed.

## 2. Pre-fix reproduction (real classification path, before any fix)

Six tests were written first and run against the unmodified classifier. Raw output:
`agent-workflow` run below is the `cargo test -p db-pro-core --lib domain::agent` filter.

```
running 19 tests
test domain::agent_workflow::tests::mixed_batch_never_auto_runs_on_the_read_only_path ... FAILED
test domain::agent::tests::destructive_batch_is_never_auto_runnable ... FAILED
test domain::agent::tests::batch_is_classified_by_its_most_dangerous_statement ... FAILED

---- domain::agent_workflow::tests::mixed_batch_never_auto_runs_on_the_read_only_path stdout ----
panicked at crates/core/src/domain/agent_workflow.rs:766:9:
a batch that ends in DROP must not auto-run: Execute(AgentToolRequest { session_id: AgentSessionId(3ff7de06-...),
run_id: AgentRunId(3662b2a3-...), document_id: "doc-a", document_version: 1, tool: RunQuery,
input: Query { sql: "SELECT 1; DROP TABLE t;" } })

---- domain::agent::tests::destructive_batch_is_never_auto_runnable stdout ----
panicked at crates/core/src/domain/agent.rs:486:9:
assertion `left == right` failed
  left: Allowed
 right: RequiresConfirmation

---- domain::agent::tests::batch_is_classified_by_its_most_dangerous_statement stdout ----
panicked at crates/core/src/domain/agent.rs:464:9:
assertion `left == right` failed
  left: ReadOnly
 right: Destructive

test result: FAILED. 16 passed; 3 failed; 0 ignored; 0 measured; 276 filtered out
```

The same experiment at the runtime guard that sits directly above `execute_multi` — run with only
`crates/core/src/domain/agent.rs` reverted to the pre-fix classifier
(`git stash push -- crates/core/src/domain/agent.rs`):

```
$ cargo test -p db-pro-runtime --lib mixed_batch_is_never_auto_run_before_execute_multi
thread 'agent_executor::tests::mixed_batch_is_never_auto_run_before_execute_multi'
panicked at crates/runtime/src/agent_executor.rs:689:9:
assertion `left == right` failed
  left: Ok(())
 right: Err(ConfirmationRequired { kind: RunDestructive })
```

`left: Ok(())` with `allow_read_only_auto_run = true` is the defect: the guard approved the batch and
`run_query` went on to `execute_multi` with no confirmation. The file was restored with
`git stash pop` immediately afterwards.

## 3. The fix

One classification change, in the domain layer, so every consumer of `AgentSqlSafety::classify`
inherits it:

| File | Change |
|---|---|
| `crates/core/src/domain/safety.rs:123` | new `classify_script_safety(sql) -> Option<StatementSafety>`: splits the script and returns the result with the highest `severity_rank` |
| `crates/core/src/domain/safety.rs:70` | new `StatementSafety::severity_rank()`: read `0` < write `1` < DDL `2` < destructive `3` |
| `crates/core/src/domain/safety.rs:135` | `split_statements` moved here from `crates/core/src/application/sql_policy.rs` (same implementation, now using this module's existing token/quote/dollar-quote skippers) |
| `crates/core/src/application/sql_policy.rs:19` | `pub use crate::domain::safety::split_statements;` — the execution policy keeps the same path and behaviour; the duplicated copy (`split_statements` + 9 private helpers) was deleted |
| `crates/core/src/domain/agent.rs:371` | `AgentSqlSafety::classify` now maps `classify_script_safety` instead of `classify_statement_safety` |

There is exactly one statement splitter and one statement classifier in the crate now; the domain
layer cannot depend on the application layer, so the splitter moved down rather than the classifier
moving up. The mapping to decisions is unchanged: read → `ReadOnly`, write/DDL → `Mutating`,
destructive → `Destructive`, no statement at all → `Unknown`.

Consumers that inherit the fix without edits: `crates/core/src/domain/agent_workflow.rs:349`,
`crates/runtime/src/agent_executor.rs:375`, `crates/runtime/src/agent_orchestrator.rs:169`
(the post-approval safety recheck now compares two batch-aware classifications, so a confirmed
mixed batch no longer looks like an escalation).

Not changed, deliberately:

- **No new parser.** The existing semicolon splitter is reused; its quote, identifier, comment and
  dollar-quote handling is unchanged (`script_safety_ignores_semicolons_inside_literals_and_comments`).
- **The interactive path is untouched.** `draft_requires_confirmation` (`crates/runtime/src/agent.rs:678`)
  still forces confirmation for any draft containing a semicolon, which is *stricter* than the agent
  path, and `reject_multi_statement` still blocks multi-statement input on the single-statement
  endpoints.
- **No guard was weakened.** The change can only raise a classification (max over statements); nothing
  that was `Mutating`/`Destructive` became `ReadOnly`. Read-only batches stay auto-runnable when the
  user has opted into `allow_read_only_auto_run`, which is what that setting is for.

## 4. Tests pinning the behaviour

| Test | Location | Asserts |
|---|---|---|
| `batch_is_classified_by_its_most_dangerous_statement` | `crates/core/src/domain/agent.rs:468` | `SELECT 1; DROP TABLE t;` → `Destructive`; `SELECT 1; UPDATE …` → `Mutating`; `SELECT 1; DELETE … WHERE` → `Mutating`; `SELECT 1; SELECT 2` → `ReadOnly`; single `DROP TABLE t` → `Destructive` |
| `destructive_batch_is_never_auto_runnable` | `crates/core/src/domain/agent.rs:490` | with `allow_read_only_auto_run = true`: mixed batch → `RequiresConfirmation`, read-only batch → `Allowed` |
| `mixed_batch_never_auto_runs_on_the_read_only_path` | `crates/core/src/domain/agent_workflow.rs:750` | `request_tool(RunQuery, "SELECT 1; DROP TABLE t;")` → `ConfirmationRequired`, `safety == Destructive`, session `AwaitingConfirmation` |
| `read_only_batch_still_auto_runs_when_enabled` | `crates/core/src/domain/agent_workflow.rs:778` | `request_tool(RunQuery, "SELECT 1; SELECT 2;")` → `Execute` (unchanged behaviour) |
| `mixed_batch_is_never_auto_run_before_execute_multi` | `crates/runtime/src/agent_executor.rs:685` | the guard above `execute_multi`: mixed batch → `ConfirmationRequired { RunDestructive }`, read-only batch → `Ok(())`, confirmed mixed batch → `Ok(())` |
| `script_safety_takes_the_most_dangerous_statement` | `crates/core/src/domain/safety.rs:1078` | severity order (destructive first *and* last, DDL in the middle), single statement, empty/`;`/comment-only → `None` |
| `script_safety_ignores_semicolons_inside_literals_and_comments` | `crates/core/src/domain/safety.rs:1109` | `SELECT ';'; SELECT 2` → `Read`; `-- ;` comment → `Read`; `DO $body$ … PERFORM 1; … $body$` → `Destructive` |

The four pre-existing splitter tests moved with the function into
`crates/core/src/domain/safety.rs` (same assertions, unchanged).

## 5. Post-fix verification on this tree

```
$ cargo test -p db-pro-core --lib      # 297 passed; 0 failed; 0 ignored
$ cargo test --workspace               # 822 passed; 0 failed; 19 ignored
$ cargo fmt --all -- --check           # exit 0
$ cargo check --workspace              # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings   # exit 0
$ cargo build --release --locked -p db-pro-native         # exit 0
$ bash .skills/perf-audit/scripts/perf-scan.sh            # PASS — 4 passed, 0 warnings, 0 failed
```

Baseline for comparison: **815 passed / 0 failed / 19 ignored** (no `DATABASE_URL` set). The delta is
**+7** and is exactly the seven tests added by this change; no test changed status.

## 6. Scope and remaining work

This change closes the classification gap; it does not close either issue on its own.

- **#147** still needs the recorded multi-statement contract (transaction/rollback wording in
  `docs/release/known-limitations.md`, plus the `docs/08-technology-decisions.md:90` /
  `docs/09-architecture-decisions.md:89-101` statements that still say multi-statement is disabled)
  and the `BEGIN`/`COMMIT`/`ROLLBACK`-inside-a-batch edge case.
- **#129** still needs the execution-safety matrix as a document.
- Neither issue has a packaged-runtime leg that this environment can run; both stay open with these
  remainders written down (see `docs/plans/issue-ledger/LEDGER.md`).
- No release-truth document claimed the previous classification behaviour, so nothing in
  `docs/release/*` needed a correction for this item.
