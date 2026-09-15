# Execution safety — destructive SQL, multi-statement, cancellation, confirmation (#129)

- **Audited baseline:** `main @ 6f75477` (issue-queue pass 3, 2026-09-15); the confirmation fix below landed
  in the same pass.
- **Issue:** #129 ([RC1][Data Safety] Audit destructive SQL, multi-statement execution, cancellation, and
  confirmation boundaries) — parent **#14**, supports #27/#29/#122/#105/#111
- **Predecessor:** the Agent-path classification gap was fixed under #147 (`49e87fe`): a script is reduced
  to its most dangerous statement before an agent tool may auto-run it. This document completes the
  remaining scope: the required per-class matrix, the parser/classifier test inventory, the confirmation
  boundary, and cancellation identity.
- **Related:** #122 (trust boundaries — its T-5 finding is settled here), #244 (export/backup cancellation),
  `docs/release/audit-data-integrity.md`.

## 1. What changed in this pass

**The query editor had no destructive-statement confirmation.** `Run` and `Run All` dispatched
`DROP`/`TRUNCATE`/`DELETE`-without-`WHERE` straight to the backend; the only gates were
`reject_multi_statement` and the read-only policy. Two independent expectations said otherwise: the
human verification runbook's **Step 13** ("a destructive-confirmation prompt appears **before** anything
reaches the backend") and the design system's unused `DestructiveOperationDialog`
(`crates/ui/src/components/transaction.rs:172-296`, reachable only from the component gallery).

Fixed in `crates/ui/src/events.rs`:

- `hold_destructive_run` classifies the text with `classify_script_safety` and, only for
  `StatementSafety::Destructive`, stores it in `pending_destructive_run` instead of dispatching;
- `confirm_pending_destructive_run` dispatches **the held text and the held buffer version** — a
  confirmation can never execute something other than what the prompt displayed;
- `cancel_pending_destructive_run` (and Escape / click-outside / close on the dialog, which reports
  `open == false`) drops it with `"Destructive statement cancelled — nothing was sent to the database"`;
- the two dispatchers were reduced to one `send_query_run` tail so the held and the direct paths cannot
  drift apart;
- the dialog (`crates/ui/src/query_view.rs`, `draw_destructive_run_dialog`) shows the statement (600-char
  preview), names the warning colour, and offers a destructive-styled confirm button.

Nothing else changed: reads, writes and plain DDL dispatch exactly as before, and the backend policy still
runs on whatever is dispatched.

## 2. Required matrix — statement/input → classifier → execution API → transaction → cancellable → confirmation → partial failure → UI result

