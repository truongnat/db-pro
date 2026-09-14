# Multi-statement contract — criterion-by-criterion audit and one measured gap (#147)

- Session: issue-queue pass 4, 2026-09-15
- Issue: **#147** ([P1][RC1][Query] Decide and enforce multi-statement partial-write semantics)
  — parent **#14**, source audit **#129**, feeds #136/#27/#29/#105/#111
- **Base head:** `main @ 3889970` (worktree clean at session start)
- **Outcome:** every criterion in the issue body checked against the tree; **one is a measured
  correctness gap, not a documentation gap**, so the issue is **left open** with a precise owner
  decision named. The stale `docs/08` line was corrected; no product code changed.

## 1. Criterion map — issue body against `main @ 3889970`

| #147 criterion | State | Where |
|---|---|---|
| Choose **exactly one** RC1 contract (A sequential partial-write / B atomic write-batch) | **met — B chosen and recorded** | `docs/09-architecture-decisions.md` §5 "Multi-statement policy — **Implemented (updated 2026-09-15, #147/#129)**": a batch containing any mutation runs as **one transaction** through `Connector::execute_transaction` (failing statement rolls the whole batch back); a read-only batch runs statement by statement |
| Option A's UI-wording items (completed count, "earlier writes may be committed") | **not applicable under B** — and stated as such in §5 rather than left implicit | same §5; `docs/release/audit-execution-safety.md` §2 rows 6-8 |
| two reads | **met** | `execute_multi` sequential path; matrix row 6 |
| read → write | **met** | `execute_multi_routes_select_then_update` (`crates/core/src/application/query_service.rs:1034`) |
| write → failing write | **met at the envelope level, connector mocked** | `execute_multi_partial_failure_preserves_earlier_results` (`:1244`) runs `SELECT 1; UPDATE t SET x = 1; SELECT 2` against a `MockDbConnector` returning `TransactionFailure { phase: Statement, index: 1, outcome: RolledBack }`, and pins the failure index, the message/code/position and that the earlier **result** is preserved in the envelope. It does not prove the provider's rollback — the live proof is Case B of §2 (0 rows surviving) |
| read → write → failure | **met at the envelope level** | same test; plus `execute_multi_routes_mutating_cte_through_transaction_while_preserving_rows` (`:1081`) and `execute_multi_commit_failure_reports_unknown_outcome` (`:1288`) |
| DDL/write combinations | **met** | `has_schema_change` → `invalidate_schema_cache` (`:220-244`); `execute_transaction` is the single path for both classes |
| quoted semicolons / comments split correctly | **met** | `split_statements` tests (`safety.rs:1054`), `reject_multi_statement_ignores_literals_identifiers_and_comments` (`sql_policy.rs:34`), `execute_semicolon_in_string_ok` (`query_service.rs:904`) |
| cancellation after an earlier statement completed | **met, at the contract level** | matrix row 10: a cancelled execution reports cancellation, not a partial write; a cancelled transaction rolls back (`TransactionFailureOutcome::RolledBack`) |
| frontend displays failure unambiguously | **met** | `UiEvent::QueryMultiCompleted` carries the per-statement envelopes; UI tests at `crates/ui/src/app_tests.rs:3403,3480` |
| **`BEGIN`/`COMMIT`/`ROLLBACK` batch test** | **NOT met — and measuring it found a correctness gap (§2)** | classification is covered (`safety.rs:673`); **execution is not, and is wrong** |
| `known-limitations` entry | **deliberately NOT added** — see §3 | — |
| `docs/08` corrected | **was half-corrected; completed here** | item 5 already carried the superseded marker (`458ec72`); the `### Statement execution` bullet still claimed multi-statement was off — now superseded in the same style |
| `docs/09` corrected | **met** | §5, `458ec72` |
| #129 updated to the same semantics | **met** | #129 closed in pass 3 with `docs/release/audit-execution-safety.md`; its §4 states the contract in the same words |
| #131 updated to the same semantics | **met** | #131 closed in pass 3; `docs/release/audit-release-traceability.md` carries the capability with its evidence |
| #136 updated to the same semantics | **partially** — R016's owner was moved from #129 to #147; the register does not yet carry the contract or this finding | #136 comments; register update posted with this evidence (not a closure request — #136 stays open on its own owner decisions) |

