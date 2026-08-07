# Query Runtime Lifecycle

> Status: **Implemented** (P2-04)

## Source

- Domain model: `crates/core/src/domain/execution.rs`
- Runtime registry: `crates/tauri-app/src/cancel.rs` (`ExecutionRegistry`)
- Query command: `crates/tauri-app/src/commands/query.rs`

## Lifecycle State Machine

```text
Created → Running → Success
                    → Error
                    → Cancelled
                    → TimedOut
```

## Execution Model

```rust
QueryExecution {
    id: QueryExecutionId,
    connection_id: ConnectionId,
    status: ExecutionStatus,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    statement_count: u32,
    rows_affected: u64,
    rows_returned: u64,
}
```

## Execution Registry

The `ExecutionRegistry` replaces the old `CancelRegistry`. It tracks:

1. Each execution's lifecycle state
2. Cancel tokens (oneshot channels)
3. Result metrics (rows returned/affected, statement count)

### Key Properties

- **Cancel is idempotent**: cancelling a finished query returns `CancelResult::NotFound`
- **Timeout cleanup is deterministic**: `finish_execution` always drops the cancel token
- **No dangling state**: completed executions are removed from the registry

### API

```text
register(connection_id) → (QueryExecutionId, cancel_rx)
start_execution(exec_id)
cancel(exec_id) → CancelResult
cancel_by_connection(connection_id) → CancelResult
finish_execution(exec_id, status, rows_returned, rows_affected, statement_count)
remove(exec_id)
get_execution(exec_id) → Option<QueryExecution>
active_executions() → Vec<QueryExecution>
```

## Query Command Flow

```text
execute_query:
  1. Parse connection_id
  2. Register execution (Created state)
  3. Start execution (Running state)
  4. tokio::select! { query_future, cancel_rx }
  5a. On success → finish(Success, metrics) → remove → return Ok
  5b. On cancel → finish(Cancelled) → remove → return Err(QUERY_CANCELLED)
  5c. On timeout error → finish(TimedOut) → remove → return Err(QUERY_TIMEOUT)
  5d. On other error → finish(Error) → remove → return Err(...)
```

## Tests

- 11 domain tests for `QueryExecution` lifecycle transitions
- 13 registry tests for `ExecutionRegistry` (cancel idempotency, cleanup, timeout)
