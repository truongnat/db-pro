# Findings: Parameterized Transaction Session Lease Correctness

## Problem Description
Both `MySqlConnector::execute_parameterized_transaction` and `PostgresConnector::execute_parameterized_transaction` directly invoked `pool.begin().await` without inspecting `entry.open_session` (the active `SessionSlot`).

Contrast this with `execute_transaction`, which calls `checkout_if_held(session, ...)` to join an open transaction lease if one exists on the active query tab.

## Root Cause
`execute_parameterized_transaction` was missing the `checkout_if_held` check, causing staged table edits (`TableDataService::apply_mutations`) on active transaction tabs to run on an unleased pool connection.

## Severity
**P1** — Broken transaction session isolation semantics and incorrect database mutation context.