| # | Input | Classifier (`classify_statement_safety` / `…_script_safety`) | Execution API | Transaction semantics | Cancellable | Confirmation | Partial-failure behaviour | UI result |
|---|---|---|---|---|---|---|---|---|
| 1 | `SELECT 1` | `Read` | `QueryService::execute` (`query_service.rs:145`) | none needed; server's implicit per-statement atomicity | yes — `CancelQuery` (PG: capability-gated **off**, UI says so; SQLite: yes) | no | n/a | result grid |
| 2 | `WITH x AS (SELECT 1) SELECT * FROM x` | `Read` (`classify_cte_safety`) | `execute` | as row 1 | as row 1 | no | n/a | result grid |
| 3 | `WITH x AS (DELETE FROM t RETURNING *) SELECT * FROM x` | `Write` (`classify_cte_safety` sees the mutation) | `execute` — routed as a *query* for display but classified as a mutation | as row 1 (single statement) | as row 1 | no (write, not destructive) | n/a — one statement | result grid with the returned rows |
| 4a | `INSERT …` / `UPDATE …` | `Write` | `execute` | as row 1 | as row 1 | no | n/a | affected-row summary |
| 4b | `DELETE FROM t WHERE …` | `Write` (`is_delete_without_where` is false) | `execute` | as row 1 | yes | **no** — the predicate is the user's own bound | n/a | affected-row summary |
| 4c | `DELETE FROM t` (no `WHERE`) | **`Destructive`** | `execute` | as row 1 | yes | **yes — held by the new gate** | n/a | dialog → grid after confirm |
| 5a | `CREATE …` / `ALTER …` | `Ddl` | `execute` (single) / `schema_service::execute_ddl` (from the table editor, which is *disabled*) | as row 1 | yes | no (raw SQL) | n/a | result/summary; schema cache invalidated (`invalidate_schema_cache_if_ddl`) |
| 5b | `DROP …` / `TRUNCATE …` | **`Destructive`** | `execute` | as row 1 | yes | **yes — held** | n/a | dialog → grid after confirm |
| 6 | `SELECT 1; SELECT 2;` | `Read` (`classify_script_safety` max) | `execute_multi` (`:199`) → sequential, **no** transaction (no mutation) | each statement independently atomic | yes (per connection) | no | a later failure returns the earlier results plus the failing index | per-statement results |
| 7 | `SELECT 1; UPDATE t SET x = 1;` | `Write` | `execute_multi` → **`execute_transaction`** | **one transaction, all-or-nothing** | yes | no | **nothing applied**; `TransactionFailure { phase, statement_index, outcome, results }` | error names the failing statement index |
| 8 | `UPDATE t SET x = 1; INSERT INTO t … -- fails` | `Write` | as row 7 | as row 7 | yes | no | rollback; the failing statement's server error is shown verbatim | error + index |
| 9 | `SELECT ';' AS sep;` — semicolons inside quotes/comments/dollar quotes | one statement (splitter is quote/comment/dollar-quote aware, `safety.rs:135-160`); leading comments stripped before keyword matching | `execute` | as row 1 | yes | no | n/a | as row 1 |
| 10 | cancellation during a long query | — | `UiCommand::CancelQuery { request_id }` → `query_cancellations[request_id]` | n/a | target is resolved **by request id** (`worker.rs:1524-1532`); an unknown/finished id is a no-op, so a cancel cannot hit another execution | n/a | the cancelled execution reports cancellation, not a partial write | status + query state cleared |
| 11 | stale tab/connection switch during execution | — | result routing | n/a | n/a | n/a | results are routed by `query_document_requests[request_id] → document id` (`events.rs:577,648,834,1023`), not by "the active tab", and the run is bound to `executing_version` | a late result lands in its own document even if the user switched tabs |
| 12 | agent tool SQL (contrast row) | `AgentSqlSafety::classify` (same `classify_script_safety` core) | `agent_executor.rs` → `execute_multi` | as rows 6–8 | as row 10 | **yes** — `execution_decision` requires approval unless mode `Agent` + auto-run-read-only + `ReadOnly` classification | refused before dispatch when unapproved | approval card → result |

## 3. Parser / classifier test inventory

