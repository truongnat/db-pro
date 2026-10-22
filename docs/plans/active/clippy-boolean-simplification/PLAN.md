# Clippy Boolean Simplification Fix

## Problem
In `crates/app/src/runtime/operation_events.rs`, `take_current_connection_request` contained a nonminimal boolean expression `!pending.as_ref().is_some_and(...)` which triggered Clippy warning/error `-D clippy::nonminimal_bool`:

```text
error: this boolean expression can be simplified
  --> crates/app/src/app/../runtime/operation_events.rs:12:8
   |
12 | ...if !pending.as_ref().is_some_and(|(pending_id, _)| *pending_id == request_id.0) {
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `pending.as_ref().is_none_or(|(pending_id, _)| *pending_id != request_id.0)`
```

## Solution
Simplify the boolean check to `pending.as_ref().is_none_or(|(pending_id, _)| *pending_id != request_id.0)`.

## Scope
- `crates/app/src/runtime/operation_events.rs`

## Verification
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p db-pro-app`
