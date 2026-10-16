# SQLite Actor Resilience

State: REVIEW.
Issues: truongnat/db-pro#493 and #494.
Branch: `fix/sqlite-actor-resilience`.

## Goal

Make concurrent SQLite writes wait briefly for lock contention and prevent a panic in one actor handler from silently killing the worker.

## Scope

- Configure a finite 5-second `busy_timeout` for each actor connection.
- Catch unwinding panics around each actor command, return an explicit internal error, and keep the worker loop alive.
- Represent transaction-handler panics as an actor-phase failure with unknown outcome.
- Add tests for timeout configuration, per-command recovery, and unknown transaction outcome.

## Non-goals

- No WAL-mode changes or busy retry policy beyond SQLite's native timeout.
- No process-wide panic-hook customization.

## Acceptance

- [x] Every spawned SQLite connection uses a finite busy timeout.
- [x] A handler panic is reported and the actor processes a subsequent command.
- [x] Transaction panic reports its outcome as unknown.