| Area | Tests |
|---|---|
| Keyword classification, delete-without-WHERE, `DO`/`CALL`/`EXECUTE`, `MERGE`, `EXPLAIN` variants | `crates/core/src/domain/safety.rs` tests (300+ lines), incl. the dollar-quoted body cases at `:762,1054,1119` |
| Script reduction to the worst statement | `classify_script_safety` tests + the new `transaction_control_batches_classify_as_write_and_never_hide_a_mutation` (added in this pass) |
| Splitting with strings/comments/dollar quotes | `split_statements` tests at `safety.rs:1054` |
| Agent-path classification and confirmation | `agent_workflow.rs` tests incl. the `SELECT 1; DROP …` batch at `:750`, `incomplete_sql_is_confirmation_gated` (`agent.rs:458-465`) |
| Transaction control (`BEGIN`/`COMMIT`/`ROLLBACK`) | **new** in this pass: transaction-control keywords fall to the fail-safe `Write` default, a `BEGIN; SELECT; ROLLBACK;` script is `Write`, and `BEGIN; DROP …` is still `Destructive` |
| Transaction control — **execution**, not classification | **resolved in this pass, 2026-09-15, #147:** `QueryService::execute_multi` now refuses a batch whose statements carry their own `BEGIN`/`START TRANSACTION`/`COMMIT`/`END`/`ROLLBACK`/`SAVEPOINT`/`RELEASE` **before dispatch** (`transaction_control_verb`, `crates/core/src/domain/safety.rs`; refusal + `transaction_control_rejection` in `crates/core/src/application/query_service.rs`), so the wrapper transaction exists for the whole batch and the all-or-nothing property holds by construction. The gap was measured first (live PostgreSQL fixture: `execute_transaction(["BEGIN", "INSERT …", "COMMIT", "SELECT no_such_column …"])` → `phase=Statement index=3 outcome=RolledBack` **with 1 row surviving**; SQLite refuses the leading `BEGIN` itself and instead lets `INSERT; COMMIT; SELECT bad` survive with outcome `Unknown` — `docs/plans/active/multi-statement-transaction-control/PLAN.md` has both tables). Evidence: always-on `crates/infrastructure/tests/multistatement_transaction_control.rs` (4 live SQLite tests), `pg_integration.rs` (3 live PG tests, `#[ignore]`d) incl. a characterisation test that pins the connector-level `RolledBack`+surviving-row behaviour the refusal makes unreachable. Single statements are untouched (`BEGIN`/`COMMIT`/`ROLLBACK` typed alone keep working, #224) |
| The UI gate itself | **new** in this pass: `destructive_statement_is_held_until_it_is_confirmed`, `cancelling_a_held_destructive_statement_sends_nothing`, `reads_writes_and_plain_ddl_dispatch_without_a_prompt` (six statement classes), `a_script_whose_worst_statement_is_destructive_is_held`; falsified by disabling the gate (3 failures) before being trusted |
| Cancellation identity | runtime-level registration in `worker.rs`; no test asserts the "unknown request id is a no-op" branch — recorded as a gap, not claimed |

## 4. Disposition of the remaining scope items

| Item | State |
|---|---|
| destructive SQL confirmation | **fixed in this pass** (§1); runbook Step 13 is now passable as written |
| misclassification that executes a mutation under a read-only expectation | **none found**: `validate_against_policy` runs on both the single and the batch path (`query_service.rs:154,226,306`) and the classifier's unknown-keyword default is the fail-safe `Write` |
| "unrestricted raw SQL" communicated as such | the audit records it: the query editor intentionally allows arbitrary SQL; the safety contract is (a) read-only policy at the backend, (b) destructive statements held for confirmation, (c) batches with mutations atomic. `docs/09-architecture-decisions.md` §5 now states this (it previously said multi-statement was rejected and listed the confirmation as future work) |
| Data Grid mutations vs raw SQL | different mechanisms by design and both documented: the grid **stages** typed changes and applies them in one transaction (#62/#145 evidence); raw SQL is free text with the classification + confirmation + policy path |
| Action Platform confirmation vs raw query path | the agent path requires approval through `execution_decision` (row 12); the raw path now holds only `Destructive`. A mutation proposed by the agent is therefore *stricter* than one typed by the user, which is the intended asymmetry |
| audit/history recording for mutations | only the query history records a mutation (sql, status, duration, affected rows, `QueryHistoryRecord` in `events.rs`); the grid's staged change set is in-memory and discarded on discard/discard-confirm. Recorded, not claimed as an audit trail |
| `Apply DDL` from the table editor | stays disabled (`DDL_APPLY_ENABLED = false`, `table_editor_view.rs:15`) with the reason stated in the UI; the query editor with its new confirmation is the documented DDL path (LIM/`known-limitations.md:87` wording). Deciding whether to complete that gate is a separate capability call |
| back-up/restore cancellation | **not this issue's** — filed as #244 (E-3) from the #128 audit |

## 5. Not claimed

- No live destructive statement was executed against the PostgreSQL fixture by this audit; the matrix is
  source-derived and the new gate is covered by unit tests, not by an end-to-end run.
- No interactive confirmation of the dialog's appearance: no window server here (`R-GUI-SMOKE`); the
  runbook (Step 13) is the instrument that will observe it, and #91 owns that run.
- Cancellation *latency* is not measured here; the budget (< 250 ms ack) is in
  `.skills/perf-audit/references/perf-budgets.md` and the capability gating is in
  `docs/release/evidence/v01-runtime/providers/15-cancellation-capability-gating.md`.