## 2. The measured gap — a batch's own `COMMIT` escapes the wrapper

Method: a temporary integration test (`crates/infrastructure/tests/zz_temp_probe_147.rs`, removed
before this commit) against the live `dbpro-v01-pg-fixture`, calling
`Connector::execute_transaction` directly with a table created and dropped by the probe (fixture left
with 0 rows and no `probe_147` table). Raw output:

```
CASE A (with explicit COMMIT): phase=Statement index=3 outcome=RolledBack error=query syntax error: column "no_such_column" does not exist
CASE A rows surviving the failed batch: 1
CASE B (no explicit control): phase=Statement index=1 outcome=RolledBack error=query syntax error: column "no_such_column" does not exist
CASE B rows surviving the failed batch: 0
```

| Case | Statements | Reported outcome | Reality |
|---|---|---|---|
| **A** | `BEGIN` · `INSERT 1` · `COMMIT` · `SELECT no_such_column` | `phase=Statement index=3 outcome=RolledBack` | **1 row committed and surviving** — the explicit `COMMIT` ended the app's transaction, and the statement after it failed with nothing left to roll back |
| **B** (control) | `INSERT 1` · `SELECT no_such_column` | `phase=Statement index=1 outcome=RolledBack` | 0 rows — the documented all-or-nothing property holds |

`execute_transaction` opens `pool.begin()` and executes each statement inside it
(`crates/infrastructure/src/postgres/connector.rs:277-380`); nothing special-cases transaction
control, so a user-supplied `COMMIT` is executed as an ordinary statement and closes the wrapper's
transaction. Reachable from the product: the query editor's `Run All` sends the script to
`execute_multi`, which routes any batch with a mutation through this path
(`crates/core/src/application/query_service.rs:224,236-238`), and transaction-control keywords
classify as `Write` on purpose (fail-safe).

This is exactly the failure mode the issue was filed for — *"write A succeeds → write B fails can
leave A committed … it must not be described as atomic"* — reappearing through the one door the
contract did not close, with the extra harm that the **failure envelope still says `RolledBack`**.

## 3. Why this is not filed as a `known-limitations` entry

`docs/release/known-limitations.md` states its own rule: *"Any limitation that actually constitutes
P0/P1 correctness/security must not be hidden as a harmless known limitation."* This is a P1
product-truth/data-safety gap — the app reports a rollback that did not happen — so documenting it as
an accepted v0.1 limitation would be the discouraged disposition, and it is the contract's
enforcement that the issue asks for. The finding is therefore recorded in the execution-safety matrix
(`docs/release/audit-execution-safety.md` §3, where the classification rows already live) and reported
to the owner, not absorbed into the limitation registry.

## 4. The decision the issue now needs

The contract in `docs/09` §5 does not say what a batch may contain, and the three ways to close the
gap differ in user-visible behaviour, so the choice is the owner's rather than a workstream default:

1. **Reject** a multi-statement batch whose statements include transaction control, with a precise
   message (mirrors `reject_multi_statement`, `crates/core/src/application/sql_policy.rs:3`, and is
   consistent with manual transaction controls being `OUT_OF_SCOPE_V01` — #224). Effect: scripts that
   currently appear to run stop at dispatch. **Recommended**: it is the only option that fails closed
   and it keeps the documented contract true.
2. **Honour** explicit control inside a batch, so the user's `COMMIT` means what it says and the
   contract is restated per batch. Effect: the atomicity guarantee becomes conditional on the script,
   which has to be stated in the UI.
3. **Accept** the current behaviour as the contract. Effect: the atomicity claim in `docs/09` §5 and
   the `RolledBack` outcome both have to be weakened explicitly; the envelope would have to stop
   claiming a rollback it cannot confirm.

## 5. Gates

No product code changed in this pass (`git diff --stat HEAD -- crates/` is empty after the probe was
removed): the product-code gates are the ones recorded on the `#239` commit `ccc0a24` in this same
pass — `cargo test --workspace` **886 passed / 0 failed / 27 ignored** (baseline 883/0/27) and
CI-mirroring `--include-ignored` **913 passed / 0 failed** (baseline 910) against the same fixture. The
documentation gates were re-run after the `docs/08` and matrix edits; the fixture was started for the
probe and stopped after it.
